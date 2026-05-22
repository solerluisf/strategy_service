use strategy_service::strategy_core::application::rules::hysteresis_filter::{HysteresisFilter, HysteresisResult};
use strategy_service::strategy_core::application::rules::threshold_gate::ThresholdDecision;
use strategy_service::strategy_core::domain::strategy_config::StrategyConfig;

fn default_config() -> StrategyConfig {
    StrategyConfig::default()
}

#[test]
fn flat_state_allows_long_signal() {
    let config = default_config();
    let mut filter = HysteresisFilter::new();
    let result = filter.check("AAPL", ThresholdDecision::Long, 0.7, &config);
    assert!(matches!(result, HysteresisResult::Allow(ThresholdDecision::Long)));
    assert_eq!(filter.get_direction("AAPL"), 1);
}

#[test]
fn long_state_suppresses_another_long_signal() {
    let config = default_config();
    let mut filter = HysteresisFilter::new();
    filter.check("AAPL", ThresholdDecision::Long, 0.7, &config);

    let result = filter.check("AAPL", ThresholdDecision::Long, 0.7, &config);
    assert!(matches!(result, HysteresisResult::Suppress));
}

#[test]
fn long_state_suppresses_weak_short_signal() {
    let config = default_config();
    let mut filter = HysteresisFilter::new();
    filter.check("AAPL", ThresholdDecision::Long, 0.7, &config);

    // short_threshold = -0.6, hysteresis_band = 0.15, reversal = -0.75
    // forecast -0.7 is not <= -0.75, so should suppress
    let result = filter.check("AAPL", ThresholdDecision::Short, -0.7, &config);
    assert!(matches!(result, HysteresisResult::Suppress));
}

#[test]
fn long_state_allows_strong_short_signal() {
    let config = default_config();
    let mut filter = HysteresisFilter::new();
    filter.check("AAPL", ThresholdDecision::Long, 0.7, &config);

    // forecast -0.8 <= -0.75, should allow
    let result = filter.check("AAPL", ThresholdDecision::Short, -0.8, &config);
    assert!(matches!(result, HysteresisResult::Allow(ThresholdDecision::Short)));
    assert_eq!(filter.get_direction("AAPL"), -1);
}

#[test]
fn reset_returns_state_to_flat() {
    let config = default_config();
    let mut filter = HysteresisFilter::new();
    filter.check("AAPL", ThresholdDecision::Long, 0.7, &config);
    assert_eq!(filter.get_direction("AAPL"), 1);

    filter.reset("AAPL");
    assert_eq!(filter.get_direction("AAPL"), 0);
}

#[test]
fn short_state_suppresses_another_short_signal() {
    let config = default_config();
    let mut filter = HysteresisFilter::new();
    filter.check("AAPL", ThresholdDecision::Short, -0.7, &config);

    let result = filter.check("AAPL", ThresholdDecision::Short, -0.7, &config);
    assert!(matches!(result, HysteresisResult::Suppress));
}

#[test]
fn short_state_allows_strong_long_signal() {
    let config = default_config();
    let mut filter = HysteresisFilter::new();
    filter.check("AAPL", ThresholdDecision::Short, -0.7, &config);

    // long_threshold = 0.6, hysteresis_band = 0.15, reversal = 0.75
    let result = filter.check("AAPL", ThresholdDecision::Long, 0.8, &config);
    assert!(matches!(result, HysteresisResult::Allow(ThresholdDecision::Long)));
}

#[test]
fn exit_signal_allowed_from_long_state() {
    let config = default_config();
    let mut filter = HysteresisFilter::new();
    filter.check("AAPL", ThresholdDecision::Long, 0.7, &config);

    let result = filter.check("AAPL", ThresholdDecision::Exit, 0.05, &config);
    assert!(matches!(result, HysteresisResult::Allow(ThresholdDecision::Exit)));
}
