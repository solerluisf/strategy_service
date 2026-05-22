use strategy_service::strategy_core::application::rules::threshold_gate::{ThresholdGate, ThresholdDecision};
use strategy_service::strategy_core::domain::strategy_config::StrategyConfig;

fn default_config() -> StrategyConfig {
    StrategyConfig::default()
}

#[test]
fn forecast_above_long_threshold_returns_long() {
    let config = default_config();
    let decision = ThresholdGate::evaluate(0.7, 0.6, &config);
    assert!(matches!(decision, ThresholdDecision::Long));
}

#[test]
fn forecast_below_short_threshold_returns_short() {
    let config = default_config();
    let decision = ThresholdGate::evaluate(-0.7, 0.6, &config);
    assert!(matches!(decision, ThresholdDecision::Short));
}

#[test]
fn forecast_near_zero_returns_exit() {
    let config = default_config();
    let decision = ThresholdGate::evaluate(0.05, 0.6, &config);
    assert!(matches!(decision, ThresholdDecision::Exit));
}

#[test]
fn confidence_below_minimum_returns_no_signal() {
    let config = default_config();
    let decision = ThresholdGate::evaluate(0.9, 0.3, &config);
    assert!(matches!(decision, ThresholdDecision::NoSignal));
}

#[test]
fn forecast_exactly_at_long_threshold_returns_long() {
    let config = default_config();
    let decision = ThresholdGate::evaluate(0.6, 0.6, &config);
    assert!(matches!(decision, ThresholdDecision::Long));
}

#[test]
fn forecast_exactly_at_short_threshold_returns_short() {
    let config = default_config();
    let decision = ThresholdGate::evaluate(-0.6, 0.6, &config);
    assert!(matches!(decision, ThresholdDecision::Short));
}

#[test]
fn forecast_between_thresholds_returns_no_signal() {
    let config = default_config();
    let decision = ThresholdGate::evaluate(0.3, 0.6, &config);
    assert!(matches!(decision, ThresholdDecision::NoSignal));
}
