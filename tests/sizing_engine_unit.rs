use std::collections::HashMap;
use strategy_service::strategy_core::application::rules::sizing_engine::SizingEngine;
use strategy_service::strategy_core::domain::inference_input::ModelOutputs;
use strategy_service::strategy_core::domain::position::PositionState;
use strategy_service::strategy_core::domain::strategy_config::{SizingMethod, StrategyConfig};
use strategy_service::strategy_core::domain::trade_intent::SizeHint;

fn default_config() -> StrategyConfig {
    StrategyConfig::default()
}

fn flat_position() -> PositionState {
    PositionState {
        symbol: "AAPL".to_string(),
        net_units: 0.0,
        avg_entry_price: 0.0,
        last_updated_ns: 0,
    }
}

fn outputs_with_confidence(confidence: f64) -> ModelOutputs {
    ModelOutputs {
        forecast: Some(0.7),
        confidence: Some(confidence),
        action_score: None,
        regime_label: None,
        regime_strength: None,
        raw: HashMap::new(),
    }
}

fn outputs_with_atr(atr: f64) -> ModelOutputs {
    let mut raw = HashMap::new();
    raw.insert("atr_14".to_string(), atr);
    ModelOutputs {
        forecast: Some(0.7),
        confidence: Some(0.8),
        action_score: None,
        regime_label: None,
        regime_strength: None,
        raw,
    }
}

#[test]
fn portfolio_pct_scales_by_confidence() {
    let mut config = default_config();
    config.sizing_method = SizingMethod::PortfolioPct;
    config.base_size = 0.02;
    config.scale_by_confidence = true;

    let outputs = outputs_with_confidence(0.8);
    let hint = SizingEngine::compute(&outputs, &config, &flat_position());

    match hint {
        SizeHint::PortfolioPct(val) => {
            assert!((val - 0.016).abs() < 1e-9); // 0.02 * 0.8 = 0.016
        }
        _ => panic!("Expected PortfolioPct, got {:?}", hint),
    }
}

#[test]
fn portfolio_pct_does_not_scale_when_disabled() {
    let mut config = default_config();
    config.sizing_method = SizingMethod::PortfolioPct;
    config.base_size = 0.02;
    config.scale_by_confidence = false;

    let outputs = outputs_with_confidence(0.8);
    let hint = SizingEngine::compute(&outputs, &config, &flat_position());

    match hint {
        SizeHint::PortfolioPct(val) => {
            assert!((val - 0.02).abs() < 1e-9);
        }
        _ => panic!("Expected PortfolioPct, got {:?}", hint),
    }
}

#[test]
fn risk_based_uses_atr() {
    let mut config = default_config();
    config.sizing_method = SizingMethod::RiskBased;
    config.base_size = 100.0; // larger base to avoid the max(1.0) floor
    config.scale_by_confidence = true;

    let outputs = outputs_with_atr(2.5);
    let hint = SizingEngine::compute(&outputs, &config, &flat_position());

    match hint {
        SizeHint::RiskBased(val) => {
            // base_size * confidence = 100 * 0.8 = 80
            // risk_units = 80 / 2.5 = 32
            assert!((val - 32.0).abs() < 1e-9);
        }
        _ => panic!("Expected RiskBased, got {:?}", hint),
    }
}

#[test]
fn risk_based_falls_back_to_atr_1_0_when_missing() {
    let mut config = default_config();
    config.sizing_method = SizingMethod::RiskBased;
    config.base_size = 50.0; // larger base to avoid the max(1.0) floor
    config.scale_by_confidence = true;

    let outputs = outputs_with_confidence(0.8); // no atr_14 in raw
    let hint = SizingEngine::compute(&outputs, &config, &flat_position());

    match hint {
        SizeHint::RiskBased(val) => {
            // base_size * confidence = 50 * 0.8 = 40
            // risk_units = 40 / 1.0 (default atr) = 40
            assert!((val - 40.0).abs() < 1e-9);
        }
        _ => panic!("Expected RiskBased, got {:?}", hint),
    }
}

#[test]
fn units_method_returns_units() {
    let mut config = default_config();
    config.sizing_method = SizingMethod::Units;
    config.base_size = 100.0;
    config.scale_by_confidence = true;

    let outputs = outputs_with_confidence(0.5);
    let hint = SizingEngine::compute(&outputs, &config, &flat_position());

    match hint {
        SizeHint::Units(val) => {
            assert!((val - 50.0).abs() < 1e-9); // 100 * 0.5 = 50
        }
        _ => panic!("Expected Units, got {:?}", hint),
    }
}

#[test]
fn notional_method_returns_notional() {
    let mut config = default_config();
    config.sizing_method = SizingMethod::Notional;
    config.base_size = 10000.0;
    config.scale_by_confidence = true;

    let outputs = outputs_with_confidence(0.9);
    let hint = SizingEngine::compute(&outputs, &config, &flat_position());

    match hint {
        SizeHint::Notional(val) => {
            assert!((val - 9000.0).abs() < 1e-9); // 10000 * 0.9 = 9000
        }
        _ => panic!("Expected Notional, got {:?}", hint),
    }
}
