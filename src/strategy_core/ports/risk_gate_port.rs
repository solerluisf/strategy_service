use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::strategy_core::domain::errors::StrategyError;
use crate::strategy_core::domain::trade_intent::TradeIntent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskCheckRequest {
    pub intent: TradeIntent,
    pub request_id: String,
    pub timestamp_ns: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskDecision {
    pub request_id: String,
    pub approved: bool,
    pub reason: Option<String>,
}

#[async_trait]
pub trait IRiskGateClient: Send + Sync {
    async fn check_risk(&self, intent: TradeIntent) -> Result<RiskDecision, StrategyError>;
}
