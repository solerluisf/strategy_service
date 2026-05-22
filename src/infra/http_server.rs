use std::sync::Arc;

use axum::{Router, routing::get, Json};

use crate::strategy_core::ports::health_port::IHealthReporter;

pub struct HttpServer;

impl HttpServer {
    pub async fn spawn(
        port: u16,
        health_reporter: Arc<dyn IHealthReporter>,
    ) -> tokio::task::JoinHandle<()> {
        let app = Router::new()
            .route("/health", get({
                let reporter = health_reporter.clone();
                move || health_handler(reporter)
            }));

        let addr = format!("0.0.0.0:{}", port);
        tracing::info!(addr = %addr, "HTTP server starting");

        tokio::spawn(async move {
            let listener = match tokio::net::TcpListener::bind(&addr).await {
                Ok(l) => l,
                Err(e) => {
                    tracing::error!(error = %e, "Failed to bind HTTP server");
                    return;
                }
            };

            if let Err(e) = axum::serve(listener, app).await {
                tracing::error!(error = %e, "HTTP server error");
            }
        })
    }
}

async fn health_handler(reporter: Arc<dyn IHealthReporter>) -> Json<serde_json::Value> {
    let health = reporter.get_health().await;
    let json = serde_json::json!({
        "timestamp_ms": health.timestamp_ms,
        "overall_status": health.overall_status,
        "kill_switch_active": health.kill_switch_active,
        "operation_mode": health.operation_mode,
        "strategy_enabled": health.strategy_enabled,
        "strategy_id": health.strategy_id,
        "strategy_version": health.strategy_version,
        "intents_generated_total": health.intents_generated_total,
        "intents_suppressed_by_threshold": health.intents_suppressed_by_threshold,
        "intents_suppressed_by_hysteresis": health.intents_suppressed_by_hysteresis,
        "intents_suppressed_by_cooldown": health.intents_suppressed_by_cooldown,
        "intents_suppressed_by_regime": health.intents_suppressed_by_regime,
        "intents_suppressed_by_position": health.intents_suppressed_by_position,
        "intents_suppressed_by_staleness": health.intents_suppressed_by_staleness,
        "per_symbol": health.per_symbol,
    });
    Json(json)
}
