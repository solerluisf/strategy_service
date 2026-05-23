use std::collections::HashMap;
use std::sync::RwLock;

use crate::strategy_core::domain::inference_input::InferenceEvent;
use crate::strategy_core::domain::trading_strategy::{ITradingStrategy, StrategySignal, StrategyType};
use crate::strategy_core::domain::trade_intent::{IntentSide, IntentType};

#[derive(Debug, Clone)]
pub struct ArbitrageConfig {
    pub spread_threshold: f64,
    pub min_profit_pct: f64,
    pub max_position_size: f64,
}

impl Default for ArbitrageConfig {
    fn default() -> Self {
        Self {
            spread_threshold: 0.001,
            min_profit_pct: 0.0005,
            max_position_size: 1000.0,
        }
    }
}

pub struct ArbitrageStrategy {
    config: ArbitrageConfig,
    last_prices: RwLock<HashMap<String, f64>>,
    reference_prices: RwLock<HashMap<String, f64>>,
}

impl ArbitrageStrategy {
    pub fn new(config: ArbitrageConfig) -> Self {
        Self {
            config,
            last_prices: RwLock::new(HashMap::new()),
            reference_prices: RwLock::new(HashMap::new()),
        }
    }

    pub fn set_reference_price(&self, symbol: &str, price: f64) {
        if let Ok(mut prices) = self.reference_prices.write() {
            prices.insert(symbol.to_string(), price);
        }
    }
}

impl ITradingStrategy for ArbitrageStrategy {
    fn name(&self) -> StrategyType {
        StrategyType::Arbitrage
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn evaluate(&self, event: &InferenceEvent) -> Option<StrategySignal> {
        let current_price = event.outputs.raw.get("price").copied()?;
        let reference_price = self.reference_prices.read().ok()?.get(&event.symbol).copied()?;

        {
            let mut prices = self.last_prices.write().ok()?;
            prices.insert(event.symbol.clone(), current_price);
        }

        let spread = (current_price - reference_price) / reference_price;
        let abs_spread = spread.abs();

        if abs_spread < self.config.spread_threshold {
            return None;
        }

        let profit_pct = abs_spread - (self.config.min_profit_pct * 2.0);
        if profit_pct <= 0.0 {
            return None;
        }

        let signal = if spread > 0.0 {
            StrategySignal {
                side: IntentSide::Short,
                intent_type: IntentType::Entry,
                strength: abs_spread,
                metadata: Some(format!("arb_spread_positive: {:.6}", spread)),
            }
        } else {
            StrategySignal {
                side: IntentSide::Long,
                intent_type: IntentType::Entry,
                strength: abs_spread,
                metadata: Some(format!("arb_spread_negative: {:.6}", spread)),
            }
        };

        Some(signal)
    }

    fn clone_box(&self) -> Box<dyn ITradingStrategy> {
        Box::new(ArbitrageStrategy {
            config: self.config.clone(),
            last_prices: RwLock::new(self.last_prices.read().map(|h| h.clone()).unwrap_or_default()),
            reference_prices: RwLock::new(self.reference_prices.read().map(|h| h.clone()).unwrap_or_default()),
        })
    }

    fn reset_symbol_state(&mut self, symbol: &str) {
        if let Ok(mut prices) = self.last_prices.write() {
            prices.remove(symbol);
        }
        if let Ok(mut prices) = self.reference_prices.write() {
            prices.remove(symbol);
        }
    }
}
