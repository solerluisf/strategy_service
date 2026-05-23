use std::collections::HashMap;
use std::sync::RwLock;

use crate::strategy_core::domain::inference_input::InferenceEvent;
use crate::strategy_core::domain::trading_strategy::{ITradingStrategy, StrategySignal, StrategyType};
use crate::strategy_core::domain::trade_intent::{IntentSide, IntentType};

#[derive(Debug, Clone)]
pub struct MomentumConfig {
    pub lookback_period: u32,
    pub entry_threshold: f64,
    pub exit_threshold: f64,
    pub min_confidence: f64,
}

impl Default for MomentumConfig {
    fn default() -> Self {
        Self {
            lookback_period: 20,
            entry_threshold: 0.02,
            exit_threshold: 0.01,
            min_confidence: 0.6,
        }
    }
}

pub struct MomentumStrategy {
    config: MomentumConfig,
    price_history: RwLock<HashMap<String, Vec<f64>>>,
}

impl MomentumStrategy {
    pub fn new(config: MomentumConfig) -> Self {
        Self {
            config,
            price_history: RwLock::new(HashMap::new()),
        }
    }

    fn compute_momentum(prices: &[f64], period: u32) -> Option<f64> {
        if prices.len() < period as usize {
            return None;
        }
        let current = *prices.last()?;
        let past = prices[prices.len() - period as usize];
        if past == 0.0 {
            return None;
        }
        Some((current - past) / past)
    }
}

impl ITradingStrategy for MomentumStrategy {
    fn name(&self) -> StrategyType {
        StrategyType::Momentum
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn evaluate(&self, event: &InferenceEvent) -> Option<StrategySignal> {
        let confidence = event.outputs.confidence.unwrap_or(0.0);
        if confidence < self.config.min_confidence {
            return None;
        }

        let price = event.outputs.raw.get("price").copied()?;

        {
            let mut history = self.price_history.write().ok()?;
            history.entry(event.symbol.clone()).or_default().push(price);
            let max_len = (self.config.lookback_period as usize) * 2;
            if let Some(h) = history.get_mut(&event.symbol) {
                if h.len() > max_len {
                    h.drain(..h.len() - max_len);
                }
            }
        }

        let prices = self.price_history.read().ok()?.get(&event.symbol)?.clone();
        let momentum = Self::compute_momentum(&prices, self.config.lookback_period)?;

        let signal = if momentum > self.config.entry_threshold {
            Some(StrategySignal {
                side: IntentSide::Long,
                intent_type: IntentType::Entry,
                strength: momentum.abs() * confidence,
                metadata: Some(format!("momentum_positive: {:.4}", momentum)),
            })
        } else if momentum < -self.config.entry_threshold {
            Some(StrategySignal {
                side: IntentSide::CloseLong,
                intent_type: IntentType::Exit,
                strength: momentum.abs() * confidence,
                metadata: Some(format!("momentum_negative: {:.4}", momentum)),
            })
        } else {
            None
        };

        signal
    }

    fn clone_box(&self) -> Box<dyn ITradingStrategy> {
        Box::new(MomentumStrategy {
            config: self.config.clone(),
            price_history: RwLock::new(self.price_history.read().map(|h| h.clone()).unwrap_or_default()),
        })
    }

    fn reset_symbol_state(&mut self, symbol: &str) {
        if let Ok(mut history) = self.price_history.write() {
            history.remove(symbol);
        }
    }
}
