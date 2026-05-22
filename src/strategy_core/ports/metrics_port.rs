use async_trait::async_trait;

#[async_trait]
pub trait IMetricsPort: Send + Sync {
    fn increment_counter(&self, name: &str, labels: &[(&str, &str)]);
    fn record_histogram(&self, name: &str, value: f64, labels: &[(&str, &str)]);
    fn set_gauge(&self, name: &str, value: f64, labels: &[(&str, &str)]);
}
