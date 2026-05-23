use std::collections::HashMap;
use std::sync::Arc;

use crate::strategy_core::domain::inference_input::{InferenceEvent, RegimeLabel};
use crate::strategy_core::domain::trading_strategy::{ITradingStrategy, StrategySignal, StrategyType};
use crate::strategy_core::domain::strategy_config::StrategyConfig;

pub struct StrategySelector {
    strategies: HashMap<StrategyType, Box<dyn ITradingStrategy>>,
    active_strategy: StrategyType,
    regime_strategy_map: HashMap<RegimeLabel, StrategyType>,
    config: Arc<std::sync::RwLock<StrategyConfig>>,
}

impl StrategySelector {
    pub fn new(
        strategies: HashMap<StrategyType, Box<dyn ITradingStrategy>>,
        default_strategy: StrategyType,
        config: Arc<std::sync::RwLock<StrategyConfig>>,
    ) -> Result<Self, String> {
        if !strategies.contains_key(&default_strategy) {
            return Err(format!(
                "Default strategy {:?} not registered",
                default_strategy
            ));
        }

        let mut regime_map = HashMap::new();
        regime_map.insert(RegimeLabel::Trending, StrategyType::Momentum);
        regime_map.insert(RegimeLabel::Ranging, StrategyType::RSI);
        regime_map.insert(RegimeLabel::Volatile, StrategyType::Arbitrage);
        regime_map.insert(RegimeLabel::Unknown, default_strategy);

        Ok(Self {
            strategies,
            active_strategy: default_strategy,
            regime_strategy_map: regime_map,
            config,
        })
    }

    pub fn register_strategy(&mut self, strategy: Box<dyn ITradingStrategy>) -> Result<(), String> {
        let name = strategy.name();
        if self.strategies.contains_key(&name) {
            return Err(format!("Strategy {:?} already registered", name));
        }
        self.strategies.insert(name, strategy);
        Ok(())
    }

    pub fn set_active_strategy(&mut self, strategy_type: StrategyType) -> Result<(), String> {
        if !self.strategies.contains_key(&strategy_type) {
            return Err(format!("Strategy {:?} not registered", strategy_type));
        }
        tracing::info!(
            from = %self.active_strategy,
            to = %strategy_type,
            "Strategy switched at runtime"
        );
        self.active_strategy = strategy_type;
        Ok(())
    }

    pub fn set_regime_strategy(&mut self, regime: RegimeLabel, strategy_type: StrategyType) -> Result<(), String> {
        if !self.strategies.contains_key(&strategy_type) {
            return Err(format!("Strategy {:?} not registered", strategy_type));
        }
        self.regime_strategy_map.insert(regime, strategy_type);
        Ok(())
    }

    pub fn select_by_regime(&mut self, regime: &RegimeLabel) -> Result<(), String> {
        if let Some(strategy_type) = self.regime_strategy_map.get(regime) {
            let target = *strategy_type;
            if target != self.active_strategy {
                self.set_active_strategy(target)?;
            }
        }
        Ok(())
    }

    pub fn evaluate(&self, event: &InferenceEvent) -> Option<StrategySignal> {
        self.strategies
            .get(&self.active_strategy)
            .and_then(|s| s.evaluate(event))
    }

    pub fn active_strategy(&self) -> StrategyType {
        self.active_strategy
    }

    pub fn active_strategy_ref(&self) -> Option<&dyn ITradingStrategy> {
        self.strategies.get(&self.active_strategy).map(|s| s.as_ref())
    }

    pub fn available_strategies(&self) -> Vec<StrategyType> {
        self.strategies.keys().cloned().collect()
    }

    pub fn get_strategy_info(&self) -> HashMap<String, String> {
        let mut info = HashMap::new();
        info.insert("active".to_string(), self.active_strategy.to_string());
        info.insert(
            "available".to_string(),
            self.strategies
                .keys()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(","),
        );
        for (regime, strategy) in &self.regime_strategy_map {
            let regime_str = match regime {
                RegimeLabel::Trending => "trending",
                RegimeLabel::Ranging => "ranging",
                RegimeLabel::Volatile => "volatile",
                RegimeLabel::Unknown => "unknown",
            };
            info.insert(
                format!("regime_{}", regime_str),
                strategy.to_string(),
            );
        }
        info
    }

    pub fn reset_symbol_state(&mut self, symbol: &str) {
        for strategy in self.strategies.values_mut() {
            strategy.reset_symbol_state(symbol);
        }
    }
}
