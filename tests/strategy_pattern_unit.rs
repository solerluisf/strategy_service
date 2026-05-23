use std::collections::HashMap;

use strategy_service::strategy_core::domain::inference_input::{InferenceEvent, ModelOutputs, RegimeLabel};
use strategy_service::strategy_core::domain::strategies::moving_average::{MovingAverageConfig, MovingAverageStrategy};
use strategy_service::strategy_core::domain::strategies::rsi::{RSIConfig, RSIStrategy};
use strategy_service::strategy_core::domain::strategies::arbitrage::{ArbitrageConfig, ArbitrageStrategy};
use strategy_service::strategy_core::domain::strategies::momentum::{MomentumConfig, MomentumStrategy};
use strategy_service::strategy_core::domain::trading_strategy::{ITradingStrategy, StrategyType};
use strategy_service::strategy_core::domain::trade_intent::{IntentSide, IntentType};

fn make_event(symbol: &str, price: f64, confidence: f64) -> InferenceEvent {
    let mut raw = HashMap::new();
    raw.insert("price".to_string(), price);

    InferenceEvent {
        event_id: format!("evt-{}", symbol),
        symbol: symbol.to_string(),
        timestamp_ns: 1_000_000_000_000u64,
        inferred_ns: 1_000_000_000_000u64,
        sequence_number: 1,
        trace_id: "trace-1".to_string(),
        model_id: "model-v1".to_string(),
        model_version: "1.0.0".to_string(),
        feature_version: "1.0.0".to_string(),
        regime: RegimeLabel::Trending,
        outputs: ModelOutputs {
            forecast: Some(0.5),
            confidence: Some(confidence),
            action_score: None,
            regime_label: None,
            regime_strength: None,
            raw,
        },
        latency_us: 100,
    }
}

#[test]
fn moving_average_strategy_name() {
    let strategy = MovingAverageStrategy::new(MovingAverageConfig::default());
    assert_eq!(strategy.name(), StrategyType::MovingAverage);
}

#[test]
fn moving_average_returns_none_without_history() {
    let strategy = MovingAverageStrategy::new(MovingAverageConfig::default());
    let event = make_event("AAPL", 100.0, 0.8);
    assert!(strategy.evaluate(&event).is_none());
}

#[test]
fn rsi_strategy_name() {
    let strategy = RSIStrategy::new(RSIConfig::default());
    assert_eq!(strategy.name(), StrategyType::RSI);
}

#[test]
fn rsi_returns_none_without_history() {
    let strategy = RSIStrategy::new(RSIConfig::default());
    let event = make_event("AAPL", 100.0, 0.8);
    assert!(strategy.evaluate(&event).is_none());
}

#[test]
fn arbitrage_strategy_name() {
    let strategy = ArbitrageStrategy::new(ArbitrageConfig::default());
    assert_eq!(strategy.name(), StrategyType::Arbitrage);
}

#[test]
fn arbitrage_returns_none_without_reference() {
    let strategy = ArbitrageStrategy::new(ArbitrageConfig::default());
    let event = make_event("AAPL", 100.0, 0.8);
    assert!(strategy.evaluate(&event).is_none());
}

#[test]
fn arbitrage_signals_on_large_spread() {
    let strategy = ArbitrageStrategy::new(ArbitrageConfig::default());
    strategy.set_reference_price("AAPL", 100.0);

    let event = make_event("AAPL", 101.0, 0.8);
    let signal = strategy.evaluate(&event);
    assert!(signal.is_some());
    let signal = signal.unwrap();
    assert!(matches!(signal.side, IntentSide::Short));
    assert!(matches!(signal.intent_type, IntentType::Entry));
}

#[test]
fn momentum_strategy_name() {
    let strategy = MomentumStrategy::new(MomentumConfig::default());
    assert_eq!(strategy.name(), StrategyType::Momentum);
}

#[test]
fn momentum_returns_none_with_low_confidence() {
    let strategy = MomentumStrategy::new(MomentumConfig::default());
    let event = make_event("AAPL", 100.0, 0.3);
    assert!(strategy.evaluate(&event).is_none());
}

#[test]
fn momentum_returns_none_without_history() {
    let strategy = MomentumStrategy::new(MomentumConfig::default());
    let event = make_event("AAPL", 100.0, 0.8);
    assert!(strategy.evaluate(&event).is_none());
}
