use async_trait::async_trait;

use crate::strategy_core::domain::errors::StrategyError;
use crate::strategy_core::domain::trade_intent::TradeIntent;

#[async_trait]
pub trait IJournalPort: Send + Sync {
    async fn append_intent(&self, intent: &TradeIntent) -> Result<(), StrategyError>;
    async fn append_control_event(&self, event_type: &str, data: &[u8]) -> Result<(), StrategyError>;
    fn get_last_control_event(&self, event_type: &str) -> Result<Option<Vec<u8>>, StrategyError>;
}
