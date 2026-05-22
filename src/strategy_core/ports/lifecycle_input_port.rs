use async_trait::async_trait;

pub struct LifecycleMessage {
    pub topic: String,
    pub payload: Vec<u8>,
    pub received_ns: u64,
}

#[async_trait]
pub trait ILifecycleInputPort: Send + Sync {
    async fn recv(&self) -> Option<LifecycleMessage>;
}
