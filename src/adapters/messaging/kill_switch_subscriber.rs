use std::sync::Arc;
use std::thread;

use tokio::sync::mpsc;

use crate::strategy_core::application::kill_switch::KillSwitch;

pub enum KillSwitchCommand {
    Shutdown,
}

pub struct KillSwitchSubscriber;

impl KillSwitchSubscriber {
    pub fn spawn(
        endpoint: &str,
        kill_switch: Arc<KillSwitch>,
    ) -> mpsc::Sender<KillSwitchCommand> {
        let (tx, mut rx) = mpsc::channel::<KillSwitchCommand>(16);
        let endpoint = endpoint.to_string();

        thread::spawn(move || {
            let ctx = zmq::Context::new();
            let socket = ctx.socket(zmq::SUB).expect("Failed to create SUB socket");
            socket.set_subscribe(b"").expect("Failed to set subscription");
            socket.connect(&endpoint).expect("Failed to connect to kill switch endpoint");

            tracing::info!(endpoint = %endpoint, "Kill switch subscriber connected");

            loop {
                let poll_result = socket.poll(zmq::POLLIN, 100);

                if let Ok(events) = poll_result {
                    if events > 0 {
                        let _topic = socket.recv_bytes(0).expect("Failed to receive topic");
                        let payload = socket.recv_bytes(0).expect("Failed to receive payload");

                        if let Ok(reason) = String::from_utf8(payload) {
                            kill_switch.activate(reason.clone());
                            tracing::warn!(reason = %reason, "Kill switch activated via ZMQ");
                        }
                    }
                }

                match rx.try_recv() {
                    Ok(KillSwitchCommand::Shutdown) => break,
                    Err(_) => {}
                }
            }

            tracing::info!("Kill switch subscriber shutting down");
        });

        tx
    }
}
