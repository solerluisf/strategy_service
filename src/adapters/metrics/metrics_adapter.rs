use std::collections::HashMap;
use std::sync::RwLock;

use async_trait::async_trait;

use crate::strategy_core::ports::metrics_port::IMetricsPort;

#[derive(Clone)]
pub struct MetricValue {
    value: f64,
    labels: Vec<(String, String)>,
}

pub struct MetricsAdapter {
    counters: RwLock<HashMap<String, Vec<MetricValue>>>,
    histograms: RwLock<HashMap<String, Vec<f64>>>,
    gauges: RwLock<HashMap<String, Vec<MetricValue>>>,
}

impl MetricsAdapter {
    pub fn new() -> Self {
        Self {
            counters: RwLock::new(HashMap::new()),
            histograms: RwLock::new(HashMap::new()),
            gauges: RwLock::new(HashMap::new()),
        }
    }

    pub fn get_counters(&self) -> HashMap<String, Vec<MetricValue>> {
        self.counters.read().unwrap().clone()
    }

    pub fn get_gauges(&self) -> HashMap<String, Vec<MetricValue>> {
        self.gauges.read().unwrap().clone()
    }
}

#[async_trait]
impl IMetricsPort for MetricsAdapter {
    fn increment_counter(&self, name: &str, labels: &[(&str, &str)]) {
        let mut counters = self.counters.write().unwrap();
        let entry = counters.entry(name.to_string()).or_default();

        let label_key: Vec<(String, String)> = labels.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();

        if let Some(existing) = entry.iter_mut().find(|e| e.labels == label_key) {
            existing.value += 1.0;
        } else {
            entry.push(MetricValue {
                value: 1.0,
                labels: label_key,
            });
        }
    }

    fn record_histogram(&self, name: &str, value: f64, _labels: &[(&str, &str)]) {
        let mut histograms = self.histograms.write().unwrap();
        histograms.entry(name.to_string()).or_default().push(value);
    }

    fn set_gauge(&self, name: &str, value: f64, labels: &[(&str, &str)]) {
        let mut gauges = self.gauges.write().unwrap();
        let entry = gauges.entry(name.to_string()).or_default();

        let label_key: Vec<(String, String)> = labels.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();

        if let Some(existing) = entry.iter_mut().find(|e| e.labels == label_key) {
            existing.value = value;
        } else {
            entry.push(MetricValue {
                value,
                labels: label_key,
            });
        }
    }
}

impl Default for MetricsAdapter {
    fn default() -> Self {
        Self::new()
    }
}
