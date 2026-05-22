use std::sync::RwLock;

use crate::strategy_core::domain::operation_mode::OperationMode;

pub struct ModeController {
    mode: RwLock<OperationMode>,
}

impl ModeController {
    pub fn new(initial: OperationMode) -> Self {
        Self {
            mode: RwLock::new(initial),
        }
    }

    pub fn get(&self) -> OperationMode {
        *self.mode.read().unwrap()
    }

    pub fn set(&self, mode: OperationMode) {
        let mut current = self.mode.write().unwrap();
        tracing::info!(?mode, old_mode = ?*current, "Operation mode changed");
        *current = mode;
    }

    pub fn is_active(&self) -> bool {
        let mode = self.get();
        mode != OperationMode::Offline && mode != OperationMode::Readonly
    }
}
