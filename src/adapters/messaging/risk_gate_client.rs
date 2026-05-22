use std::collections::HashMap;
use std::thread;

use async_trait::async_trait;
use tokio::sync::{mpsc, oneshot};

use crate::strategy_core::domain::errors::StrategyError;
use crate::strategy_core::domain::trade_intent::TradeIntent;
use crate::strategy_core::ports::risk_gate_port::{IRiskGateClient, RiskCheckRequest, RiskDecision};
use crate::adapters::messaging::wire_codec::encode_msgpack;

pub enum RiskGateCommand {
    Check {
        intent: TradeIntent,
        request_id: String,
        response_tx: oneshot::Sender<Result<RiskDecision, StrategyError>>,
    },
    Shutdown,
}

pub struct RiskGateClient {
    sender: mpsc::Sender<RiskGateCommand>,
}

impl RiskGateClient {
    pub fn spawn(endpoint: &str) -> Self {
        let (tx, mut rx) = mpsc::channel::<RiskGateCommand>(256);
        let endpoint_owned = endpoint.to_string();

        thread::spawn(move || {
            let ctx = zmq::Context::new();
            let socket = ctx.socket(zmq::DEALER).expect("Failed to create DEALER socket");
            socket.connect(&endpoint_owned).expect("Failed to connect to risk router");

            tracing::info!(endpoint = %endpoint_owned, "Risk gate client connected");

            let mut pending: HashMap<String, oneshot::Sender<Result<RiskDecision, StrategyError>>> = HashMap::new();

            loop {
                let poll_result = socket.poll(zmq::POLLIN, 100);

                if let Ok(events) = poll_result {
                    if events > 0 {
                        if let Ok(bytes) = socket.recv_bytes(0) {
                            let decoded: Result<(RiskDecision, _), _> =
                                crate::adapters::messaging::wire_codec::decode_msgpack(&bytes);

                            if let Ok((decision, _)) = decoded {
                                if let Some(tx) = pending.remove(&decision.request_id) {
                                    let _ = tx.send(Ok(decision));
                                }
                            }
                        }
                    }
                }

                match rx.try_recv() {
                    Ok(RiskGateCommand::Check { intent, request_id, response_tx }) => {
                        let request = RiskCheckRequest {
                            intent,
                            request_id: request_id.clone(),
                            timestamp_ns: current_time_ns(),
                        };

                        let encoded = match encode_msgpack(&request) {
                            Ok(bytes) => bytes,
                            Err(e) => {
                                let _ = response_tx.send(Err(StrategyError::Encode(e)));
                                continue;
                            }
                        };

                        let request_id_for_pending = request_id.clone();
                        pending.insert(request_id_for_pending, response_tx);

                        if let Err(e) = socket.send(&encoded, 0) {
                            tracing::error!(error = %e, "Failed to send risk check request");
                            if let Some(tx) = pending.remove(&request_id) {
                                let _ = tx.send(Err(StrategyError::Messaging(e.to_string())));
                            }
                        }
                    }
                    Ok(RiskGateCommand::Shutdown) => break,
                    Err(_) => {}
                }
            }

            for (_, tx) in pending.drain() {
                let _ = tx.send(Err(StrategyError::Shutdown));
            }

            tracing::info!("Risk gate client shutting down");
        });

        Self { sender: tx }
    }
}

#[async_trait]
impl IRiskGateClient for RiskGateClient {
    async fn check_risk(&self, intent: TradeIntent) -> Result<RiskDecision, StrategyError> {
        let request_id = uuid::Uuid::new_v4().to_string();
        let (tx, rx) = oneshot::channel();

        self.sender
            .send(RiskGateCommand::Check { intent, request_id, response_tx: tx })
            .await
            .map_err(|e| StrategyError::Messaging(format!("Risk gate channel closed: {}", e)))?;

        rx.await
            .map_err(|e| StrategyError::Messaging(format!("Risk check response channel dropped: {}", e)))?
    }
}

fn current_time_ns() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}
