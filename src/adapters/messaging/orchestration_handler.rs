use std::sync::Arc;

use tokio::sync::mpsc;

use crate::strategy_core::application::kill_switch::KillSwitch;
use crate::strategy_core::application::mode_controller::ModeController;
use crate::strategy_core::domain::strategy_config::StrategyConfig;
use crate::strategy_core::ports::orchestration_port::{OrchestrationAck, OrchestrationCommand};

pub struct OrchestrationHandler;

impl OrchestrationHandler {
    pub fn spawn(
        endpoint: &str,
        ack_endpoint: &str,
        kill_switch: Arc<KillSwitch>,
        mode_controller: Arc<ModeController>,
        config_path: String,
    ) -> mpsc::Sender<OrchestrationCommand> {
        let (tx, mut rx) = mpsc::channel::<OrchestrationCommand>(64);
        let endpoint = endpoint.to_string();
        let ack_endpoint = ack_endpoint.to_string();

        tokio::spawn(async move {
            let ctx = zmq::Context::new();
            let socket = ctx.socket(zmq::SUB).expect("Failed to create SUB socket");
            socket.set_subscribe(b"orchestrator.control.strategy.").expect("Failed to set subscription");
            socket.connect(&endpoint).expect("Failed to connect to orchestrator endpoint");

            let ack_socket = ctx.socket(zmq::PUB).expect("Failed to create ack PUB socket");
            ack_socket.bind(&ack_endpoint).expect("Failed to bind ack endpoint");

            tracing::info!(endpoint = %endpoint, "Orchestration handler connected");

            while let Some(cmd) = rx.recv().await {
                let command_type = command_type_name(&cmd);
                let (success, error) = Self::dispatch(
                    cmd,
                    &kill_switch,
                    &mode_controller,
                    &config_path,
                );

                let ack = OrchestrationAck {
                    command_type,
                    success,
                    error,
                    timestamp_ms: current_time_ms(),
                };

                let ack_bytes = match serde_json::to_vec(&ack) {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        tracing::error!(error = %e, "Failed to serialize orchestration ack");
                        continue;
                    }
                };

                let topic = "service.strategy.control.ack";
                if let Err(e) = ack_socket.send(topic.as_bytes(), zmq::SNDMORE) {
                    tracing::error!(error = %e, "Failed to send ack topic");
                }
                if let Err(e) = ack_socket.send(&ack_bytes, 0) {
                    tracing::error!(error = %e, "Failed to send ack payload");
                }
            }

            tracing::info!("Orchestration handler shutting down");
        });

        tx
    }

    fn dispatch(
        cmd: OrchestrationCommand,
        kill_switch: &Arc<KillSwitch>,
        mode_controller: &Arc<ModeController>,
        config_path: &str,
    ) -> (bool, Option<String>) {
        match cmd {
            OrchestrationCommand::SetOperationMode(mode) => {
                mode_controller.set(mode);
                (true, None)
            }
            OrchestrationCommand::ActivateKillSwitch { reason } => {
                kill_switch.activate(reason);
                (true, None)
            }
            OrchestrationCommand::ClearKillSwitch => {
                kill_switch.clear();
                (true, None)
            }
            OrchestrationCommand::ReloadStrategyConfig => {
                match StrategyConfig::from_file(config_path) {
                    Ok(_config) => {
                        // Config loaded — caller should apply via engine.reload_config()
                        (true, None)
                    }
                    Err(e) => (false, Some(e)),
                }
            }
            _ => (true, None),
        }
    }
}

fn command_type_name(cmd: &OrchestrationCommand) -> String {
    match cmd {
        OrchestrationCommand::SetOperationMode(_) => "SetOperationMode",
        OrchestrationCommand::ActivateKillSwitch { .. } => "ActivateKillSwitch",
        OrchestrationCommand::ClearKillSwitch => "ClearKillSwitch",
        OrchestrationCommand::ReloadStrategyConfig => "ReloadStrategyConfig",
        OrchestrationCommand::SetStrategyEnabled { .. } => "SetStrategyEnabled",
        OrchestrationCommand::SetLongEntryThreshold { .. } => "SetLongEntryThreshold",
        OrchestrationCommand::SetShortEntryThreshold { .. } => "SetShortEntryThreshold",
        OrchestrationCommand::SetConfidenceMinimum { .. } => "SetConfidenceMinimum",
        OrchestrationCommand::SetHysteresisBand { .. } => "SetHysteresisBand",
        OrchestrationCommand::SetEntryCooldown { .. } => "SetEntryCooldown",
        OrchestrationCommand::SetExitCooldown { .. } => "SetExitCooldown",
        OrchestrationCommand::SetRegimeGate { .. } => "SetRegimeGate",
        OrchestrationCommand::SetMaxLongUnits { .. } => "SetMaxLongUnits",
        OrchestrationCommand::SetMaxShortUnits { .. } => "SetMaxShortUnits",
        OrchestrationCommand::SetAllowShort { .. } => "SetAllowShort",
        OrchestrationCommand::SetSizingMethod { .. } => "SetSizingMethod",
        OrchestrationCommand::SetBaseSize { .. } => "SetBaseSize",
        OrchestrationCommand::PauseSymbol { .. } => "PauseSymbol",
        OrchestrationCommand::ResumeSymbol { .. } => "ResumeSymbol",
        OrchestrationCommand::FlattenSymbol { .. } => "FlattenSymbol",
        OrchestrationCommand::ResetCooldown { .. } => "ResetCooldown",
        OrchestrationCommand::ResetHysteresis { .. } => "ResetHysteresis",
    }
    .to_string()
}

fn current_time_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
