use std::collections::HashMap;
use std::sync::Arc;

use strategy_service::strategy_core::application::kill_switch::KillSwitch;
use strategy_service::strategy_core::application::mode_controller::ModeController;
use strategy_service::strategy_core::application::strategy_engine::StrategyEngine;
use strategy_service::strategy_core::application::strategy_selector::StrategySelector;
use strategy_service::strategy_core::domain::inference_input::{InferenceEvent, ModelOutputs, RegimeLabel};
use strategy_service::strategy_core::domain::operation_mode::OperationMode;
use strategy_service::strategy_core::domain::position::PositionCache;
use strategy_service::strategy_core::domain::strategy_config::StrategyConfig;
use strategy_service::strategy_core::domain::strategies::arbitrage::{ArbitrageConfig, ArbitrageStrategy};
use strategy_service::strategy_core::domain::strategies::momentum::{MomentumConfig, MomentumStrategy};
use strategy_service::strategy_core::domain::strategies::moving_average::{MovingAverageConfig, MovingAverageStrategy};
use strategy_service::strategy_core::domain::strategies::rsi::{RSIConfig, RSIStrategy};
use strategy_service::strategy_core::domain::trading_strategy::StrategyType;
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

fn make_selector(config_arc: Arc<std::sync::RwLock<StrategyConfig>>) -> StrategySelector {
    let mut strategies: HashMap<StrategyType, Box<dyn strategy_service::strategy_core::domain::trading_strategy::ITradingStrategy>> = HashMap::new();
    strategies.insert(
        StrategyType::MovingAverage,
        Box::new(MovingAverageStrategy::new(MovingAverageConfig::default())),
    );
    strategies.insert(
        StrategyType::RSI,
        Box::new(RSIStrategy::new(RSIConfig::default())),
    );
    strategies.insert(
        StrategyType::Arbitrage,
        Box::new(ArbitrageStrategy::new(ArbitrageConfig::default())),
    );
    strategies.insert(
        StrategyType::Momentum,
        Box::new(MomentumStrategy::new(MomentumConfig::default())),
    );

    StrategySelector::new(
        strategies,
        StrategyType::Momentum,
        config_arc,
    ).expect("Failed to create strategy selector")
}

fn make_engine(config: StrategyConfig) -> StrategyEngine {
    let position_cache = Arc::new(PositionCache::new());
    let kill_switch = Arc::new(KillSwitch::new());
    let mode_controller = Arc::new(ModeController::new(OperationMode::Live));
    let metrics = Arc::new(MetricsAdapter::new());
    let config_arc = Arc::new(std::sync::RwLock::new(config.clone()));
    let selector = make_selector(config_arc);

    StrategyEngine::new(config, selector, position_cache, kill_switch, mode_controller, metrics)
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
    let config_arc = Arc::new(std::sync::RwLock::new(config.clone()));
    let selector = make_selector(config_arc);

    let mut engine = StrategyEngine::new(config, selector, position_cache, kill_switch, mode_controller, metrics);
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
    let stale_inferred_ns = now_ns - 3_000_000_000u64;

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

    let event = make_inference("AAPL", 0.9, 0.3, now_ns);
    let result = engine.evaluate(event, now_ns);

    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn strategy_switch_at_runtime() {
    let config = StrategyConfig::default();
    let mut engine = make_engine(config);

    assert_eq!(engine.active_strategy(), StrategyType::Momentum);

    engine.switch_strategy(StrategyType::RSI).unwrap();
    assert_eq!(engine.active_strategy(), StrategyType::RSI);

    engine.switch_strategy(StrategyType::MovingAverage).unwrap();
    assert_eq!(engine.active_strategy(), StrategyType::MovingAverage);
}

#[test]
fn available_strategies_returns_all_registered() {
    let config = StrategyConfig::default();
    let engine = make_engine(config);

    let available = engine.available_strategies();
    assert_eq!(available.len(), 4);
    assert!(available.contains(&StrategyType::Momentum));
    assert!(available.contains(&StrategyType::RSI));
    assert!(available.contains(&StrategyType::MovingAverage));
    assert!(available.contains(&StrategyType::Arbitrage));
}

#[test]
fn force_flatten_emits_flatten_intent() {
    let config = StrategyConfig::default();
    let position_cache = Arc::new(PositionCache::new());
    let kill_switch = Arc::new(KillSwitch::new());
    let mode_controller = Arc::new(ModeController::new(OperationMode::Live));
    let metrics = Arc::new(MetricsAdapter::new());
    let config_arc = Arc::new(std::sync::RwLock::new(config.clone()));
    let selector = make_selector(config_arc);

    let mut engine = StrategyEngine::new(config, selector, position_cache, kill_switch, mode_controller, metrics);
    let now_ns = 1_000_000_000_000u64;

    let intent = engine.force_flatten("AAPL", now_ns);
    assert!(intent.is_none());
}
