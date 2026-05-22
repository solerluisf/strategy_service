use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceEvent {
    pub event_id: String,
    pub symbol: String,
    pub timestamp_ns: u64,
    pub inferred_ns: u64,
    pub sequence_number: u64,
    pub trace_id: String,
    pub model_id: String,
    pub model_version: String,
    pub feature_version: String,
    pub regime: RegimeLabel,
    pub outputs: ModelOutputs,
    pub latency_us: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelOutputs {
    pub forecast: Option<f64>,
    pub confidence: Option<f64>,
    pub action_score: Option<f64>,
    pub regime_label: Option<f64>,
    pub regime_strength: Option<f64>,
    pub raw: HashMap<String, f64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RegimeLabel {
    Ranging,
    Trending,
    Volatile,
    Unknown,
}
