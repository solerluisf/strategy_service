use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug)]
pub struct KillSwitch {
    active: AtomicBool,
    reason: std::sync::RwLock<Option<String>>,
}

impl KillSwitch {
    pub fn new() -> Self {
        Self {
            active: AtomicBool::new(false),
            reason: std::sync::RwLock::new(None),
        }
    }

    pub fn activate(&self, reason: String) {
        self.active.store(true, Ordering::SeqCst);
        *self.reason.write().unwrap() = Some(reason.clone());
        tracing::warn!(reason = %reason, "Kill switch activated");
    }

    pub fn clear(&self) {
        self.active.store(false, Ordering::SeqCst);
        *self.reason.write().unwrap() = None;
        tracing::info!("Kill switch cleared");
    }

    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }

    pub fn reason(&self) -> Option<String> {
        self.reason.read().unwrap().clone()
    }
}

impl Default for KillSwitch {
    fn default() -> Self {
        Self::new()
    }
}

pub type SharedKillSwitch = Arc<KillSwitch>;
