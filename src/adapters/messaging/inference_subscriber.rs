use std::sync::Arc;
use std::thread;

use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::strategy_core::ports::inference_input_port::{IInferenceInputPort, InferenceMessage};

pub struct InferenceSubscriber {
    receiver: Arc<tokio::sync::Mutex<mpsc::Receiver<InferenceMessage>>>,
}

impl InferenceSubscriber {
    pub fn spawn(endpoint: &str, topic_prefix: &str) -> Self {
        let (tx, rx) = mpsc::channel::<InferenceMessage>(1024);
        let endpoint = endpoint.to_string();
        let topic_prefix = topic_prefix.to_string();

        thread::spawn(move || {
            let ctx = zmq::Context::new();
            let socket = ctx.socket(zmq::SUB).expect("Failed to create SUB socket");

            socket.set_subscribe(topic_prefix.as_bytes()).expect("Failed to set subscription");
            socket.connect(&endpoint).expect("Failed to connect to model endpoint");

            tracing::info!(endpoint = %endpoint, topic = %topic_prefix, "Inference subscriber connected");

            loop {
                let topic_bytes = socket.recv_bytes(0).expect("Failed to receive topic");
                let payload = socket.recv_bytes(0).expect("Failed to receive payload");

                let received_ns = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64;

                let msg = InferenceMessage {
                    topic: String::from_utf8_lossy(&topic_bytes).to_string(),
                    payload,
                    received_ns,
                };

                if tx.blocking_send(msg).is_err() {
                    tracing::info!("Inference subscriber channel closed, exiting");
                    break;
                }
            }
        });

        Self {
            receiver: Arc::new(tokio::sync::Mutex::new(rx)),
        }
    }
}

#[async_trait]
impl IInferenceInputPort for InferenceSubscriber {
    async fn recv(&self) -> Option<InferenceMessage> {
        let mut rx = self.receiver.lock().await;
        rx.recv().await
    }
}
