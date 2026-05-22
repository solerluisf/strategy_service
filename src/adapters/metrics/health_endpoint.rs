use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;

use crate::strategy_core::application::kill_switch::KillSwitch;
use crate::strategy_core::application::mode_controller::ModeController;
use crate::strategy_core::domain::position::PositionCache;
use crate::strategy_core::ports::health_port::{IHealthReporter, StrategyHealthSnapshot, SymbolStrategyHealth};

pub struct HealthReporter {
    kill_switch: Arc<KillSwitch>,
    mode_controller: Arc<ModeController>,
    position_cache: Arc<PositionCache>,
    intents_generated_total: u64,
    intents_suppressed_by_threshold: u64,
    intents_suppressed_by_hysteresis: u64,
    intents_suppressed_by_cooldown: u64,
    intents_suppressed_by_regime: u64,
    intents_suppressed_by_position: u64,
    intents_suppressed_by_staleness: u64,
}

impl HealthReporter {
    pub fn new(
        kill_switch: Arc<KillSwitch>,
        mode_controller: Arc<ModeController>,
        position_cache: Arc<PositionCache>,
    ) -> Self {
        Self {
            kill_switch,
            mode_controller,
            position_cache,
            intents_generated_total: 0,
            intents_suppressed_by_threshold: 0,
            intents_suppressed_by_hysteresis: 0,
            intents_suppressed_by_cooldown: 0,
            intents_suppressed_by_regime: 0,
            intents_suppressed_by_position: 0,
            intents_suppressed_by_staleness: 0,
        }
    }

    pub fn increment_generated(&mut self) {
        self.intents_generated_total += 1;
    }

    pub fn increment_suppressed_threshold(&mut self) {
        self.intents_suppressed_by_threshold += 1;
    }

    pub fn increment_suppressed_hysteresis(&mut self) {
        self.intents_suppressed_by_hysteresis += 1;
    }

    pub fn increment_suppressed_cooldown(&mut self) {
        self.intents_suppressed_by_cooldown += 1;
    }

    pub fn increment_suppressed_regime(&mut self) {
        self.intents_suppressed_by_regime += 1;
    }

    pub fn increment_suppressed_position(&mut self) {
        self.intents_suppressed_by_position += 1;
    }

    pub fn increment_suppressed_staleness(&mut self) {
        self.intents_suppressed_by_staleness += 1;
    }
}

#[async_trait]
impl IHealthReporter for HealthReporter {
    async fn get_health(&self) -> StrategyHealthSnapshot {
        let positions = self.position_cache.get_all();
        let mut per_symbol = HashMap::new();

        for pos in positions {
            per_symbol.insert(
                pos.symbol.clone(),
                SymbolStrategyHealth {
                    last_intent_ms: 0,
                    current_position_units: pos.net_units,
                    current_regime: "unknown".to_string(),
                    cooldown_active: false,
                    cooldown_remaining_ms: 0,
                    hysteresis_direction: 0,
                },
            );
        }

        let mode = self.mode_controller.get();
        let overall_status = if self.kill_switch.is_active() {
            "stopped".to_string()
        } else {
            "running".to_string()
        };

        StrategyHealthSnapshot {
            timestamp_ms: current_time_ms(),
            overall_status,
            kill_switch_active: self.kill_switch.is_active(),
            operation_mode: mode.to_string(),
            strategy_enabled: mode != crate::strategy_core::domain::operation_mode::OperationMode::Offline,
            strategy_id: "strategy_service".to_string(),
            strategy_version: "1.0.0".to_string(),
            intents_generated_total: self.intents_generated_total,
            intents_suppressed_by_threshold: self.intents_suppressed_by_threshold,
            intents_suppressed_by_hysteresis: self.intents_suppressed_by_hysteresis,
            intents_suppressed_by_cooldown: self.intents_suppressed_by_cooldown,
            intents_suppressed_by_regime: self.intents_suppressed_by_regime,
            intents_suppressed_by_position: self.intents_suppressed_by_position,
            intents_suppressed_by_staleness: self.intents_suppressed_by_staleness,
            per_symbol,
        }
    }
}

fn current_time_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
