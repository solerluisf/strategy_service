use async_trait::async_trait;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct StrategyHealthSnapshot {
    pub timestamp_ms: u64,
    pub overall_status: String,
    pub kill_switch_active: bool,
    pub operation_mode: String,
    pub strategy_enabled: bool,
    pub strategy_id: String,
    pub strategy_version: String,
    pub intents_generated_total: u64,
    pub intents_suppressed_by_threshold: u64,
    pub intents_suppressed_by_hysteresis: u64,
    pub intents_suppressed_by_cooldown: u64,
    pub intents_suppressed_by_regime: u64,
    pub intents_suppressed_by_position: u64,
    pub intents_suppressed_by_staleness: u64,
    pub per_symbol: HashMap<String, SymbolStrategyHealth>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SymbolStrategyHealth {
    pub last_intent_ms: u64,
    pub current_position_units: f64,
    pub current_regime: String,
    pub cooldown_active: bool,
    pub cooldown_remaining_ms: u64,
    pub hysteresis_direction: i8,
}

#[async_trait]
pub trait IHealthReporter: Send + Sync {
    async fn get_health(&self) -> StrategyHealthSnapshot;
}
