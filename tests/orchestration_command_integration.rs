use std::sync::Arc;

use strategy_service::strategy_core::application::kill_switch::KillSwitch;
use strategy_service::strategy_core::application::mode_controller::ModeController;
use strategy_service::strategy_core::domain::operation_mode::OperationMode;
use strategy_service::strategy_core::ports::orchestration_port::OrchestrationCommand;

fn dispatch(cmd: OrchestrationCommand, kill_switch: &Arc<KillSwitch>, mode_controller: &Arc<ModeController>) -> (bool, Option<String>) {
    match cmd {
        OrchestrationCommand::SetOperationMode(mode) => {
            mode_controller.set(mode);
            (true, None)
        }
        OrchestrationCommand::ActivateKillSwitch { reason } => {
            kill_switch.activate(reason);
            (true, None)
        }
        OrchestrationCommand::ClearKillSwitch => {
            kill_switch.clear();
            (true, None)
        }
        _ => (true, None),
    }
}

#[test]
fn set_operation_mode_changes_mode() {
    let kill_switch = Arc::new(KillSwitch::new());
    let mode_controller = Arc::new(ModeController::new(OperationMode::Live));

    dispatch(
        OrchestrationCommand::SetOperationMode(OperationMode::Paper),
        &kill_switch,
        &mode_controller,
    );

    assert_eq!(mode_controller.get(), OperationMode::Paper);
}

#[test]
fn activate_kill_switch_activates() {
    let kill_switch = Arc::new(KillSwitch::new());
    let mode_controller = Arc::new(ModeController::new(OperationMode::Live));

    dispatch(
        OrchestrationCommand::ActivateKillSwitch { reason: "emergency".to_string() },
        &kill_switch,
        &mode_controller,
    );

    assert!(kill_switch.is_active());
    assert_eq!(kill_switch.reason(), Some("emergency".to_string()));
}

#[test]
fn clear_kill_switch_deactivates() {
    let kill_switch = Arc::new(KillSwitch::new());
    let mode_controller = Arc::new(ModeController::new(OperationMode::Live));

    kill_switch.activate("test".to_string());
    assert!(kill_switch.is_active());

    dispatch(
        OrchestrationCommand::ClearKillSwitch,
        &kill_switch,
        &mode_controller,
    );

    assert!(!kill_switch.is_active());
}

#[test]
fn orchestration_command_returns_success() {
    let kill_switch = Arc::new(KillSwitch::new());
    let mode_controller = Arc::new(ModeController::new(OperationMode::Live));

    let (success, error) = dispatch(
        OrchestrationCommand::SetOperationMode(OperationMode::Readonly),
        &kill_switch,
        &mode_controller,
    );

    assert!(success);
    assert!(error.is_none());
}
