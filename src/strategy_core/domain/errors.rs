use thiserror::Error;

#[derive(Error, Debug)]
pub enum StrategyError {
    #[error("Wire format error: {0}")]
    WireFormat(String),

    #[error("Decode error: {0}")]
    Decode(String),

    #[error("Encode error: {0}")]
    Encode(String),

    #[error("Messaging error: {0}")]
    Messaging(String),

    #[error("Journal error: {0}")]
    Journal(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Position error: {0}")]
    Position(String),

    #[error("Risk gate error: {0}")]
    RiskGate(String),

    #[error("Idempotency error: {0}")]
    Idempotency(String),

    #[error("Shutdown requested")]
    Shutdown,
}
