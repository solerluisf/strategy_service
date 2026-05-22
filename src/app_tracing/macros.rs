#[macro_export]
macro_rules! trace_inference {
    ($event:expr) => {
        tracing::trace!(
            symbol = %$event.symbol,
            event_id = %$event.event_id,
            model_id = %$event.model_id,
            "Inference event received"
        );
    };
}

#[macro_export]
macro_rules! trace_intent {
    ($intent:expr) => {
        tracing::trace!(
            intent_id = %$intent.intent_id,
            symbol = %$intent.symbol,
            side = ?$intent.side,
            intent_type = ?$intent.intent_type,
            "Trade intent generated"
        );
    };
}

#[macro_export]
macro_rules! trace_suppression {
    ($symbol:expr, $reason:expr) => {
        tracing::debug!(
            symbol = %$symbol,
            reason = %$reason,
            "Signal suppressed"
        );
    };
}
