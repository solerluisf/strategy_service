use std::sync::Arc;
use std::thread;

use tokio::sync::mpsc;

use crate::strategy_core::application::mode_controller::ModeController;
use crate::strategy_core::domain::operation_mode::OperationMode;

pub enum ModeCommand {
    Shutdown,
}

pub struct ModeSubscriber;

impl ModeSubscriber {
    pub fn spawn(
        endpoint: &str,
        ack_endpoint: &str,
        mode_controller: Arc<ModeController>,
    ) -> mpsc::Sender<ModeCommand> {
        let (tx, mut rx) = mpsc::channel::<ModeCommand>(16);
        let endpoint = endpoint.to_string();
        let ack_endpoint = ack_endpoint.to_string();

        thread::spawn(move || {
            let ctx = zmq::Context::new();
            let socket = ctx.socket(zmq::SUB).expect("Failed to create SUB socket");
            socket.set_subscribe(b"").expect("Failed to set subscription");
            socket.connect(&endpoint).expect("Failed to connect to mode endpoint");

            let ack_socket = ctx.socket(zmq::PUB).expect("Failed to create ack PUB socket");
            ack_socket.bind(&ack_endpoint).expect("Failed to bind ack endpoint");

            tracing::info!(endpoint = %endpoint, "Mode subscriber connected");

            loop {
                let poll_result = socket.poll(zmq::POLLIN, 100);

                if let Ok(events) = poll_result {
                    if events > 0 {
                        let _topic = socket.recv_bytes(0).expect("Failed to receive topic");
                        let payload = socket.recv_bytes(0).expect("Failed to receive payload");

                        if let Ok(mode_str) = String::from_utf8(payload) {
                            if let Ok(mode) = mode_str.parse::<OperationMode>() {
                                mode_controller.set(mode);
                                tracing::info!(?mode, "Operation mode changed via ZMQ");

                                let ack = serde_json::json!({
                                    "command_type": "SetOperationMode",
                                    "success": true,
                                    "timestamp_ms": current_time_ms(),
                                });
                                if let Ok(ack_bytes) = serde_json::to_vec(&ack) {
                                    let _ = ack_socket.send(
                                        "service.strategy.control.ack".as_bytes(),
                                        zmq::SNDMORE,
                                    );
                                    let _ = ack_socket.send(&ack_bytes, 0);
                                }
                            }
                        }
                    }
                }

                match rx.try_recv() {
                    Ok(ModeCommand::Shutdown) => break,
                    Err(_) => {}
                }
            }

            tracing::info!("Mode subscriber shutting down");
        });

        tx
    }
}

fn current_time_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
