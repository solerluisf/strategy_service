use std::collections::HashMap;
use std::sync::RwLock;

use crate::strategy_core::domain::inference_input::InferenceEvent;
use crate::strategy_core::domain::trading_strategy::{ITradingStrategy, StrategySignal, StrategyType};
use crate::strategy_core::domain::trade_intent::{IntentSide, IntentType};

#[derive(Debug, Clone)]
pub struct MovingAverageConfig {
    pub fast_period: u32,
    pub slow_period: u32,
    pub signal_period: u32,
    pub entry_threshold: f64,
    pub exit_threshold: f64,
}

impl Default for MovingAverageConfig {
    fn default() -> Self {
        Self {
            fast_period: 12,
            slow_period: 26,
            signal_period: 9,
            entry_threshold: 0.0,
            exit_threshold: 0.0,
        }
    }
}

pub struct MovingAverageStrategy {
    config: MovingAverageConfig,
    price_history: RwLock<HashMap<String, Vec<f64>>>,
    macd_history: RwLock<HashMap<String, Vec<f64>>>,
}

impl MovingAverageStrategy {
    pub fn new(config: MovingAverageConfig) -> Self {
        Self {
            config,
            price_history: RwLock::new(HashMap::new()),
            macd_history: RwLock::new(HashMap::new()),
        }
    }

    fn compute_ema(prices: &[f64], period: u32) -> Option<f64> {
        if prices.len() < period as usize {
            return None;
        }
        let multiplier = 2.0 / (period as f64 + 1.0);
        let mut ema = prices[..period as usize].iter().sum::<f64>() / period as f64;
        for price in prices.iter().skip(period as usize) {
            ema = (price - ema) * multiplier + ema;
        }
        Some(ema)
    }

    fn compute_macd(&self, prices: &[f64]) -> Option<f64> {
        let fast_ema = Self::compute_ema(prices, self.config.fast_period)?;
        let slow_ema = Self::compute_ema(prices, self.config.slow_period)?;
        Some(fast_ema - slow_ema)
    }

    fn update_history(&self, symbol: &str, price: f64, macd: f64) {
        if let Ok(mut history) = self.price_history.write() {
            history.entry(symbol.to_string()).or_default().push(price);
            let max_len = (self.config.slow_period as usize) * 2;
            if let Some(h) = history.get_mut(symbol) {
                if h.len() > max_len {
                    h.drain(..h.len() - max_len);
                }
            }
        }
        if let Ok(mut history) = self.macd_history.write() {
            history.entry(symbol.to_string()).or_default().push(macd);
            let max_len = (self.config.signal_period as usize) * 2;
            if let Some(h) = history.get_mut(symbol) {
                if h.len() > max_len {
                    h.drain(..h.len() - max_len);
                }
            }
        }
    }
}

impl ITradingStrategy for MovingAverageStrategy {
    fn name(&self) -> StrategyType {
        StrategyType::MovingAverage
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn evaluate(&self, event: &InferenceEvent) -> Option<StrategySignal> {
        let price = event.outputs.raw.get("price").copied()?;
        self.update_history(&event.symbol, price, 0.0);

        let prices = self.price_history.read().ok()?.get(&event.symbol)?.clone();
        let macd = self.compute_macd(&prices)?;
        self.update_history(&event.symbol, price, macd);

        let macd_values = self.macd_history.read().ok()?.get(&event.symbol)?.clone();
        if macd_values.len() < 2 {
            return None;
        }

        let prev_macd = macd_values[macd_values.len() - 2];
        let signal = if macd > self.config.entry_threshold && prev_macd <= self.config.entry_threshold {
            Some(StrategySignal {
                side: IntentSide::Long,
                intent_type: IntentType::Entry,
                strength: macd.abs(),
                metadata: Some(format!("macd_cross_up: {:.4}", macd)),
            })
        } else if macd < self.config.exit_threshold && prev_macd >= self.config.exit_threshold {
            Some(StrategySignal {
                side: IntentSide::CloseLong,
                intent_type: IntentType::Exit,
                strength: macd.abs(),
                metadata: Some(format!("macd_cross_down: {:.4}", macd)),
            })
        } else {
            None
        };

        signal
    }

    fn clone_box(&self) -> Box<dyn ITradingStrategy> {
        Box::new(MovingAverageStrategy {
            config: self.config.clone(),
            price_history: RwLock::new(self.price_history.read().map(|h| h.clone()).unwrap_or_default()),
            macd_history: RwLock::new(self.macd_history.read().map(|h| h.clone()).unwrap_or_default()),
        })
    }

    fn reset_symbol_state(&mut self, symbol: &str) {
        if let Ok(mut history) = self.price_history.write() {
            history.remove(symbol);
        }
        if let Ok(mut history) = self.macd_history.write() {
            history.remove(symbol);
        }
    }
}
