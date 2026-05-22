use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::strategy_core::domain::errors::StrategyError;
use crate::strategy_core::domain::trade_intent::TradeIntent;
use crate::strategy_core::ports::intent_pub_port::IIntentPublisher;
use crate::adapters::messaging::wire_codec::encode_trade_intent;

pub enum IntentPublisherCommand {
    Publish(TradeIntent),
    Shutdown,
}

pub struct IntentPublisher {
    sender: mpsc::Sender<IntentPublisherCommand>,
}

impl IntentPublisher {
    pub fn spawn(endpoint: &str, topic_prefix: &str) -> Self {
        let (tx, mut rx) = mpsc::channel::<IntentPublisherCommand>(256);
        let endpoint = endpoint.to_string();
        let topic_prefix = topic_prefix.to_string();

        tokio::spawn(async move {
            let ctx = zmq::Context::new();
            let socket = ctx.socket(zmq::PUB).expect("Failed to create PUB socket");
            socket.bind(&endpoint).expect("Failed to bind intent PUB socket");

            tracing::info!(endpoint = %endpoint, "Intent publisher bound");

            while let Some(cmd) = rx.recv().await {
                match cmd {
                    IntentPublisherCommand::Publish(intent) => {
                        let topic = format!("{}{}", topic_prefix, intent.symbol);
                        let encoded = match encode_trade_intent(&intent) {
                            Ok(bytes) => bytes,
                            Err(e) => {
                                tracing::error!(error = %e, "Failed to encode trade intent");
                                continue;
                            }
                        };

                        if let Err(e) = socket.send(topic.as_bytes(), zmq::SNDMORE) {
                            tracing::error!(error = %e, "Failed to send topic frame");
                            continue;
                        }
                        if let Err(e) = socket.send(&encoded, 0) {
                            tracing::error!(error = %e, "Failed to send payload frame");
                        }
                    }
                    IntentPublisherCommand::Shutdown => break,
                }
            }

            tracing::info!("Intent publisher shutting down");
        });

        Self { sender: tx }
    }

    pub async fn publish(&self, intent: &TradeIntent) -> Result<(), StrategyError> {
        self.sender
            .send(IntentPublisherCommand::Publish(intent.clone()))
            .await
            .map_err(|e| StrategyError::Messaging(format!("Intent publisher channel closed: {}", e)))
    }
}

#[async_trait]
impl IIntentPublisher for IntentPublisher {
    async fn publish(&self, intent: &TradeIntent) -> Result<(), StrategyError> {
        self.publish(intent).await
    }
}
