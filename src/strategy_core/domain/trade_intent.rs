use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeIntent {
    pub intent_id: String,
    pub symbol: String,
    pub side: IntentSide,
    pub size_hint: SizeHint,
    pub intent_type: IntentType,
    pub urgency: IntentUrgency,
    pub strategy_id: String,
    pub strategy_version: String,
    pub model_id: String,
    pub model_version: String,
    pub trace_id: String,
    pub timestamp_ns: u64,
    pub generated_ns: u64,
    pub sequence_number: u64,
    pub expires_ns: Option<u64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum IntentSide {
    Long,
    Short,
    CloseLong,
    CloseShort,
    Flatten,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SizeHint {
    Units(f64),
    Notional(f64),
    PortfolioPct(f64),
    RiskBased(f64),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum IntentType {
    Entry,
    Exit,
    ScaleIn,
    ScaleOut,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum IntentUrgency {
    Passive,
    Normal,
    Aggressive,
}

pub fn current_time_ns() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

impl TradeIntent {
    pub fn new(
        inference: &crate::strategy_core::domain::inference_input::InferenceEvent,
        side: IntentSide,
        size_hint: SizeHint,
        intent_type: IntentType,
        urgency: IntentUrgency,
        strategy_id: &str,
        strategy_version: &str,
        sequence_number: u64,
        expires_ns: Option<u64>,
    ) -> Self {
        Self {
            intent_id: uuid::Uuid::new_v4().to_string(),
            symbol: inference.symbol.clone(),
            side,
            size_hint,
            intent_type,
            urgency,
            strategy_id: strategy_id.to_string(),
            strategy_version: strategy_version.to_string(),
            model_id: inference.model_id.clone(),
            model_version: inference.model_version.clone(),
            trace_id: inference.trace_id.clone(),
            timestamp_ns: inference.timestamp_ns,
            generated_ns: current_time_ns(),
            sequence_number,
            expires_ns,
            metadata: HashMap::new(),
        }
    }

    pub fn is_expired(&self, now_ns: u64) -> bool {
        self.expires_ns.map_or(false, |exp| now_ns > exp)
    }
}
