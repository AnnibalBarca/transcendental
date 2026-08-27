use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{config::ServiceConfig, metrics::app_metrics::AppMetrics};

#[derive(Clone)]
pub struct AppState {
    pub metrics: Arc<AppMetrics>,
    pub http_client: Client,
    pub request_router: crate::http::router::Router,
    pub redis_pool: deadpool_redis::Pool,
    pub config: Arc<ServiceConfig>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ErrorResponse {
    pub error: String,
    pub status: u16,
}
