use strategy_service::strategy_core::application::idempotency::IdempotencyStore;

#[test]
fn first_intent_id_not_seen() {
    let store = IdempotencyStore::new(60);
    assert!(!store.seen("intent-1"));
}

#[test]
fn mark_seen_makes_intent_seen() {
    let store = IdempotencyStore::new(60);
    store.mark_seen("intent-1").unwrap();
    assert!(store.seen("intent-1"));
}

#[test]
fn different_intent_ids_are_independent() {
    let store = IdempotencyStore::new(60);
    store.mark_seen("intent-1").unwrap();
    assert!(store.seen("intent-1"));
    assert!(!store.seen("intent-2"));
}

#[test]
fn cleanup_clears_all_seen_ids() {
    let store = IdempotencyStore::new(60);
    store.mark_seen("intent-1").unwrap();
    store.mark_seen("intent-2").unwrap();
    store.mark_seen("intent-3").unwrap();

    store.cleanup();

    assert!(!store.seen("intent-1"));
    assert!(!store.seen("intent-2"));
    assert!(!store.seen("intent-3"));
}

#[test]
fn mark_seen_returns_ok() {
    let store = IdempotencyStore::new(60);
    let result = store.mark_seen("intent-1");
    assert!(result.is_ok());
}
