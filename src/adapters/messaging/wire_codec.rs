use serde::{Deserialize, Serialize};

pub const BGW1_MAGIC: &[u8] = &[0x42, 0x47, 0x57, 0x31]; // "BGW1"

pub struct WireFormat {
    pub has_magic: bool,
    pub payload_len: usize,
}

pub fn encode_msgpack<T: Serialize>(value: &T) -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    buf.extend_from_slice(BGW1_MAGIC);
    rmp_serde::encode::write(&mut buf, value)
        .map_err(|e| format!("MessagePack encode error: {}", e))?;
    Ok(buf)
}

pub fn decode_msgpack<T: for<'de> Deserialize<'de>>(bytes: &[u8]) -> Result<(T, WireFormat), String> {
    if bytes.len() < 4 {
        return Err("Payload too short for BGW1 magic".to_string());
    }

    let has_magic = &bytes[0..4] == BGW1_MAGIC;
    let payload = if has_magic {
        &bytes[4..]
    } else {
        bytes
    };

    let payload_len = payload.len();
    let value: T = rmp_serde::decode::from_slice(payload)
        .map_err(|e| format!("MessagePack decode error: {}", e))?;

    Ok((value, WireFormat { has_magic, payload_len }))
}

pub fn encode_trade_intent(intent: &crate::strategy_core::domain::trade_intent::TradeIntent) -> Result<Vec<u8>, String> {
    encode_msgpack(intent)
}

pub fn decode_trade_intent(bytes: &[u8]) -> Result<(crate::strategy_core::domain::trade_intent::TradeIntent, WireFormat), String> {
    decode_msgpack(bytes)
}

pub fn decode_inference_event(bytes: &[u8]) -> Result<(crate::strategy_core::domain::inference_input::InferenceEvent, WireFormat), String> {
    decode_msgpack(bytes)
}

pub fn decode_order_lifecycle_event(bytes: &[u8]) -> Result<(crate::strategy_core::domain::order_lifecycle::OrderLifecycleEvent, WireFormat), String> {
    decode_msgpack(bytes)
}
