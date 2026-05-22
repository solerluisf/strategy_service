use strategy_service::strategy_core::application::rules::regime_gate::RegimeGate;
use strategy_service::strategy_core::domain::inference_input::RegimeLabel;
use strategy_service::strategy_core::domain::strategy_config::StrategyConfig;
use strategy_service::strategy_core::domain::trade_intent::IntentType;

fn default_config() -> StrategyConfig {
    StrategyConfig::default()
}

#[test]
fn volatile_regime_suppresses_entry() {
    let config = default_config();
    assert!(config.suppress_entries_in_volatile);
    let allowed = RegimeGate::allows(&RegimeLabel::Volatile, &IntentType::Entry, &config);
    assert!(!allowed);
}

#[test]
fn volatile_regime_suppresses_scale_in() {
    let config = default_config();
    let allowed = RegimeGate::allows(&RegimeLabel::Volatile, &IntentType::ScaleIn, &config);
    assert!(!allowed);
}

#[test]
fn volatile_regime_allows_exit() {
    let config = default_config();
    let allowed = RegimeGate::allows(&RegimeLabel::Volatile, &IntentType::Exit, &config);
    assert!(allowed);
}

#[test]
fn trending_regime_allows_entry() {
    let config = default_config();
    let allowed = RegimeGate::allows(&RegimeLabel::Trending, &IntentType::Entry, &config);
    assert!(allowed);
}

#[test]
fn ranging_regime_allows_all() {
    let config = default_config();
    assert!(RegimeGate::allows(&RegimeLabel::Ranging, &IntentType::Entry, &config));
    assert!(RegimeGate::allows(&RegimeLabel::Ranging, &IntentType::Exit, &config));
    assert!(RegimeGate::allows(&RegimeLabel::Ranging, &IntentType::ScaleIn, &config));
    assert!(RegimeGate::allows(&RegimeLabel::Ranging, &IntentType::ScaleOut, &config));
}

#[test]
fn unknown_regime_allows_all() {
    let config = default_config();
    assert!(RegimeGate::allows(&RegimeLabel::Unknown, &IntentType::Entry, &config));
}
