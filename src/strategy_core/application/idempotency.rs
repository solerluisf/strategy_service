use std::collections::HashSet;
use std::sync::Mutex;
use std::time::Instant;

use crate::strategy_core::domain::errors::StrategyError;

pub struct IdempotencyStore {
    seen: Mutex<HashSet<String>>,
    last_cleanup: Mutex<Instant>,
    cleanup_interval: std::time::Duration,
}

impl IdempotencyStore {
    pub fn new(_ttl_secs: u64) -> Self {
        Self {
            seen: Mutex::new(HashSet::new()),
            last_cleanup: Mutex::new(Instant::now()),
            cleanup_interval: std::time::Duration::from_secs(60),
        }
    }

    pub fn seen(&self, intent_id: &str) -> bool {
        self.seen.lock().unwrap().contains(intent_id)
    }

    pub fn mark_seen(&self, intent_id: &str) -> Result<(), StrategyError> {
        let mut store = self.seen.lock().unwrap();
        store.insert(intent_id.to_string());
        drop(store);

        self.maybe_cleanup();
        Ok(())
    }

    pub fn cleanup(&self) {
        let mut store = self.seen.lock().unwrap();
        store.clear();
        *self.last_cleanup.lock().unwrap() = Instant::now();
    }

    fn maybe_cleanup(&self) {
        let now = Instant::now();
        let last = self.last_cleanup.lock().unwrap();
        if now.duration_since(*last) > self.cleanup_interval {
            drop(last);
            self.cleanup();
        }
    }
}
