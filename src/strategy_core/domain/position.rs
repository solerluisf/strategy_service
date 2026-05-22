use std::collections::HashMap;
use std::sync::RwLock;

use crate::strategy_core::domain::order_lifecycle::OrderLifecycleEvent;

#[derive(Debug, Clone, Default)]
pub struct PositionState {
    pub symbol: String,
    pub net_units: f64,
    pub avg_entry_price: f64,
    pub last_updated_ns: u64,
}

impl PositionState {
    pub fn is_flat(&self) -> bool {
        self.net_units.abs() < 1e-9
    }

    pub fn is_long(&self) -> bool {
        self.net_units > 1e-9
    }

    pub fn is_short(&self) -> bool {
        self.net_units < -1e-9
    }
}

pub struct PositionCache {
    positions: RwLock<HashMap<String, PositionState>>,
}

impl PositionCache {
    pub fn new() -> Self {
        Self {
            positions: RwLock::new(HashMap::new()),
        }
    }

    pub fn apply_lifecycle_event(&self, event: &OrderLifecycleEvent) {
        match event.event_type {
            crate::strategy_core::domain::order_lifecycle::OrderLifecycleEventType::Filled
            | crate::strategy_core::domain::order_lifecycle::OrderLifecycleEventType::PartialFill => {
                let filled_qty = event.payload.get("filled_qty").and_then(|v| v.as_f64());
                let side = event.payload.get("side").and_then(|v| v.as_str());

                if let (Some(qty), Some(side)) = (filled_qty, side) {
                    let mut positions = self.positions.write().unwrap();
                    let state = positions
                        .entry(event.symbol.clone())
                        .or_insert_with(|| PositionState {
                            symbol: event.symbol.clone(),
                            net_units: 0.0,
                            avg_entry_price: 0.0,
                            last_updated_ns: 0,
                        });

                    let now_ns = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos() as u64;

                    match side {
                        "buy" | "Buy" | "BUY" => {
                            state.net_units += qty;
                        }
                        "sell" | "Sell" | "SELL" => {
                            state.net_units -= qty;
                        }
                        _ => {
                            tracing::warn!(side = %side, "Unknown order side in lifecycle event");
                        }
                    }
                    state.last_updated_ns = now_ns;
                } else {
                    tracing::warn!(
                        event_id = %event.event_id,
                        "Missing filled_qty or side in lifecycle event payload"
                    );
                }
            }
            _ => {}
        }
    }

    pub fn get(&self, symbol: &str) -> PositionState {
        let positions = self.positions.read().unwrap();
        positions
            .get(symbol)
            .cloned()
            .unwrap_or_else(|| PositionState {
                symbol: symbol.to_string(),
                net_units: 0.0,
                avg_entry_price: 0.0,
                last_updated_ns: 0,
            })
    }

    pub fn get_all(&self) -> Vec<PositionState> {
        let positions = self.positions.read().unwrap();
        positions.values().cloned().collect()
    }
}

impl Default for PositionCache {
    fn default() -> Self {
        Self::new()
    }
}
