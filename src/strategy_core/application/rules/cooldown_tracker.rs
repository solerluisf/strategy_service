use std::collections::HashMap;

use crate::strategy_core::domain::strategy_config::StrategyConfig;
use crate::strategy_core::domain::trade_intent::IntentType;

pub struct CooldownTracker {
    last_emitted: HashMap<String, HashMap<String, u64>>,
}

pub enum CooldownResult {
    Ready,
    Suppressed { remaining_ms: u64 },
}

impl CooldownTracker {
    pub fn new() -> Self {
        Self {
            last_emitted: HashMap::new(),
        }
    }

    pub fn check(
        &self,
        symbol: &str,
        intent_type: &IntentType,
        now_ns: u64,
        config: &StrategyConfig,
    ) -> CooldownResult {
        let cooldown_ms = match intent_type {
            IntentType::Entry | IntentType::ScaleIn => config.entry_cooldown_ms,
            IntentType::Exit | IntentType::ScaleOut => config.exit_cooldown_ms,
        };
        let cooldown_ns = cooldown_ms * 1_000_000;

        let key = intent_type_key(intent_type);
        if let Some(symbol_cooldowns) = self.last_emitted.get(symbol) {
            if let Some(&last_ns) = symbol_cooldowns.get(&key) {
                let elapsed = now_ns.saturating_sub(last_ns);
                if elapsed < cooldown_ns {
                    let remaining_ms = (cooldown_ns - elapsed) / 1_000_000;
                    return CooldownResult::Suppressed { remaining_ms };
                }
            }
        }

        CooldownResult::Ready
    }

    pub fn mark_emitted(&mut self, symbol: &str, intent_type: &IntentType, now_ns: u64) {
        let key = intent_type_key(intent_type);
        self.last_emitted
            .entry(symbol.to_string())
            .or_default()
            .insert(key, now_ns);
    }

    pub fn reset(&mut self, symbol: &str) {
        self.last_emitted.remove(symbol);
    }
}

fn intent_type_key(intent_type: &IntentType) -> String {
    match intent_type {
        IntentType::Entry => "entry".to_string(),
        IntentType::Exit => "exit".to_string(),
        IntentType::ScaleIn => "scale_in".to_string(),
        IntentType::ScaleOut => "scale_out".to_string(),
    }
}

impl Default for CooldownTracker {
    fn default() -> Self {
        Self::new()
    }
}
