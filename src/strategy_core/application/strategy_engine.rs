use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::strategy_core::application::idempotency::IdempotencyStore;
use crate::strategy_core::application::kill_switch::KillSwitch;
use crate::strategy_core::application::mode_controller::ModeController;
use crate::strategy_core::application::rules::cooldown_tracker::{CooldownResult, CooldownTracker};
use crate::strategy_core::application::rules::hysteresis_filter::{HysteresisFilter, HysteresisResult};
use crate::strategy_core::application::rules::position_gate::{PositionGate, PositionGateResult};
use crate::strategy_core::application::rules::regime_gate::RegimeGate;
use crate::strategy_core::application::rules::sizing_engine::SizingEngine;
use crate::strategy_core::application::rules::staleness_guard::{StalenessGuard, StalenessResult};
use crate::strategy_core::application::rules::threshold_gate::{ThresholdDecision, ThresholdGate};
use crate::strategy_core::application::strategy_selector::StrategySelector;
use crate::strategy_core::domain::errors::StrategyError;
use crate::strategy_core::domain::inference_input::InferenceEvent;
use crate::strategy_core::domain::position::PositionCache;
use crate::strategy_core::domain::strategy_config::StrategyConfig;
use crate::strategy_core::domain::trade_intent::{IntentSide, IntentType, IntentUrgency, TradeIntent};
use crate::strategy_core::domain::trading_strategy::StrategyType;
use crate::strategy_core::ports::metrics_port::IMetricsPort;

pub struct StrategyEngine {
    config: Arc<std::sync::RwLock<StrategyConfig>>,
    selector: StrategySelector,
    hysteresis: HysteresisFilter,
    cooldown: CooldownTracker,
    staleness_guard: StalenessGuard,
    position_cache: Arc<PositionCache>,
    idempotency: IdempotencyStore,
    kill_switch: Arc<KillSwitch>,
    mode_controller: Arc<ModeController>,
    paused_symbols: HashSet<String>,
    per_symbol_seq: HashMap<String, u64>,
    metrics: Arc<dyn IMetricsPort>,
}

impl StrategyEngine {
    pub fn new(
        config: StrategyConfig,
        selector: StrategySelector,
        position_cache: Arc<PositionCache>,
        kill_switch: Arc<KillSwitch>,
        mode_controller: Arc<ModeController>,
        metrics: Arc<dyn IMetricsPort>,
    ) -> Self {
        let staleness_guard = StalenessGuard::new(config.inference_staleness_ms);
        Self {
            config: Arc::new(std::sync::RwLock::new(config)),
            selector,
            hysteresis: HysteresisFilter::new(),
            cooldown: CooldownTracker::new(),
            staleness_guard,
            position_cache,
            idempotency: IdempotencyStore::new(60),
            kill_switch,
            mode_controller,
            paused_symbols: HashSet::new(),
            per_symbol_seq: HashMap::new(),
            metrics,
        }
    }

    pub fn evaluate(
        &mut self,
        event: InferenceEvent,
        now_ns: u64,
    ) -> Result<Option<TradeIntent>, StrategyError> {
        if self.kill_switch.is_active() {
            return Ok(None);
        }
        if self.paused_symbols.contains(&event.symbol) {
            return Ok(None);
        }
        let mode = self.mode_controller.get();
        if mode == crate::strategy_core::domain::operation_mode::OperationMode::Readonly
            || mode == crate::strategy_core::domain::operation_mode::OperationMode::Offline
        {
            return Ok(None);
        }

        match self.staleness_guard.check(event.inferred_ns, now_ns) {
            StalenessResult::Stale { age_ms } => {
                tracing::debug!(
                    symbol = %event.symbol, age_ms = age_ms,
                    "Inference stale, suppressing"
                );
                self.metrics.increment_counter(
                    "strategy_suppressed_by_staleness",
                    &[("symbol", &event.symbol)],
                );
                return Ok(None);
            }
            StalenessResult::Fresh => {}
        }

        self.selector.select_by_regime(&event.regime).ok();

        let strategy_signal = self.selector.evaluate(&event);

        let config = self.config.read().unwrap();

        let (side, intent_type, forecast) = match strategy_signal {
            Some(signal) => {
                let forecast = signal.strength;
                let confidence = event.outputs.confidence.unwrap_or(0.0);
                if confidence < config.confidence_minimum {
                    self.metrics.increment_counter(
                        "strategy_suppressed_by_threshold",
                        &[("symbol", &event.symbol)],
                    );
                    return Ok(None);
                }
                (signal.side, signal.intent_type, forecast)
            }
            None => {
                let forecast = event.outputs.forecast.unwrap_or(0.0);
                let confidence = event.outputs.confidence.unwrap_or(0.0);
                let decision = ThresholdGate::evaluate(forecast, confidence, &config);
                if matches!(decision, ThresholdDecision::NoSignal) {
                    self.metrics.increment_counter(
                        "strategy_suppressed_by_threshold",
                        &[("symbol", &event.symbol)],
                    );
                    return Ok(None);
                }
                let hyst_result = self.hysteresis.check(&event.symbol, decision, forecast, &config);
                match hyst_result {
                    HysteresisResult::Suppress => {
                        self.metrics.increment_counter(
                            "strategy_suppressed_by_hysteresis",
                            &[("symbol", &event.symbol)],
                        );
                        return Ok(None);
                    }
                    HysteresisResult::Allow(_) => {}
                }
                let position = self.position_cache.get(&event.symbol);
                let (side, intent_type) = decision_to_side_and_type(&hyst_result, &position, &config);
                (side, intent_type, forecast)
            }
        };

        let position = self.position_cache.get(&event.symbol);
        let position_result = PositionGate::check(&position, &side, &config);
        match position_result {
            PositionGateResult::Suppress { reason } => {
                tracing::debug!(symbol = %event.symbol, reason = %reason, "Position gate suppressed");
                self.metrics.increment_counter(
                    "strategy_suppressed_by_position",
                    &[("symbol", &event.symbol)],
                );
                return Ok(None);
            }
            PositionGateResult::Allow => {}
        }

        let regime_allowed = RegimeGate::allows(&event.regime, &intent_type, &config);
        if !regime_allowed {
            tracing::debug!(
                symbol = %event.symbol, regime = ?event.regime, intent_type = ?intent_type,
                "Regime gate suppressed"
            );
            self.metrics.increment_counter(
                "strategy_suppressed_by_regime",
                &[("symbol", &event.symbol), ("regime", regime_label_str(&event.regime))],
            );
            return Ok(None);
        }

        let cooldown_result = self.cooldown.check(&event.symbol, &intent_type, now_ns, &config);
        match cooldown_result {
            CooldownResult::Suppressed { remaining_ms } => {
                tracing::debug!(
                    symbol = %event.symbol, remaining_ms = remaining_ms,
                    "Cooldown suppressed"
                );
                self.metrics.increment_counter(
                    "strategy_suppressed_by_cooldown",
                    &[("symbol", &event.symbol)],
                );
                return Ok(None);
            }
            CooldownResult::Ready => {}
        }

        let size_hint = SizingEngine::compute(&event.outputs, &config, &position);
        let urgency = compute_urgency(forecast.abs(), &config);

        let seq = self.per_symbol_seq.entry(event.symbol.clone()).or_insert(0);
        let current_seq = *seq;
        *seq += 1;

        let expires_ns = config.intent_ttl_ms.map(|ttl| now_ns + ttl * 1_000_000);

        let intent = TradeIntent::new(
            &event,
            side,
            size_hint,
            intent_type,
            urgency,
            &config.strategy_id,
            &config.strategy_version,
            current_seq,
            expires_ns,
        );

        drop(config);

        self.idempotency.mark_seen(&intent.intent_id)?;
        self.cooldown.mark_emitted(&event.symbol, &intent_type, now_ns);

        self.metrics.increment_counter(
            "strategy_intents_generated",
            &[("symbol", &intent.symbol), ("side", intent_side_str(&side))],
        );

        self.metrics.increment_counter(
            "strategy_active",
            &[("strategy", &self.selector.active_strategy().to_string())],
        );

        Ok(Some(intent))
    }

    pub fn reload_config(&self, new_config: StrategyConfig) {
        let mut config = self.config.write().unwrap();
        *config = new_config;
        self.metrics.increment_counter("strategy_config_reloads_total", &[]);
    }

    pub fn switch_strategy(&mut self, strategy_type: StrategyType) -> Result<(), String> {
        self.selector.set_active_strategy(strategy_type)?;
        self.metrics.increment_counter(
            "strategy_switches_total",
            &[("strategy", &strategy_type.to_string())],
        );
        Ok(())
    }

    pub fn active_strategy(&self) -> StrategyType {
        self.selector.active_strategy()
    }

    pub fn available_strategies(&self) -> Vec<StrategyType> {
        self.selector.available_strategies()
    }

    pub fn get_strategy_info(&self) -> HashMap<String, String> {
        self.selector.get_strategy_info()
    }

    pub fn reset_symbol_state(&mut self, symbol: &str) {
        self.hysteresis.reset(symbol);
        self.cooldown.reset(symbol);
        self.selector.reset_symbol_state(symbol);
    }

    pub fn force_flatten(&mut self, symbol: &str, now_ns: u64) -> Option<TradeIntent> {
        let config = self.config.read().unwrap();
        let position = self.position_cache.get(symbol);

        if position.is_flat() {
            return None;
        }

        let side = if position.is_long() {
            IntentSide::CloseLong
        } else {
            IntentSide::CloseShort
        };

        let size_hint = crate::strategy_core::domain::trade_intent::SizeHint::Units(position.net_units.abs());
        let seq = self.per_symbol_seq.entry(symbol.to_string()).or_insert(0);
        let current_seq = *seq;
        *seq += 1;

        let expires_ns = config.intent_ttl_ms.map(|ttl| now_ns + ttl * 1_000_000);

        let inference = InferenceEvent {
            event_id: format!("flatten-{}", symbol),
            symbol: symbol.to_string(),
            timestamp_ns: now_ns,
            inferred_ns: now_ns,
            sequence_number: current_seq,
            trace_id: "flatten".to_string(),
            model_id: "flatten".to_string(),
            model_version: "1.0.0".to_string(),
            feature_version: "1.0.0".to_string(),
            regime: crate::strategy_core::domain::inference_input::RegimeLabel::Unknown,
            outputs: crate::strategy_core::domain::inference_input::ModelOutputs {
                forecast: Some(0.0),
                confidence: Some(1.0),
                action_score: None,
                regime_label: None,
                regime_strength: None,
                raw: std::collections::HashMap::new(),
            },
            latency_us: 0,
        };

        let intent = TradeIntent::new(
            &inference,
            side,
            size_hint,
            IntentType::Exit,
            IntentUrgency::Aggressive,
            &config.strategy_id,
            &config.strategy_version,
            current_seq,
            expires_ns,
        );

        Some(intent)
    }

    pub fn pause_symbol(&mut self, symbol: &str) {
        self.paused_symbols.insert(symbol.to_string());
    }

    pub fn resume_symbol(&mut self, symbol: &str) {
        self.paused_symbols.remove(symbol);
    }

    pub fn get_hysteresis_direction(&self, symbol: &str) -> i8 {
        self.hysteresis.get_direction(symbol)
    }
}

fn decision_to_side_and_type(
    hyst_result: &HysteresisResult,
    position: &crate::strategy_core::domain::position::PositionState,
    config: &StrategyConfig,
) -> (IntentSide, IntentType) {
    let decision = match hyst_result {
        HysteresisResult::Allow(d) => d,
        HysteresisResult::Suppress => {
            return (IntentSide::Long, IntentType::Entry);
        }
    };

    match (decision, position) {
        (ThresholdDecision::Long, p) if p.is_flat() => (IntentSide::Long, IntentType::Entry),
        (ThresholdDecision::Long, p) if p.is_long() => (IntentSide::Long, IntentType::ScaleIn),
        (ThresholdDecision::Long, p) if p.is_short() => (IntentSide::CloseShort, IntentType::Exit),
        (ThresholdDecision::Short, p) if p.is_flat() => {
            if config.allow_short {
                (IntentSide::Short, IntentType::Entry)
            } else {
                (IntentSide::Long, IntentType::Entry)
            }
        }
        (ThresholdDecision::Short, p) if p.is_short() => (IntentSide::Short, IntentType::ScaleIn),
        (ThresholdDecision::Short, p) if p.is_long() => (IntentSide::CloseLong, IntentType::Exit),
        (ThresholdDecision::Exit, p) if p.is_long() => (IntentSide::CloseLong, IntentType::Exit),
        (ThresholdDecision::Exit, p) if p.is_short() => (IntentSide::CloseShort, IntentType::Exit),
        (ThresholdDecision::Exit, _) => (IntentSide::Long, IntentType::Entry),
        _ => (IntentSide::Long, IntentType::Entry),
    }
}

fn compute_urgency(forecast_abs: f64, config: &StrategyConfig) -> IntentUrgency {
    if forecast_abs >= config.aggressive_threshold {
        IntentUrgency::Aggressive
    } else if forecast_abs >= config.passive_threshold {
        IntentUrgency::Normal
    } else {
        IntentUrgency::Passive
    }
}

fn intent_side_str(side: &IntentSide) -> &'static str {
    match side {
        IntentSide::Long => "long",
        IntentSide::Short => "short",
        IntentSide::CloseLong => "close_long",
        IntentSide::CloseShort => "close_short",
        IntentSide::Flatten => "flatten",
    }
}

fn regime_label_str(regime: &crate::strategy_core::domain::inference_input::RegimeLabel) -> &'static str {
    match regime {
        crate::strategy_core::domain::inference_input::RegimeLabel::Ranging => "ranging",
        crate::strategy_core::domain::inference_input::RegimeLabel::Trending => "trending",
        crate::strategy_core::domain::inference_input::RegimeLabel::Volatile => "volatile",
        crate::strategy_core::domain::inference_input::RegimeLabel::Unknown => "unknown",
    }
}
