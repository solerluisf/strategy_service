use std::sync::Arc;

use crate::strategy_core::application::kill_switch::KillSwitch;
use crate::strategy_core::domain::errors::StrategyError;
use crate::strategy_core::domain::trade_intent::TradeIntent;
use crate::strategy_core::ports::journal_port::IJournalPort;

pub struct ReplayController {
    journal: Arc<dyn IJournalPort>,
    kill_switch: Arc<KillSwitch>,
}

impl ReplayController {
    pub fn new(journal: Arc<dyn IJournalPort>, kill_switch: Arc<KillSwitch>) -> Self {
        Self {
            journal,
            kill_switch,
        }
    }

    pub async fn replay(
        &self,
        _symbol_filter: Option<&str>,
    ) -> Result<Vec<TradeIntent>, StrategyError> {
        let mut intents: Vec<TradeIntent> = Vec::new();

        if self.kill_switch.is_active() {
            return Ok(intents);
        }

        let last_event = self.journal.get_last_control_event("trade_intent")?;
        if let Some(data) = last_event {
            let intent: TradeIntent = serde_json::from_slice(&data)
                .map_err(|e| StrategyError::Journal(format!("Replay deserialize: {}", e)))?;
            intents.push(intent);
        }

        Ok(intents)
    }
}
