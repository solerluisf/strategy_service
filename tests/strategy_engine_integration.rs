use std::collections::HashMap;
use std::sync::Arc;

use strategy_service::strategy_core::application::kill_switch::KillSwitch;
use strategy_service::strategy_core::application::mode_controller::ModeController;
use strategy_service::strategy_core::application::strategy_engine::StrategyEngine;
use strategy_service::strategy_core::domain::inference_input::{InferenceEvent, ModelOutputs, RegimeLabel};
use strategy_service::strategy_core::domain::operation_mode::OperationMode;
use strategy_service::strategy_core::domain::position::PositionCache;
use strategy_service::strategy_core::domain::strategy_config::StrategyConfig;
use strategy_service::adapters::metrics::metrics_adapter::MetricsAdapter;

fn make_inference(symbol: &str, forecast: f64, confidence: f64, inferred_ns: u64) -> InferenceEvent {
    InferenceEvent {
        event_id: format!("evt-{}", symbol),
        symbol: symbol.to_string(),
        timestamp_ns: 1_000_000_000_000u64,
        inferred_ns,
        sequence_number: 1,
        trace_id: "trace-1".to_string(),
        model_id: "model-v1".to_string(),
        model_version: "1.0.0".to_string(),
        feature_version: "1.0.0".to_string(),
        regime: RegimeLabel::Trending,
        outputs: ModelOutputs {
            forecast: Some(forecast),
            confidence: Some(confidence),
            action_score: None,
            regime_label: None,
            regime_strength: None,
            raw: HashMap::new(),
        },
        latency_us: 100,
    }
}

fn make_engine(config: StrategyConfig) -> StrategyEngine {
    let position_cache = Arc::new(PositionCache::new());
    let kill_switch = Arc::new(KillSwitch::new());
    let mode_controller = Arc::new(ModeController::new(OperationMode::Live));
    let metrics = Arc::new(MetricsAdapter::new());

    StrategyEngine::new(config, position_cache, kill_switch, mode_controller, metrics)
}

#[test]
fn valid_inference_emits_trade_intent() {
    let config = StrategyConfig::default();
    let mut engine = make_engine(config);
    let now_ns = 1_000_000_000_000u64;

    let event = make_inference("AAPL", 0.7, 0.8, now_ns);
    let result = engine.evaluate(event, now_ns);

    assert!(result.is_ok());
    assert!(result.unwrap().is_some());
}

#[test]
fn kill_switch_suppresses_intent() {
    let config = StrategyConfig::default();
    let position_cache = Arc::new(PositionCache::new());
    let kill_switch = Arc::new(KillSwitch::new());
    kill_switch.activate("test".to_string());
    let mode_controller = Arc::new(ModeController::new(OperationMode::Live));
    let metrics = Arc::new(MetricsAdapter::new());

    let mut engine = StrategyEngine::new(config, position_cache, kill_switch, mode_controller, metrics);
    let now_ns = 1_000_000_000_000u64;

    let event = make_inference("AAPL", 0.9, 0.9, now_ns);
    let result = engine.evaluate(event, now_ns);

    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn stale_inference_suppresses_intent() {
    let mut config = StrategyConfig::default();
    config.inference_staleness_ms = 2000;
    let mut engine = make_engine(config);

    let now_ns = 1_000_000_000_000u64;
    let stale_inferred_ns = now_ns - 3_000_000_000u64; // 3 seconds old

    let event = make_inference("AAPL", 0.9, 0.9, stale_inferred_ns);
    let result = engine.evaluate(event, now_ns);

    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn low_confidence_suppresses_intent() {
    let config = StrategyConfig::default();
    let mut engine = make_engine(config);
    let now_ns = 1_000_000_000_000u64;

    let event = make_inference("AAPL", 0.9, 0.3, now_ns); // confidence 0.3 < 0.5 minimum
    let result = engine.evaluate(event, now_ns);

    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn hysteresis_suppresses_duplicate_long_signals() {
    let config = StrategyConfig::default();
    let mut engine = make_engine(config);
    let now_ns = 1_000_000_000_000u64;

    let event1 = make_inference("AAPL", 0.7, 0.8, now_ns);
    let result1 = engine.evaluate(event1, now_ns);
    assert!(result1.unwrap().is_some());

    let event2 = make_inference("AAPL", 0.8, 0.8, now_ns + 1);
    let result2 = engine.evaluate(event2, now_ns + 1);
    assert!(result2.unwrap().is_none()); // suppressed by hysteresis
}

#[test]
fn force_flatten_emits_flatten_intent() {
    let config = StrategyConfig::default();
    let position_cache = Arc::new(PositionCache::new());
    let kill_switch = Arc::new(KillSwitch::new());
    let mode_controller = Arc::new(ModeController::new(OperationMode::Live));
    let metrics = Arc::new(MetricsAdapter::new());

    let mut engine = StrategyEngine::new(config, position_cache, kill_switch, mode_controller, metrics);
    let now_ns = 1_000_000_000_000u64;

    let intent = engine.force_flatten("AAPL", now_ns);
    assert!(intent.is_none()); // flat position, no flatten needed
}
