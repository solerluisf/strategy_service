use std::collections::HashMap;

use crate::strategy_core::domain::strategy_config::StrategyConfig;
use crate::strategy_core::application::rules::threshold_gate::ThresholdDecision;

pub struct HysteresisFilter {
    state: HashMap<String, i8>,
}

pub enum HysteresisResult {
    Allow(ThresholdDecision),
    Suppress,
}

impl HysteresisFilter {
    pub fn new() -> Self {
        Self {
            state: HashMap::new(),
        }
    }

    pub fn check(
        &mut self,
        symbol: &str,
        decision: ThresholdDecision,
        forecast: f64,
        config: &StrategyConfig,
    ) -> HysteresisResult {
        let current = self.state.get(symbol).copied().unwrap_or(0);

        let result = match (current, &decision) {
            (0, _) => HysteresisResult::Allow(decision),

            (1, ThresholdDecision::Long) => HysteresisResult::Suppress,
            (1, ThresholdDecision::Exit) => {
                if forecast.abs() < config.exit_threshold {
                    HysteresisResult::Allow(ThresholdDecision::Exit)
                } else {
                    HysteresisResult::Suppress
                }
            }
            (1, ThresholdDecision::Short) => {
                let reversal = config.short_entry_threshold - config.hysteresis_band;
                if forecast <= reversal {
                    HysteresisResult::Allow(ThresholdDecision::Short)
                } else {
                    HysteresisResult::Suppress
                }
            }

            (-1, ThresholdDecision::Short) => HysteresisResult::Suppress,
            (-1, ThresholdDecision::Exit) => {
                if forecast.abs() < config.exit_threshold {
                    HysteresisResult::Allow(ThresholdDecision::Exit)
                } else {
                    HysteresisResult::Suppress
                }
            }
            (-1, ThresholdDecision::Long) => {
                let reversal = config.long_entry_threshold + config.hysteresis_band;
                if forecast >= reversal {
                    HysteresisResult::Allow(ThresholdDecision::Long)
                } else {
                    HysteresisResult::Suppress
                }
            }

            _ => HysteresisResult::Allow(decision),
        };

        if let HysteresisResult::Allow(ref d) = result {
            let new_state = match d {
                ThresholdDecision::Long => 1,
                ThresholdDecision::Short => -1,
                ThresholdDecision::Exit => 0,
                ThresholdDecision::NoSignal => current,
            };
            self.state.insert(symbol.to_string(), new_state);
        }

        result
    }

    pub fn reset(&mut self, symbol: &str) {
        self.state.insert(symbol.to_string(), 0);
    }

    pub fn get_direction(&self, symbol: &str) -> i8 {
        self.state.get(symbol).copied().unwrap_or(0)
    }
}

impl Default for HysteresisFilter {
    fn default() -> Self {
        Self::new()
    }
}
