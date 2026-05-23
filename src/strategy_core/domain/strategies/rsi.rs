use std::collections::HashMap;
use std::sync::RwLock;

use crate::strategy_core::domain::inference_input::InferenceEvent;
use crate::strategy_core::domain::trading_strategy::{ITradingStrategy, StrategySignal, StrategyType};
use crate::strategy_core::domain::trade_intent::{IntentSide, IntentType};

#[derive(Debug, Clone)]
pub struct RSIConfig {
    pub period: u32,
    pub overbought_threshold: f64,
    pub oversold_threshold: f64,
    pub exit_midpoint: f64,
}

impl Default for RSIConfig {
    fn default() -> Self {
        Self {
            period: 14,
            overbought_threshold: 70.0,
            oversold_threshold: 30.0,
            exit_midpoint: 50.0,
        }
    }
}

pub struct RSIStrategy {
    config: RSIConfig,
    price_history: RwLock<HashMap<String, Vec<f64>>>,
}

impl RSIStrategy {
    pub fn new(config: RSIConfig) -> Self {
        Self {
            config,
            price_history: RwLock::new(HashMap::new()),
        }
    }

    fn compute_rsi(prices: &[f64], period: u32) -> Option<f64> {
        if prices.len() < period as usize + 1 {
            return None;
        }

        let mut gains = 0.0;
        let mut losses = 0.0;

        for i in 1..=period as usize {
            let diff = prices[i] - prices[i - 1];
            if diff > 0.0 {
                gains += diff;
            } else {
                losses -= diff;
            }
        }

        let avg_gain = gains / period as f64;
        let avg_loss = losses / period as f64;

        if avg_loss == 0.0 {
            return Some(100.0);
        }

        let rs = avg_gain / avg_loss;
        Some(100.0 - (100.0 / (1.0 + rs)))
    }
}

impl ITradingStrategy for RSIStrategy {
    fn name(&self) -> StrategyType {
        StrategyType::RSI
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn evaluate(&self, event: &InferenceEvent) -> Option<StrategySignal> {
        let price = event.outputs.raw.get("price").copied()?;

        {
            let mut history = self.price_history.write().ok()?;
            history.entry(event.symbol.clone()).or_default().push(price);
            let max_len = (self.config.period as usize) * 2;
            if let Some(h) = history.get_mut(&event.symbol) {
                if h.len() > max_len {
                    h.drain(..h.len() - max_len);
                }
            }
        }

        let prices = self.price_history.read().ok()?.get(&event.symbol)?.clone();
        let rsi = Self::compute_rsi(&prices, self.config.period)?;

        let signal = if rsi <= self.config.oversold_threshold {
            Some(StrategySignal {
                side: IntentSide::Long,
                intent_type: IntentType::Entry,
                strength: (self.config.oversold_threshold - rsi) / self.config.oversold_threshold,
                metadata: Some(format!("rsi_oversold: {:.2}", rsi)),
            })
        } else if rsi >= self.config.overbought_threshold {
            Some(StrategySignal {
                side: IntentSide::CloseLong,
                intent_type: IntentType::Exit,
                strength: (rsi - self.config.overbought_threshold) / (100.0 - self.config.overbought_threshold),
                metadata: Some(format!("rsi_overbought: {:.2}", rsi)),
            })
        } else {
            None
        };

        signal
    }

    fn clone_box(&self) -> Box<dyn ITradingStrategy> {
        Box::new(RSIStrategy {
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
