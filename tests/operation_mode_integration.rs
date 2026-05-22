use std::sync::Arc;

use strategy_service::strategy_core::application::mode_controller::ModeController;
use strategy_service::strategy_core::domain::operation_mode::OperationMode;

#[test]
fn new_controller_has_initial_mode() {
    let mc = ModeController::new(OperationMode::Live);
    assert_eq!(mc.get(), OperationMode::Live);
}

#[test]
fn set_changes_mode() {
    let mc = ModeController::new(OperationMode::Live);
    mc.set(OperationMode::Paper);
    assert_eq!(mc.get(), OperationMode::Paper);
}

#[test]
fn is_active_returns_true_for_live() {
    let mc = ModeController::new(OperationMode::Live);
    assert!(mc.is_active());
}

#[test]
fn is_active_returns_true_for_paper() {
    let mc = ModeController::new(OperationMode::Paper);
    assert!(mc.is_active());
}

#[test]
fn is_active_returns_false_for_readonly() {
    let mc = ModeController::new(OperationMode::Readonly);
    assert!(!mc.is_active());
}

#[test]
fn is_active_returns_false_for_offline() {
    let mc = ModeController::new(OperationMode::Offline);
    assert!(!mc.is_active());
}

#[test]
fn shared_mode_controller_works_across_threads() {
    let mc: Arc<ModeController> = Arc::new(ModeController::new(OperationMode::Live));
    let mc_clone = mc.clone();

    std::thread::spawn(move || {
        mc_clone.set(OperationMode::Offline);
    })
    .join()
    .unwrap();

    assert_eq!(mc.get(), OperationMode::Offline);
}
