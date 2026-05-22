use crate::strategy_core::domain::position::PositionState;
use crate::strategy_core::domain::strategy_config::StrategyConfig;
use crate::strategy_core::domain::trade_intent::IntentSide;

pub struct PositionGate;

pub enum PositionGateResult {
    Allow,
    Suppress { reason: String },
}

impl PositionGate {
    pub fn check(
        position: &PositionState,
        side: &IntentSide,
        config: &StrategyConfig,
    ) -> PositionGateResult {
        match side {
            IntentSide::Long => {
                if position.net_units >= config.max_long_units {
                    return PositionGateResult::Suppress {
                        reason: format!(
                            "At max long units: {:.2} >= {:.2}",
                            position.net_units, config.max_long_units
                        ),
                    };
                }
                PositionGateResult::Allow
            }
            IntentSide::Short => {
                if !config.allow_short {
                    return PositionGateResult::Suppress {
                        reason: "Short selling is disabled".to_string(),
                    };
                }
                if position.net_units <= -config.max_short_units {
                    return PositionGateResult::Suppress {
                        reason: format!(
                            "At max short units: {:.2} <= -{:.2}",
                            position.net_units, config.max_short_units
                        ),
                    };
                }
                PositionGateResult::Allow
            }
            IntentSide::CloseLong => {
                if position.is_flat() || position.is_short() {
                    return PositionGateResult::Suppress {
                        reason: "No long position to close".to_string(),
                    };
                }
                PositionGateResult::Allow
            }
            IntentSide::CloseShort => {
                if position.is_flat() || position.is_long() {
                    return PositionGateResult::Suppress {
                        reason: "No short position to close".to_string(),
                    };
                }
                PositionGateResult::Allow
            }
            IntentSide::Flatten => {
                if position.is_flat() {
                    return PositionGateResult::Suppress {
                        reason: "Position is already flat".to_string(),
                    };
                }
                PositionGateResult::Allow
            }
        }
    }
}
