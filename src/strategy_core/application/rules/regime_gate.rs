use crate::strategy_core::domain::inference_input::RegimeLabel;
use crate::strategy_core::domain::strategy_config::StrategyConfig;
use crate::strategy_core::domain::trade_intent::IntentType;

pub struct RegimeGate;

impl RegimeGate {
    pub fn allows(
        regime: &RegimeLabel,
        intent_type: &IntentType,
        config: &StrategyConfig,
    ) -> bool {
        match (regime, intent_type) {
            (RegimeLabel::Volatile, IntentType::Entry)
            | (RegimeLabel::Volatile, IntentType::ScaleIn)
                if config.suppress_entries_in_volatile => false,
            _ => true,
        }
    }
}
