use crate::strategy_core::domain::operation_mode::OperationMode;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub model_endpoint: String,
    pub gateway_lifecycle_endpoint: String,
    pub gateway_health_endpoint: String,
    pub risk_router_endpoint: String,
    pub publish_endpoint: String,
    pub control_endpoint: String,
    pub ack_endpoint: String,
    pub operation_mode: OperationMode,
    pub symbols: Vec<String>,
    pub strategy_config_path: String,
    pub journal_db_path: String,
    pub health_port: u16,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, String> {
        Ok(Self {
            model_endpoint: std::env::var("MODEL_ENDPOINT")
                .unwrap_or_else(|_| "tcp://127.0.0.1:5555".to_string()),
            gateway_lifecycle_endpoint: std::env::var("GATEWAY_LIFECYCLE_ENDPOINT")
                .unwrap_or_else(|_| "tcp://127.0.0.1:5557".to_string()),
            gateway_health_endpoint: std::env::var("GATEWAY_HEALTH_ENDPOINT")
                .unwrap_or_else(|_| "tcp://127.0.0.1:5560".to_string()),
            risk_router_endpoint: std::env::var("RISK_ROUTER_ENDPOINT")
                .unwrap_or_else(|_| "tcp://127.0.0.1:5558".to_string()),
            publish_endpoint: std::env::var("PUBLISH_ENDPOINT")
                .unwrap_or_else(|_| "tcp://127.0.0.1:5561".to_string()),
            control_endpoint: std::env::var("CONTROL_ENDPOINT")
                .unwrap_or_else(|_| "tcp://127.0.0.1:5562".to_string()),
            ack_endpoint: std::env::var("ACK_ENDPOINT")
                .unwrap_or_else(|_| "tcp://127.0.0.1:5563".to_string()),
            operation_mode: std::env::var("OPERATION_MODE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(OperationMode::Live),
            symbols: std::env::var("SYMBOLS")
                .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default(),
            strategy_config_path: std::env::var("STRATEGY_CONFIG_PATH")
                .unwrap_or_else(|_| "./strategy.toml".to_string()),
            journal_db_path: std::env::var("JOURNAL_DB_PATH")
                .unwrap_or_else(|_| "strategy_journal.db".to_string()),
            health_port: std::env::var("HEALTH_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(9094),
        })
    }
}
