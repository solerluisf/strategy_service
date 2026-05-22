use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

pub struct TelemetryDecorator {
    call_count: AtomicU64,
    error_count: AtomicU64,
    total_latency_us: AtomicU64,
}

impl TelemetryDecorator {
    pub fn new() -> Self {
        Self {
            call_count: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
            total_latency_us: AtomicU64::new(0),
        }
    }

    pub fn record_call(&self) -> Instant {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        Instant::now()
    }

    pub fn record_success(&self, start: Instant) {
        let elapsed_us = start.elapsed().as_micros() as u64;
        self.total_latency_us.fetch_add(elapsed_us, Ordering::SeqCst);
    }

    pub fn record_error(&self, start: Instant) {
        self.error_count.fetch_add(1, Ordering::SeqCst);
        self.record_success(start);
    }

    pub fn call_count(&self) -> u64 {
        self.call_count.load(Ordering::SeqCst)
    }

    pub fn error_count(&self) -> u64 {
        self.error_count.load(Ordering::SeqCst)
    }

    pub fn avg_latency_us(&self) -> f64 {
        let count = self.call_count.load(Ordering::SeqCst);
        if count == 0 {
            return 0.0;
        }
        self.total_latency_us.load(Ordering::SeqCst) as f64 / count as f64
    }
}

impl Default for TelemetryDecorator {
    fn default() -> Self {
        Self::new()
    }
}
