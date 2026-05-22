use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

pub struct CircuitBreaker {
    failure_threshold: u64,
    reset_timeout: Duration,
    failures: AtomicU64,
    state: AtomicU64, // 0=Closed, 1=Open, 2=HalfOpen
    last_failure: AtomicU64, // nanoseconds
    last_state_change: AtomicU64, // nanoseconds
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u64, reset_timeout: Duration) -> Self {
        Self {
            failure_threshold,
            reset_timeout,
            failures: AtomicU64::new(0),
            state: AtomicU64::new(0), // Closed
            last_failure: AtomicU64::new(0),
            last_state_change: AtomicU64::new(current_time_ns()),
        }
    }

    pub fn record_success(&self) {
        self.failures.store(0, Ordering::SeqCst);
        self.set_state(CircuitState::Closed);
    }

    pub fn record_failure(&self) {
        let failures = self.failures.fetch_add(1, Ordering::SeqCst) + 1;
        self.last_failure.store(current_time_ns(), Ordering::SeqCst);

        if failures >= self.failure_threshold {
            self.set_state(CircuitState::Open);
        }
    }

    pub fn allow_request(&self) -> bool {
        match self.get_state() {
            CircuitState::Closed => true,
            CircuitState::Open => {
                let now = current_time_ns();
                let last_change = self.last_state_change.load(Ordering::SeqCst);
                if now.saturating_sub(last_change) >= self.reset_timeout.as_nanos() as u64 {
                    self.set_state(CircuitState::HalfOpen);
                    true
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    pub fn get_state(&self) -> CircuitState {
        match self.state.load(Ordering::SeqCst) {
            0 => CircuitState::Closed,
            1 => CircuitState::Open,
            2 => CircuitState::HalfOpen,
            _ => CircuitState::Closed,
        }
    }

    fn set_state(&self, state: CircuitState) {
        let numeric = match state {
            CircuitState::Closed => 0,
            CircuitState::Open => 1,
            CircuitState::HalfOpen => 2,
        };
        self.state.store(numeric, Ordering::SeqCst);
        self.last_state_change.store(current_time_ns(), Ordering::SeqCst);
    }
}

fn current_time_ns() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

pub type SharedCircuitBreaker = Arc<CircuitBreaker>;
