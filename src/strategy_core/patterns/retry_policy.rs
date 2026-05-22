use std::time::Duration;

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
}

impl RetryPolicy {
    pub fn new(max_retries: u32, base_delay: Duration, max_delay: Duration) -> Self {
        Self {
            max_retries,
            base_delay,
            max_delay,
            backoff_multiplier: 2.0,
        }
    }

    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let multiplier = self.backoff_multiplier.powi(attempt as i32);
        let delay = self.base_delay.mul_f64(multiplier);
        delay.min(self.max_delay)
    }

    pub fn should_retry(&self, attempt: u32) -> bool {
        attempt < self.max_retries
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::new(
            3,
            Duration::from_millis(100),
            Duration::from_secs(5),
        )
    }
}
