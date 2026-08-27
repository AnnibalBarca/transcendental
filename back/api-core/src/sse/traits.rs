use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Debug;

pub trait SseEvent: Serialize + DeserializeOwned + Clone + Send + Sync + 'static + Debug {}

impl<T> SseEvent for T where T: Serialize + DeserializeOwned + Clone + Send + Sync + 'static + Debug {}

pub trait ConnectionMetadata: Clone + Send + Sync + 'static + Debug {
    fn routing_keys(&self) -> Vec<String>;
}

pub trait SseTarget<M: ConnectionMetadata>:
    Serialize + DeserializeOwned + Clone + Send + Sync + 'static + Debug
{
    fn routing_keys(&self) -> Vec<String>;
    fn matches(&self, metadata: &M) -> bool;
}

pub trait SseEventWithMetadata<M: ConnectionMetadata>: SseEvent {
    fn metadata_update(&self) -> Option<M> {
        None
    }
}
