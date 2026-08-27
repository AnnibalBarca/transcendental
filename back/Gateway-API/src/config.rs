use log::{error, warn};
use rand::Rng;
use std::env;

use crate::port_utils::{generate_random_port, is_port_free};

#[derive(Clone)]
pub struct ServiceConfig {
    pub service_name: String,
    pub instance_id: Option<String>,
    pub api_port: Option<u16>,
    pub metrics_port: Option<u16>,
    pub metrics_manager_url: Option<String>,
    pub metrics_manager_connected: bool,
    pub redis_url: Option<String>,
    pub auth_api_url: Option<String>,
    pub user_api_url: Option<String>,
    pub game_api_url: Option<String>,
    pub notification_api_url: Option<String>,
}

impl ServiceConfig {
    pub fn from_env() -> Self {
        Self {
            service_name: env::var("SERVICE_NAME").unwrap_or_else(|_| "gateway".to_string()),
            api_port: Self::load_option_port("API_PORT"),
            metrics_port: None,
            metrics_manager_url: Self::load_option_url("METRICS_MANAGER_URL"),
            redis_url: Self::load_option_url("REDIS_URL"),
            auth_api_url: Self::load_option_url("AUTH_API_URL"),
            user_api_url: Self::load_option_url("USER_API_URL"),
            game_api_url: Self::load_option_url("CHESS_API_URL"),
            notification_api_url: Self::load_option_url("NOTIFICATION_API_URL"),
            instance_id: None,
            metrics_manager_connected: false,
        }
    }

    fn load_option_port(name: &str) -> Option<u16> {
        match env::var(name).ok().and_then(|s| s.parse().ok()) {
            Some(port) => Some(port),
            None => {
                warn!("[Config] {} is missing or invalid.", name);
                None
            }
        }
    }

    fn load_option_url(name: &str) -> Option<String> {
        match env::var(name).ok() {
            Some(url) => Some(url),
            None => {
                warn!("[Config] {} not found.", name);
                None
            }
        }
    }

    fn generate_random_id() -> String {
        use rand::distributions::Alphanumeric;
        let random_id: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(12)
            .map(char::from)
            .collect();
        format!("gateway-{}", random_id)
    }

    pub fn metrics_fallback(&mut self) {
        let instance_id = Self::generate_random_id();
        let mut m_port = generate_random_port();
        let mut attempts = 0;
        let max_attempts = 100;

        while !is_port_free(m_port) && attempts < max_attempts {
            m_port = generate_random_port();
            attempts += 1;
        }

        self.metrics_manager_connected = false;

        if attempts == max_attempts {
            error!("[Discovery] Failed to init metrics fallback - disabling metrics");
            self.metrics_port = None;
            return;
        }

        warn!(
            "[Discovery] Fallback: Generated Instance ID: {}",
            instance_id
        );
        warn!("[Discovery] Fallback: Using Port: {}", m_port);

        self.instance_id = Some(instance_id);
        self.metrics_port = Some(m_port);
    }
}
