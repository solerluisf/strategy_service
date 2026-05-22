use strategy_service::strategy_core::domain::order_lifecycle::{OrderLifecycleEvent, OrderLifecycleEventType};
use strategy_service::strategy_core::domain::position::PositionCache;

fn make_lifecycle_event(symbol: &str, event_type: OrderLifecycleEventType, side: &str, qty: f64) -> OrderLifecycleEvent {
    OrderLifecycleEvent {
        event_id: format!("evt-{}", symbol),
        execution_id: format!("exec-{}", symbol),
        client_order_id: Some(format!("order-{}", symbol)),
        symbol: symbol.to_string(),
        event_type,
        timestamp: "2024-01-01T00:00:00Z".to_string(),
        payload: serde_json::json!({
            "side": side,
            "filled_qty": qty,
            "filled_price": 100.0,
        }),
    }
}

#[test]
fn filled_buy_event_increases_net_units() {
    let cache = PositionCache::new();

    let event = make_lifecycle_event("AAPL", OrderLifecycleEventType::Filled, "buy", 100.0);
    cache.apply_lifecycle_event(&event);

    let position = cache.get("AAPL");
    assert!((position.net_units - 100.0).abs() < 1e-9);
}

#[test]
fn partial_fill_increases_net_units_partially() {
    let cache = PositionCache::new();

    let event1 = make_lifecycle_event("AAPL", OrderLifecycleEventType::PartialFill, "buy", 50.0);
    cache.apply_lifecycle_event(&event1);

    let event2 = make_lifecycle_event("AAPL", OrderLifecycleEventType::PartialFill, "buy", 30.0);
    cache.apply_lifecycle_event(&event2);

    let position = cache.get("AAPL");
    assert!((position.net_units - 80.0).abs() < 1e-9);
}

#[test]
fn cancelled_event_does_not_change_position() {
    let cache = PositionCache::new();

    let event = make_lifecycle_event("AAPL", OrderLifecycleEventType::Cancelled, "buy", 100.0);
    cache.apply_lifecycle_event(&event);

    let position = cache.get("AAPL");
    assert!(position.is_flat());
}

#[test]
fn sell_fill_reduces_long_position() {
    let cache = PositionCache::new();

    let buy = make_lifecycle_event("AAPL", OrderLifecycleEventType::Filled, "buy", 100.0);
    cache.apply_lifecycle_event(&buy);

    let sell = make_lifecycle_event("AAPL", OrderLifecycleEventType::Filled, "sell", 60.0);
    cache.apply_lifecycle_event(&sell);

    let position = cache.get("AAPL");
    assert!((position.net_units - 40.0).abs() < 1e-9);
}

#[test]
fn fill_to_close_returns_to_flat() {
    let cache = PositionCache::new();

    let buy = make_lifecycle_event("AAPL", OrderLifecycleEventType::Filled, "buy", 100.0);
    cache.apply_lifecycle_event(&buy);

    let sell = make_lifecycle_event("AAPL", OrderLifecycleEventType::Filled, "sell", 100.0);
    cache.apply_lifecycle_event(&sell);

    let position = cache.get("AAPL");
    assert!(position.is_flat());
}

#[test]
fn get_all_returns_tracked_positions() {
    let cache = PositionCache::new();

    let event1 = make_lifecycle_event("AAPL", OrderLifecycleEventType::Filled, "buy", 50.0);
    cache.apply_lifecycle_event(&event1);

    let event2 = make_lifecycle_event("MSFT", OrderLifecycleEventType::Filled, "buy", 30.0);
    cache.apply_lifecycle_event(&event2);

    let all = cache.get_all();
    assert_eq!(all.len(), 2);
}

#[test]
fn unknown_symbol_returns_flat_position() {
    let cache = PositionCache::new();

    let position = cache.get("UNKNOWN");
    assert!(position.is_flat());
    assert_eq!(position.symbol, "UNKNOWN");
}
