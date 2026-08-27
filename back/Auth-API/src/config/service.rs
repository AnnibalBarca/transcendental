use log::{error, warn};
use rand::Rng;
use std::env;

use crate::port_utils::{generate_random_port, is_port_free};

#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub service_name: String,
    pub instance_id: Option<String>,
    pub api_port: Option<u16>,
    pub metrics_port: Option<u16>,
    pub metrics_manager_url: Option<String>,
    pub metrics_manager_connected: bool,
    pub redis_url: Option<String>,
    pub database_url: Option<String>,
    pub google_client_id: Option<String>,
    pub google_client_secret: Option<String>,
    pub google_redirect_uri: String,
    pub frontend_redirect_url: String,
    pub domain_name: String,
    pub ft_client_id: Option<String>,
    pub ft_client_secret: Option<String>,
    pub ft_redirect_uri: String,
    pub domain_email: String,
}

impl ServiceConfig {
    pub fn from_env() -> Self {
        Self {
            service_name: env::var("SERVICE_NAME").unwrap_or_else(|_| "Auth-API".to_string()),
            api_port: Self::load_option_port("API_PORT"),
            metrics_port: Self::load_option_port("METRIC_PORT"),
            metrics_manager_url: Self::load_option_url("METRICS_MANAGER_URL"),
            redis_url: Self::load_option_url("REDIS_URL"),
            database_url: Self::load_option_url("DATABASE_URL"),
            google_client_id: Self::load_option_url("GOOGLE_CLIENT_ID"),
            google_client_secret: Self::load_option_url("GOOGLE_CLIENT_SECRET"),
            google_redirect_uri: env::var("GOOGLE_REDIRECT_URI")
                .unwrap_or_else(|_| "http://localhost:8000/login/google".to_string()),
            frontend_redirect_url: env::var("FRONTEND_REDIRECT_URL")
                .unwrap_or_else(|_| "/".to_string()),
            domain_name: env::var("DOMAIN_NAME")
                .ok()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "http://localhost:8000".to_string()),
            ft_client_id: Self::load_option_url("FT_CLIENT_ID"),
            ft_client_secret: Self::load_option_url("FT_CLIENT_SECRET"),
            ft_redirect_uri: env::var("FT_REDIRECT_URI")
                .unwrap_or_else(|_| "http://localhost:8000/api/auth/42/callback".to_string()),
            domain_email: env::var("DOMAIN_EMAIL")
                .ok()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "auth.yourdomain.com".to_string()),
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
        format!("auth-{}", random_id)
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
