use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SseEnvelope<E, T> {
    pub target: T,
    pub event: E,
}
