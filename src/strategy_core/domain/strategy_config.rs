use serde::{Deserialize, Serialize};

use crate::strategy_core::domain::trading_strategy::StrategyType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    pub strategy_id: String,
    pub strategy_version: String,

    pub active_strategy: StrategyType,

    pub long_entry_threshold: f64,
    pub short_entry_threshold: f64,
    pub exit_threshold: f64,
    pub confidence_minimum: f64,

    pub hysteresis_band: f64,

    pub entry_cooldown_ms: u64,
    pub exit_cooldown_ms: u64,

    pub suppress_entries_in_volatile: bool,
    pub suppress_exits_in_ranging: bool,

    pub max_long_units: f64,
    pub max_short_units: f64,
    pub allow_short: bool,

    pub sizing_method: SizingMethod,
    pub base_size: f64,
    pub scale_by_confidence: bool,

    pub inference_staleness_ms: u64,

    pub intent_ttl_ms: Option<u64>,

    pub aggressive_threshold: f64,
    pub passive_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SizingMethod {
    Units,
    Notional,
    PortfolioPct,
    RiskBased,
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            strategy_id: "default".to_string(),
            strategy_version: "1.0.0".to_string(),
            active_strategy: StrategyType::Momentum,
            long_entry_threshold: 0.6,
            short_entry_threshold: -0.6,
            exit_threshold: 0.1,
            confidence_minimum: 0.5,
            hysteresis_band: 0.15,
            entry_cooldown_ms: 5000,
            exit_cooldown_ms: 2000,
            suppress_entries_in_volatile: true,
            suppress_exits_in_ranging: false,
            max_long_units: 1000.0,
            max_short_units: 1000.0,
            allow_short: false,
            sizing_method: SizingMethod::PortfolioPct,
            base_size: 0.02,
            scale_by_confidence: true,
            inference_staleness_ms: 2000,
            intent_ttl_ms: Some(30_000),
            aggressive_threshold: 0.85,
            passive_threshold: 0.65,
        }
    }
}

impl StrategyConfig {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read strategy config file '{}': {}", path, e))?;
        let config: StrategyConfig = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse strategy config: {}", e))?;
        Ok(config)
    }
}
