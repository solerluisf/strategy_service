use strategy_service::strategy_core::application::rules::position_gate::{PositionGate, PositionGateResult};
use strategy_service::strategy_core::domain::position::PositionState;
use strategy_service::strategy_core::domain::strategy_config::StrategyConfig;
use strategy_service::strategy_core::domain::trade_intent::IntentSide;

fn default_config() -> StrategyConfig {
    StrategyConfig::default()
}

fn flat_position(symbol: &str) -> PositionState {
    PositionState {
        symbol: symbol.to_string(),
        net_units: 0.0,
        avg_entry_price: 0.0,
        last_updated_ns: 0,
    }
}

fn long_position(symbol: &str, units: f64) -> PositionState {
    PositionState {
        symbol: symbol.to_string(),
        net_units: units,
        avg_entry_price: 100.0,
        last_updated_ns: 0,
    }
}

fn short_position(symbol: &str, units: f64) -> PositionState {
    PositionState {
        symbol: symbol.to_string(),
        net_units: -units,
        avg_entry_price: 100.0,
        last_updated_ns: 0,
    }
}

#[test]
fn flat_position_suppresses_close_long() {
    let config = default_config();
    let position = flat_position("AAPL");
    let result = PositionGate::check(&position, &IntentSide::CloseLong, &config);
    assert!(matches!(result, PositionGateResult::Suppress { .. }));
}

#[test]
fn flat_position_suppresses_close_short() {
    let config = default_config();
    let position = flat_position("AAPL");
    let result = PositionGate::check(&position, &IntentSide::CloseShort, &config);
    assert!(matches!(result, PositionGateResult::Suppress { .. }));
}

#[test]
fn long_at_max_units_suppresses_long_intent() {
    let config = default_config();
    let position = long_position("AAPL", 1000.0); // max_long_units = 1000
    let result = PositionGate::check(&position, &IntentSide::Long, &config);
    assert!(matches!(result, PositionGateResult::Suppress { .. }));
}

#[test]
fn long_under_max_allows_long_intent() {
    let config = default_config();
    let position = long_position("AAPL", 500.0);
    let result = PositionGate::check(&position, &IntentSide::Long, &config);
    assert!(matches!(result, PositionGateResult::Allow));
}

#[test]
fn short_suppressed_when_allow_short_false() {
    let config = default_config();
    assert!(!config.allow_short);
    let position = flat_position("AAPL");
    let result = PositionGate::check(&position, &IntentSide::Short, &config);
    assert!(matches!(result, PositionGateResult::Suppress { .. }));
}

#[test]
fn short_at_max_units_suppresses_short_intent() {
    let mut config = default_config();
    config.allow_short = true;
    let position = short_position("AAPL", 1000.0); // max_short_units = 1000
    let result = PositionGate::check(&position, &IntentSide::Short, &config);
    assert!(matches!(result, PositionGateResult::Suppress { .. }));
}

#[test]
fn long_position_allows_close_long() {
    let config = default_config();
    let position = long_position("AAPL", 100.0);
    let result = PositionGate::check(&position, &IntentSide::CloseLong, &config);
    assert!(matches!(result, PositionGateResult::Allow));
}

#[test]
fn short_position_allows_close_short() {
    let config = default_config();
    let position = short_position("AAPL", 100.0);
    let result = PositionGate::check(&position, &IntentSide::CloseShort, &config);
    assert!(matches!(result, PositionGateResult::Allow));
}

#[test]
fn flat_position_suppresses_flatten() {
    let config = default_config();
    let position = flat_position("AAPL");
    let result = PositionGate::check(&position, &IntentSide::Flatten, &config);
    assert!(matches!(result, PositionGateResult::Suppress { .. }));
}

#[test]
fn long_position_allows_flatten() {
    let config = default_config();
    let position = long_position("AAPL", 100.0);
    let result = PositionGate::check(&position, &IntentSide::Flatten, &config);
    assert!(matches!(result, PositionGateResult::Allow));
}
