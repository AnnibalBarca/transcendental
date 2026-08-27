use axum::response::Response;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct Router {
    response_channels: Arc<DashMap<String, mpsc::UnboundedSender<Response>>>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            response_channels: Arc::new(DashMap::new()),
        }
    }

    pub fn register(&self, request_id: String, sender: mpsc::UnboundedSender<Response>) {
        self.response_channels.insert(request_id, sender);
    }

    pub fn send_response(
        &self,
        request_id: &str,
        response: Response,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(sender) = self.response_channels.get(request_id) {
            sender.send(response)?;
            Ok(())
        } else {
            Err(format!("Request ID {} not found in router", request_id).into())
        }
    }

    pub fn cleanup(&self, request_id: &str) {
        self.response_channels.remove(request_id);
    }
}

impl Clone for Router {
    fn clone(&self) -> Self {
        Self {
            response_channels: Arc::clone(&self.response_channels),
        }
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}
