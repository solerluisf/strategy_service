use serde::{Deserialize, Serialize};

use crate::strategy_core::domain::inference_input::InferenceEvent;
use crate::strategy_core::domain::trade_intent::{IntentSide, IntentType};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StrategyType {
    MovingAverage,
    RSI,
    Arbitrage,
    Momentum,
}

impl std::fmt::Display for StrategyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StrategyType::MovingAverage => write!(f, "moving_average"),
            StrategyType::RSI => write!(f, "rsi"),
            StrategyType::Arbitrage => write!(f, "arbitrage"),
            StrategyType::Momentum => write!(f, "momentum"),
        }
    }
}

impl std::str::FromStr for StrategyType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "moving_average" | "MovingAverage" => Ok(StrategyType::MovingAverage),
            "rsi" | "RSI" => Ok(StrategyType::RSI),
            "arbitrage" | "Arbitrage" => Ok(StrategyType::Arbitrage),
            "momentum" | "Momentum" => Ok(StrategyType::Momentum),
            _ => Err(format!("Unknown strategy type: {}", s)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StrategySignal {
    pub side: IntentSide,
    pub intent_type: IntentType,
    pub strength: f64,
    pub metadata: Option<String>,
}

pub trait ITradingStrategy: Send + Sync {
    fn name(&self) -> StrategyType;
    fn version(&self) -> &str;
    fn evaluate(&self, event: &InferenceEvent) -> Option<StrategySignal>;
    fn clone_box(&self) -> Box<dyn ITradingStrategy>;
    fn on_regime_change(&mut self, _regime: &crate::strategy_core::domain::inference_input::RegimeLabel) {}
    fn reset_symbol_state(&mut self, _symbol: &str) {}
}

impl Clone for Box<dyn ITradingStrategy> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
