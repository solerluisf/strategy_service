use std::sync::Arc;

use strategy_service::config::app_config::AppConfig;
use strategy_service::strategy_core::application::kill_switch::KillSwitch;
use strategy_service::strategy_core::application::mode_controller::ModeController;
use strategy_service::strategy_core::application::strategy_engine::StrategyEngine;
use strategy_service::strategy_core::application::strategy_service::StrategyService;
use strategy_service::strategy_core::domain::position::PositionCache;
use strategy_service::strategy_core::domain::strategy_config::StrategyConfig;
use strategy_service::adapters::messaging::heartbeat_publisher::HeartbeatPublisher;
use strategy_service::adapters::messaging::inference_subscriber::InferenceSubscriber;
use strategy_service::adapters::messaging::intent_publisher::IntentPublisher;
use strategy_service::adapters::messaging::kill_switch_subscriber::KillSwitchSubscriber;
use strategy_service::adapters::messaging::lifecycle_subscriber::LifecycleSubscriber;
use strategy_service::adapters::messaging::mode_subscriber::ModeSubscriber;
use strategy_service::adapters::messaging::orchestration_handler::OrchestrationHandler;
use strategy_service::adapters::messaging::risk_gate_client::RiskGateClient;
use strategy_service::adapters::metrics::health_endpoint::HealthReporter;
use strategy_service::adapters::metrics::metrics_adapter::MetricsAdapter;
use strategy_service::adapters::persistence::journal_storage::JournalStorage;
use strategy_service::infra::http_server::HttpServer;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let config = AppConfig::from_env().expect("Failed to load app config");

    init_tracing();

    let strategy_config = load_strategy_config(&config.strategy_config_path);

    tracing::info!(
        strategy_id = %strategy_config.strategy_id,
        version = %strategy_config.strategy_version,
        "Strategy Service starting"
    );

    let kill_switch: Arc<KillSwitch> = Arc::new(KillSwitch::new());
    let mode_controller: Arc<ModeController> = Arc::new(ModeController::new(config.operation_mode.clone()));
    let journal = Arc::new(JournalStorage::open(&config.journal_db_path).expect("Failed to open journal"));
    let position_cache: Arc<PositionCache> = Arc::new(PositionCache::new());
    let metrics: Arc<MetricsAdapter> = Arc::new(MetricsAdapter::new());

    let inference_subscriber: Arc<InferenceSubscriber> =
        Arc::new(InferenceSubscriber::spawn(&config.model_endpoint, "service.model.inference."));

    let lifecycle_subscriber: Arc<LifecycleSubscriber> =
        Arc::new(LifecycleSubscriber::spawn(&config.gateway_lifecycle_endpoint, "order.lifecycle."));

    let intent_publisher: Arc<IntentPublisher> =
        Arc::new(IntentPublisher::spawn(&config.publish_endpoint, "service.strategy.intent."));

    let risk_gate: Arc<RiskGateClient> =
        Arc::new(RiskGateClient::spawn(&config.risk_router_endpoint));

    let engine = StrategyEngine::new(
        strategy_config,
        position_cache.clone(),
        kill_switch.clone(),
        mode_controller.clone(),
        metrics.clone(),
    );

    let service = StrategyService::new(
        engine,
        inference_subscriber,
        lifecycle_subscriber,
        position_cache.clone(),
        intent_publisher,
        risk_gate,
        journal.clone(),
    );

    recover_kill_switch_state(&journal, &kill_switch);
    recover_mode_state(&journal, &mode_controller);

    let health_reporter: Arc<HealthReporter> = Arc::new(HealthReporter::new(
        kill_switch.clone(),
        mode_controller.clone(),
        position_cache.clone(),
    ));

    let _http_handle = HttpServer::spawn(config.health_port, health_reporter.clone()).await;

    let _heartbeat_tx = HeartbeatPublisher::spawn(
        &config.ack_endpoint,
        kill_switch.is_active(),
        mode_controller.get().to_string(),
        true,
    );

    let _kill_switch_tx = KillSwitchSubscriber::spawn(
        &config.control_endpoint,
        kill_switch.clone(),
    );

    let _mode_tx = ModeSubscriber::spawn(
        &config.control_endpoint,
        &config.ack_endpoint,
        mode_controller.clone(),
    );

    let _orch_tx = OrchestrationHandler::spawn(
        &config.control_endpoint,
        &config.ack_endpoint,
        kill_switch.clone(),
        mode_controller.clone(),
        config.strategy_config_path.clone(),
    );

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

    let service_handle = tokio::spawn(service.run(shutdown_rx));

    tracing::info!("Strategy Service fully initialized, awaiting signals");

    wait_for_shutdown().await;

    tracing::info!("Shutdown signal received, stopping all components");

    shutdown_tx.send(true).expect("Failed to send shutdown signal");

    let _ = service_handle.await;

    tracing::info!("Strategy Service stopped");
}

fn init_tracing() {
    let env_filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .init();
}

fn load_strategy_config(path: &str) -> StrategyConfig {
    match StrategyConfig::from_file(path) {
        Ok(config) => {
            tracing::info!(path = %path, strategy_id = %config.strategy_id, "Strategy config loaded");
            config
        }
        Err(e) => {
            tracing::warn!(error = %e, "Failed to load strategy config, using defaults");
            StrategyConfig::default()
        }
    }
}

fn recover_kill_switch_state(journal: &JournalStorage, kill_switch: &KillSwitch) {
    if let Ok(Some(data)) = journal.get_last_control_event("kill_switch") {
        if let Ok(reason) = String::from_utf8(data) {
            kill_switch.activate(reason.clone());
            tracing::warn!(reason = %reason, "Kill switch state recovered from journal");
        }
    }
}

fn recover_mode_state(journal: &JournalStorage, mode_controller: &ModeController) {
    if let Ok(Some(data)) = journal.get_last_control_event("mode") {
        if let Ok(mode_str) = String::from_utf8(data) {
            if let Ok(mode) = mode_str.parse() {
                mode_controller.set(mode);
                tracing::info!(?mode, "Operation mode recovered from journal");
            }
        }
    }
}

async fn wait_for_shutdown() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        let mut sigterm = signal(SignalKind::Term).expect("Failed to register SIGTERM handler");
        let mut sigint = signal(SignalKind::Interrupt).expect("Failed to register SIGINT handler");

        tokio::select! {
            _ = sigterm.recv() => tracing::info!("Received SIGTERM"),
            _ = sigint.recv() => tracing::info!("Received SIGINT"),
        }
    }

    #[cfg(windows)]
    {
        use tokio::signal::windows;
        let mut ctrl_c = windows::ctrl_c().expect("Failed to register Ctrl+C handler");
        let mut ctrl_break = windows::ctrl_break().expect("Failed to register Ctrl+Break handler");

        tokio::select! {
            _ = ctrl_c.recv() => tracing::info!("Received Ctrl+C"),
            _ = ctrl_break.recv() => tracing::info!("Received Ctrl+Break"),
        }
    }
}
