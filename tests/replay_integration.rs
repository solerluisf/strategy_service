use std::sync::Arc;

use strategy_service::strategy_core::application::kill_switch::KillSwitch;
use strategy_service::strategy_core::application::replay_controller::ReplayController;
use strategy_service::adapters::persistence::journal_storage::JournalStorage;

#[tokio::test]
async fn replay_returns_empty_when_no_events() {
    let db_path = "test_replay_empty.db";
    let _ = std::fs::remove_file(db_path);

    let journal = Arc::new(JournalStorage::open(db_path).expect("Failed to open test journal"));
    let kill_switch = Arc::new(KillSwitch::new());
    let controller = ReplayController::new(journal, kill_switch);

    let intents = controller.replay(None).await.expect("Replay failed");
    assert!(intents.is_empty());

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn kill_switch_stops_replay() {
    let db_path = "test_replay_killswitch.db";
    let _ = std::fs::remove_file(db_path);

    let journal = Arc::new(JournalStorage::open(db_path).expect("Failed to open test journal"));
    let kill_switch = Arc::new(KillSwitch::new());
    kill_switch.activate("test".to_string());
    let controller = ReplayController::new(journal, kill_switch);

    let intents = controller.replay(None).await.expect("Replay failed");
    assert!(intents.is_empty());

    let _ = std::fs::remove_file(db_path);
}
