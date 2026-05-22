use std::sync::Arc;

use strategy_service::strategy_core::application::kill_switch::KillSwitch;

#[test]
fn new_kill_switch_is_inactive() {
    let ks = KillSwitch::new();
    assert!(!ks.is_active());
    assert!(ks.reason().is_none());
}

#[test]
fn activate_sets_active_and_reason() {
    let ks = KillSwitch::new();
    ks.activate("test reason".to_string());
    assert!(ks.is_active());
    assert_eq!(ks.reason(), Some("test reason".to_string()));
}

#[test]
fn clear_resets_to_inactive() {
    let ks = KillSwitch::new();
    ks.activate("test reason".to_string());
    ks.clear();
    assert!(!ks.is_active());
    assert!(ks.reason().is_none());
}

#[test]
fn activate_overwrites_previous_reason() {
    let ks = KillSwitch::new();
    ks.activate("first reason".to_string());
    ks.activate("second reason".to_string());
    assert_eq!(ks.reason(), Some("second reason".to_string()));
}

#[test]
fn shared_kill_switch_works_across_threads() {
    let ks: Arc<KillSwitch> = Arc::new(KillSwitch::new());
    let ks_clone = ks.clone();

    std::thread::spawn(move || {
        ks_clone.activate("thread reason".to_string());
    })
    .join()
    .unwrap();

    assert!(ks.is_active());
    assert_eq!(ks.reason(), Some("thread reason".to_string()));
}
