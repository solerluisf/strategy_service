use async_trait::async_trait;

use crate::strategy_core::domain::errors::StrategyError;
use crate::strategy_core::domain::trade_intent::TradeIntent;

#[async_trait]
pub trait IIntentPublisher: Send + Sync {
    async fn publish(&self, intent: &TradeIntent) -> Result<(), StrategyError>;
}
