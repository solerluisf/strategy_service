use crate::strategy_core::domain::inference_input::ModelOutputs;
use crate::strategy_core::domain::position::PositionState;
use crate::strategy_core::domain::strategy_config::{SizingMethod, StrategyConfig};
use crate::strategy_core::domain::trade_intent::SizeHint;

pub struct SizingEngine;

impl SizingEngine {
    pub fn compute(
        outputs: &ModelOutputs,
        config: &StrategyConfig,
        _position: &PositionState,
    ) -> SizeHint {
        let confidence_scale = if config.scale_by_confidence {
            outputs.confidence.unwrap_or(1.0)
        } else {
            1.0
        };

        let scaled_size = config.base_size * confidence_scale;

        match config.sizing_method {
            SizingMethod::Units => SizeHint::Units(scaled_size),
            SizingMethod::Notional => SizeHint::Notional(scaled_size),
            SizingMethod::PortfolioPct => SizeHint::PortfolioPct(scaled_size),
            SizingMethod::RiskBased => {
                let atr = outputs.raw.get("atr_14").copied().unwrap_or(1.0);
                let risk_units = (scaled_size / atr).max(1.0);
                SizeHint::RiskBased(risk_units)
            }
        }
    }
}
