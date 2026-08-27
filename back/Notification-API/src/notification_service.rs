use std::sync::Arc;

use api_core::sse::SseConnectionManager;

use crate::event::{NotificationEvent, NotificationMetadata, NotificationTarget};

pub type NotificationManager =
    Arc<SseConnectionManager<NotificationEvent, NotificationMetadata, NotificationTarget>>;

pub struct NotificationService {
    manager: NotificationManager,
}

impl NotificationService {
    pub fn new(
        manager: Arc<
            SseConnectionManager<NotificationEvent, NotificationMetadata, NotificationTarget>,
        >,
    ) -> Self {
        Self { manager }
    }

    pub fn manager(&self) -> NotificationManager {
        Arc::clone(&self.manager)
    }
}
