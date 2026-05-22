use crate::strategy_core::domain::strategy_config::StrategyConfig;

pub struct ThresholdGate;

pub enum ThresholdDecision {
    Long,
    Short,
    Exit,
    NoSignal,
}

impl ThresholdGate {
    pub fn evaluate(
        forecast: f64,
        confidence: f64,
        config: &StrategyConfig,
    ) -> ThresholdDecision {
        if confidence < config.confidence_minimum {
            return ThresholdDecision::NoSignal;
        }
        if forecast >= config.long_entry_threshold {
            return ThresholdDecision::Long;
        }
        if forecast <= config.short_entry_threshold {
            return ThresholdDecision::Short;
        }
        if forecast.abs() < config.exit_threshold {
            return ThresholdDecision::Exit;
        }
        ThresholdDecision::NoSignal
    }
}
