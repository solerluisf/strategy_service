use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderLifecycleEvent {
    pub event_id: String,
    pub execution_id: String,
    pub client_order_id: Option<String>,
    pub symbol: String,
    pub event_type: OrderLifecycleEventType,
    pub timestamp: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderLifecycleEventType {
    Submitted,
    PartialFill,
    Filled,
    Rejected,
    Cancelled,
    Replaced,
    Expired,
    Error,
}
