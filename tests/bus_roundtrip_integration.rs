use std::collections::HashMap;

use strategy_service::strategy_core::domain::inference_input::{InferenceEvent, ModelOutputs, RegimeLabel};
use strategy_service::strategy_core::domain::trade_intent::{TradeIntent, current_time_ns};
use strategy_service::adapters::messaging::wire_codec::{encode_trade_intent, decode_trade_intent, encode_msgpack, decode_inference_event};

fn make_inference() -> InferenceEvent {
    InferenceEvent {
        event_id: "evt-test-1".to_string(),
        symbol: "AAPL".to_string(),
        timestamp_ns: 1_000_000_000_000u64,
        inferred_ns: 1_000_000_000_000u64,
        sequence_number: 1,
        trace_id: "trace-test-1".to_string(),
        model_id: "model-v1".to_string(),
        model_version: "1.0.0".to_string(),
        feature_version: "1.0.0".to_string(),
        regime: RegimeLabel::Trending,
        outputs: ModelOutputs {
            forecast: Some(0.7),
            confidence: Some(0.8),
            action_score: None,
            regime_label: None,
            regime_strength: None,
            raw: HashMap::new(),
        },
        latency_us: 100,
    }
}

fn make_intent() -> TradeIntent {
    let inference = make_inference();
    TradeIntent::new(
        &inference,
        strategy_service::strategy_core::domain::trade_intent::IntentSide::Long,
        strategy_service::strategy_core::domain::trade_intent::SizeHint::PortfolioPct(0.02),
        strategy_service::strategy_core::domain::trade_intent::IntentType::Entry,
        strategy_service::strategy_core::domain::trade_intent::IntentUrgency::Normal,
        "strategy-v1",
        "1.0.0",
        1,
        None,
    )
}

#[test]
fn encode_decode_inference_event_roundtrip() {
    let event = make_inference();
    let encoded = encode_msgpack(&event).expect("Failed to encode");

    let (decoded, wire_format) = decode_inference_event(&encoded).expect("Failed to decode");

    assert!(wire_format.has_magic);
    assert_eq!(decoded.event_id, event.event_id);
    assert_eq!(decoded.symbol, event.symbol);
    assert_eq!(decoded.trace_id, event.trace_id);
    assert_eq!(decoded.model_id, event.model_id);
    assert_eq!(decoded.outputs.forecast, event.outputs.forecast);
    assert_eq!(decoded.outputs.confidence, event.outputs.confidence);
}

#[test]
fn encode_decode_trade_intent_roundtrip() {
    let intent = make_intent();
    let encoded = encode_trade_intent(&intent).expect("Failed to encode");

    let (decoded, wire_format) = decode_trade_intent(&encoded).expect("Failed to decode");

    assert!(wire_format.has_magic);
    assert_eq!(decoded.symbol, intent.symbol);
    assert_eq!(decoded.trace_id, intent.trace_id);
    assert_eq!(decoded.model_id, intent.model_id);
    assert_eq!(decoded.model_version, intent.model_version);
    assert_eq!(decoded.strategy_id, intent.strategy_id);
    assert_eq!(decoded.sequence_number, intent.sequence_number);
}

#[test]
fn trade_intent_has_valid_uuid() {
    let intent = make_intent();
    assert!(!intent.intent_id.is_empty());
    assert!(intent.intent_id.len() > 10); // UUID v4 is 36 chars
}

#[test]
fn trade_intent_propagates_inference_fields() {
    let inference = make_inference();
    let intent = TradeIntent::new(
        &inference,
        strategy_service::strategy_core::domain::trade_intent::IntentSide::Long,
        strategy_service::strategy_core::domain::trade_intent::SizeHint::Units(100.0),
        strategy_service::strategy_core::domain::trade_intent::IntentType::Entry,
        strategy_service::strategy_core::domain::trade_intent::IntentUrgency::Normal,
        "strategy-v1",
        "1.0.0",
        1,
        None,
    );

    assert_eq!(intent.symbol, inference.symbol);
    assert_eq!(intent.trace_id, inference.trace_id);
    assert_eq!(intent.model_id, inference.model_id);
    assert_eq!(intent.model_version, inference.model_version);
    assert_eq!(intent.timestamp_ns, inference.timestamp_ns);
}

#[test]
fn trade_intent_generated_ns_is_recent() {
    let intent = make_intent();
    let now = current_time_ns();
    let diff = now.saturating_sub(intent.generated_ns);
    assert!(diff < 1_000_000_000); // within 1 second
}
