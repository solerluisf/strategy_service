pub struct StalenessGuard {
    threshold_ms: u64,
}

pub enum StalenessResult {
    Fresh,
    Stale { age_ms: u64 },
}

impl StalenessGuard {
    pub fn new(threshold_ms: u64) -> Self {
        Self { threshold_ms }
    }

    pub fn check(&self, inferred_ns: u64, now_ns: u64) -> StalenessResult {
        let age_ns = now_ns.saturating_sub(inferred_ns);
        let age_ms = age_ns / 1_000_000;

        if age_ms > self.threshold_ms {
            StalenessResult::Stale { age_ms }
        } else {
            StalenessResult::Fresh
        }
    }
}
