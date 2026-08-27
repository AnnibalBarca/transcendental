use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceRequest {
    pub id: String,
    pub method: String,
    pub action: String,
    pub cookies: HashMap<String, String>,
    #[allow(dead_code)]
    pub body: String,
    #[allow(dead_code)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub internal: bool,
}
