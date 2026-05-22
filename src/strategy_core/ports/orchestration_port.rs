use serde::Serialize;

use crate::strategy_core::domain::operation_mode::OperationMode;

pub enum OrchestrationCommand {
    SetOperationMode(OperationMode),
    ActivateKillSwitch { reason: String },
    ClearKillSwitch,
    ReloadStrategyConfig,
    SetStrategyEnabled { enabled: bool },
    SetLongEntryThreshold { value: f64 },
    SetShortEntryThreshold { value: f64 },
    SetConfidenceMinimum { value: f64 },
    SetHysteresisBand { value: f64 },
    SetEntryCooldown { ms: u64 },
    SetExitCooldown { ms: u64 },
    SetRegimeGate { regime: String, suppress_entries: bool },
    SetMaxLongUnits { symbol: Option<String>, units: f64 },
    SetMaxShortUnits { symbol: Option<String>, units: f64 },
    SetAllowShort { enabled: bool },
    SetSizingMethod { method: String },
    SetBaseSize { value: f64 },
    PauseSymbol { symbol: String },
    ResumeSymbol { symbol: String },
    FlattenSymbol { symbol: String },
    ResetCooldown { symbol: String },
    ResetHysteresis { symbol: String },
}

#[derive(Serialize)]
pub struct OrchestrationAck {
    pub command_type: String,
    pub success: bool,
    pub error: Option<String>,
    pub timestamp_ms: u64,
}
