use crate::config::service::ServiceConfig;

use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryRequest {
    pub service_name: String,
    pub service_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryResponse {
    pub instance_id: String,
    pub metrics_port: u16,
    pub status: String,
}

pub async fn discover_metrics_manager(config: &mut ServiceConfig, timeout_millis: u64) {
    let manager_url = if let Some(url) = config.metrics_manager_url.as_deref() {
        if url.is_empty() {
            warn!("[Discovery] METRICS_MANAGER_URL is an empty string");
            config.metrics_fallback();
            return;
        }
        url
    } else {
        warn!("[Discovery] METRICS_MANAGER_URL is None - using fallback config");
        config.metrics_fallback();
        return;
    };

    let request = DiscoveryRequest {
        service_name: config.service_name.clone(),
        service_type: "auth-service".to_string(),
    };

    let url = format!("{}/discover", manager_url);
    let client = reqwest::Client::new();

    info!("[Discovery] Pinging Metrics Manager at: {}", url);

    let result = tokio::time::timeout(
        Duration::from_millis(timeout_millis),
        client.post(&url).json(&request).send(),
    )
    .await;

    match result {
        Ok(Ok(response)) if response.status().is_success() => {
            match response.json::<DiscoveryResponse>().await {
                Ok(discovery_response) => {
                    info!("[Discovery] Connected to Metrics Manager successfully");

                    config.instance_id = Some(discovery_response.instance_id);
                    config.metrics_port = Some(discovery_response.metrics_port);
                    config.metrics_manager_connected = true;
                }
                Err(e) => {
                    warn!("[Discovery] Failed to parse response: {}", e);
                    config.metrics_fallback();
                }
            }
        }
        Ok(Ok(response)) => {
            warn!("[Discovery] Metrics Manager error: {}", response.status());
            config.metrics_fallback();
        }
        Ok(Err(e)) => {
            warn!("[Discovery] Connection failed: {}", e);
            config.metrics_fallback();
        }
        Err(_) => {
            warn!("[Discovery] Timeout ({}s)", timeout_millis as f64 / 1000.0);
            config.metrics_fallback();
        }
    }
}
