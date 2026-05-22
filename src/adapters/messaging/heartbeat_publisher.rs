use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;

pub enum HeartbeatCommand {
    Shutdown,
}

pub struct HeartbeatPublisher;

impl HeartbeatPublisher {
    pub fn spawn(
        endpoint: &str,
        kill_switch_active: bool,
        operation_mode: String,
        strategy_enabled: bool,
    ) -> mpsc::Sender<HeartbeatCommand> {
        let (tx, mut rx) = mpsc::channel::<HeartbeatCommand>(16);
        let endpoint = endpoint.to_string();

        tokio::spawn(async move {
            let ctx = zmq::Context::new();
            let socket = ctx.socket(zmq::PUB).expect("Failed to create PUB socket");
            socket.bind(&endpoint).expect("Failed to bind heartbeat PUB socket");

            tracing::info!(endpoint = %endpoint, "Heartbeat publisher bound");

            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        let hb = StrategyHeartbeat {
                            timestamp_ms: current_time_ms(),
                            kill_switch_active,
                            operation_mode: operation_mode.clone(),
                            overall_status: if kill_switch_active { "stopped".to_string() } else { "running".to_string() },
                            strategy_enabled,
                            intents_generated_total: 0,
                            intents_suppressed_total: 0,
                            symbols_tracked: 0,
                            per_symbol_cooldown_active: HashMap::new(),
                        };

                        let encoded = match serde_json::to_vec(&hb) {
                            Ok(bytes) => bytes,
                            Err(e) => {
                                tracing::error!(error = %e, "Failed to serialize heartbeat");
                                continue;
                            }
                        };

                        let topic = "service.strategy.health";
                        if let Err(e) = socket.send(topic.as_bytes(), zmq::SNDMORE) {
                            tracing::error!(error = %e, "Failed to send heartbeat topic");
                        }
                        if let Err(e) = socket.send(&encoded, 0) {
                            tracing::error!(error = %e, "Failed to send heartbeat payload");
                        }
                    }
                    Some(cmd) = rx.recv() => {
                        if matches!(cmd, HeartbeatCommand::Shutdown) {
                            break;
                        }
                    }
                }
            }

            tracing::info!("Heartbeat publisher shutting down");
        });

        tx
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyHeartbeat {
    pub timestamp_ms: u64,
    pub kill_switch_active: bool,
    pub operation_mode: String,
    pub overall_status: String,
    pub strategy_enabled: bool,
    pub intents_generated_total: u64,
    pub intents_suppressed_total: u64,
    pub symbols_tracked: u32,
    pub per_symbol_cooldown_active: HashMap<String, bool>,
}

fn current_time_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
