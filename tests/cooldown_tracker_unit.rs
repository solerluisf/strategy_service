use strategy_service::strategy_core::application::rules::cooldown_tracker::{CooldownResult, CooldownTracker};
use strategy_service::strategy_core::domain::strategy_config::StrategyConfig;
use strategy_service::strategy_core::domain::trade_intent::IntentType;

fn default_config() -> StrategyConfig {
    StrategyConfig::default()
}

#[test]
fn first_intent_is_ready() {
    let config = default_config();
    let tracker = CooldownTracker::new();
    let now_ns = 1_000_000_000_000u64;
    let result = tracker.check("AAPL", &IntentType::Entry, now_ns, &config);
    assert!(matches!(result, CooldownResult::Ready));
}

#[test]
fn second_intent_within_cooldown_is_suppressed() {
    let config = default_config();
    let mut tracker = CooldownTracker::new();
    let now_ns = 1_000_000_000_000u64;

    tracker.mark_emitted("AAPL", &IntentType::Entry, now_ns);

    // 3 seconds later (cooldown is 5000ms = 5s)
    let later_ns = now_ns + 3_000_000_000u64;
    let result = tracker.check("AAPL", &IntentType::Entry, later_ns, &config);
    match result {
        CooldownResult::Suppressed { remaining_ms } => {
            assert!(remaining_ms > 0);
            assert!(remaining_ms <= 2000);
        }
        CooldownResult::Ready => panic!("Expected suppressed, got ready"),
    }
}

#[test]
fn after_cooldown_expires_is_ready() {
    let config = default_config();
    let mut tracker = CooldownTracker::new();
    let now_ns = 1_000_000_000_000u64;

    tracker.mark_emitted("AAPL", &IntentType::Entry, now_ns);

    // 5001ms later (just past 5000ms cooldown)
    let later_ns = now_ns + 5_001_000_000u64;
    let result = tracker.check("AAPL", &IntentType::Entry, later_ns, &config);
    assert!(matches!(result, CooldownResult::Ready));
}

#[test]
fn entry_and_exit_cooldowns_are_independent() {
    let config = default_config();
    let mut tracker = CooldownTracker::new();
    let now_ns = 1_000_000_000_000u64;

    tracker.mark_emitted("AAPL", &IntentType::Entry, now_ns);

    // 1 second later, entry should be suppressed but exit should be ready
    let later_ns = now_ns + 1_000_000_000u64;

    let entry_result = tracker.check("AAPL", &IntentType::Entry, later_ns, &config);
    assert!(matches!(entry_result, CooldownResult::Suppressed { .. }));

    let exit_result = tracker.check("AAPL", &IntentType::Exit, later_ns, &config);
    assert!(matches!(exit_result, CooldownResult::Ready));
}

#[test]
fn different_symbols_have_separate_cooldowns() {
    let config = default_config();
    let mut tracker = CooldownTracker::new();
    let now_ns = 1_000_000_000_000u64;

    tracker.mark_emitted("AAPL", &IntentType::Entry, now_ns);

    let later_ns = now_ns + 1_000_000_000u64;

    let aapl_result = tracker.check("AAPL", &IntentType::Entry, later_ns, &config);
    assert!(matches!(aapl_result, CooldownResult::Suppressed { .. }));

    let msft_result = tracker.check("MSFT", &IntentType::Entry, later_ns, &config);
    assert!(matches!(msft_result, CooldownResult::Ready));
}

#[test]
fn reset_clears_cooldown() {
    let config = default_config();
    let mut tracker = CooldownTracker::new();
    let now_ns = 1_000_000_000_000u64;

    tracker.mark_emitted("AAPL", &IntentType::Entry, now_ns);
    tracker.reset("AAPL");

    let later_ns = now_ns + 1_000_000_000u64;
    let result = tracker.check("AAPL", &IntentType::Entry, later_ns, &config);
    assert!(matches!(result, CooldownResult::Ready));
}
