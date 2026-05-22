use std::sync::Arc;

use crate::strategy_core::application::strategy_engine::StrategyEngine;
use crate::strategy_core::domain::errors::StrategyError;
use crate::strategy_core::ports::inference_input_port::{IInferenceInputPort, InferenceMessage};
use crate::strategy_core::ports::intent_pub_port::IIntentPublisher;
use crate::strategy_core::ports::journal_port::IJournalPort;
use crate::strategy_core::ports::lifecycle_input_port::{ILifecycleInputPort, LifecycleMessage};
use crate::strategy_core::ports::risk_gate_port::IRiskGateClient;
use crate::strategy_core::domain::position::PositionCache;
use crate::adapters::messaging::wire_codec::{decode_inference_event, decode_order_lifecycle_event};

pub struct StrategyService {
    engine: StrategyEngine,
    inference_input: Arc<dyn IInferenceInputPort>,
    lifecycle_input: Arc<dyn ILifecycleInputPort>,
    position_cache: Arc<PositionCache>,
    publisher: Arc<dyn IIntentPublisher>,
    risk_gate: Arc<dyn IRiskGateClient>,
    journal: Arc<dyn IJournalPort>,
}

impl StrategyService {
    pub fn new(
        engine: StrategyEngine,
        inference_input: Arc<dyn IInferenceInputPort>,
        lifecycle_input: Arc<dyn ILifecycleInputPort>,
        position_cache: Arc<PositionCache>,
        publisher: Arc<dyn IIntentPublisher>,
        risk_gate: Arc<dyn IRiskGateClient>,
        journal: Arc<dyn IJournalPort>,
    ) -> Self {
        Self {
            engine,
            inference_input,
            lifecycle_input,
            position_cache,
            publisher,
            risk_gate,
            journal,
        }
    }

    pub async fn run(
        mut self,
        mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
    ) {
        loop {
            tokio::select! {
                Some(msg) = self.inference_input.recv() => {
                    if let Err(e) = self.process_inference(msg).await {
                        tracing::error!(error = %e, "Error processing inference");
                    }
                }
                Some(msg) = self.lifecycle_input.recv() => {
                    if let Err(e) = self.process_lifecycle(msg).await {
                        tracing::error!(error = %e, "Error processing lifecycle");
                    }
                }
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        tracing::info!("Shutdown signal received, stopping StrategyService");
                        break;
                    }
                }
            }
        }
    }

    pub async fn process_inference(
        &mut self,
        msg: InferenceMessage,
    ) -> Result<(), StrategyError> {
        let now_ns = current_time_ns();

        let (event, _wire_format) = match decode_inference_event(&msg.payload) {
            Ok(decoded) => decoded,
            Err(e) => {
                tracing::warn!(error = %e, "Failed to decode inference event");
                return Ok(());
            }
        };

        let intent_opt = self.engine.evaluate(event, now_ns)?;

        if let Some(intent) = intent_opt {
            self.journal.append_intent(&intent).await?;
            self.publisher.publish(&intent).await?;

            let risk_decision = self.risk_gate.check_risk(intent.clone()).await?;

            if risk_decision.approved {
                tracing::info!(
                    intent_id = %intent.intent_id, symbol = %intent.symbol,
                    "Intent approved by risk"
                );
            } else {
                tracing::warn!(
                    intent_id = %intent.intent_id, symbol = %intent.symbol,
                    reason = ?risk_decision.reason,
                    "Intent rejected by risk"
                );
            }
        }

        Ok(())
    }

    pub async fn process_lifecycle(
        &mut self,
        msg: LifecycleMessage,
    ) -> Result<(), StrategyError> {
        let (event, _wire_format) = match decode_order_lifecycle_event(&msg.payload) {
            Ok(decoded) => decoded,
            Err(e) => {
                tracing::warn!(error = %e, "Failed to decode lifecycle event");
                return Ok(());
            }
        };

        self.position_cache.apply_lifecycle_event(&event);

        Ok(())
    }
}

fn current_time_ns() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}
