use dashmap::DashMap;
use hdrhistogram::Histogram;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

pub struct AppMetrics {
    pub total_requests: AtomicU64,
    pub total_requests_success: AtomicU64,
    pub total_requests_failure: AtomicU64,
    pub global_latency: Mutex<Histogram<u64>>,
    pub scopes: DashMap<String, Arc<ScopeMetrics>>,
}

pub struct ScopeMetrics {
    pub count: AtomicU64,
    pub errors: AtomicU64,
    pub latency: Mutex<Histogram<u64>>,
}

#[derive(Serialize)]
pub struct ScopeSnapshot {
    pub count: u64,
    pub errors: u64,
    pub p95_ms: u64,
    pub p99_ms: u64,
}

#[derive(Serialize)]
pub struct MetricsSnapshot {
    pub total_requests: u64,
    pub total_errors: u64,
    pub global_p95_ms: u64,
    pub global_p99_ms: u64,
    pub scopes: HashMap<String, ScopeSnapshot>,
}

impl AppMetrics {
    pub fn new() -> Self {
        Self {
            total_requests: AtomicU64::new(0),
            total_requests_success: AtomicU64::new(0),
            total_requests_failure: AtomicU64::new(0),
            global_latency: Mutex::new(Histogram::new_with_bounds(1, 60_000, 3).unwrap()),
            scopes: DashMap::new(),
        }
    }

    pub fn record(&self, scope_name: &str, latency_ms: u64, is_error: bool) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        if is_error {
            self.total_requests_failure.fetch_add(1, Ordering::Relaxed);
        } else {
            self.total_requests_success.fetch_add(1, Ordering::Relaxed);
        }

        if let Ok(mut g) = self.global_latency.lock() {
            let _ = g.record(latency_ms);
        }

        if let Some(scope_ref) = self.scopes.get_mut(scope_name) {
            scope_ref.count.fetch_add(1, Ordering::Relaxed);
            if is_error {
                scope_ref.errors.fetch_add(1, Ordering::Relaxed);
            }

            if let Ok(mut l) = scope_ref.latency.lock() {
                let _ = l.record(latency_ms);
            }
        } else {
            let new_scope = Arc::new(ScopeMetrics {
                count: AtomicU64::new(1),
                errors: if is_error {
                    AtomicU64::new(1)
                } else {
                    AtomicU64::new(0)
                },
                latency: Mutex::new(Histogram::new_with_bounds(1, 60_000, 3).unwrap()),
            });

            if is_error {
                new_scope.errors.fetch_add(1, Ordering::Relaxed);
            }

            if let Ok(mut l) = new_scope.latency.lock() {
                let _ = l.record(latency_ms);
            }

            self.scopes.insert(scope_name.to_string(), new_scope);
        }
    }

    pub fn get_snapshot(&self) -> MetricsSnapshot {
        let mut scopes_map = HashMap::new();

        for entry in self.scopes.iter() {
            let name = entry.key().clone();
            let s = entry.value();
            if let Ok(l) = s.latency.lock() {
                scopes_map.insert(
                    name,
                    ScopeSnapshot {
                        count: s.count.load(Ordering::Relaxed),
                        errors: s.errors.load(Ordering::Relaxed),
                        p95_ms: l.value_at_quantile(0.95),
                        p99_ms: l.value_at_quantile(0.99),
                    },
                );
            }
        }

        let (g95, g99) = if let Ok(l) = self.global_latency.lock() {
            (l.value_at_quantile(0.95), l.value_at_quantile(0.99))
        } else {
            (0, 0)
        };

        MetricsSnapshot {
            total_requests: self.total_requests.load(Ordering::Relaxed),
            total_errors: self.total_requests_failure.load(Ordering::Relaxed),
            global_p95_ms: g95,
            global_p99_ms: g99,
            scopes: scopes_map,
        }
    }

    pub fn gather(&self) -> String {
        let snap = self.get_snapshot();
        let mut output = String::new();

        output.push_str(&format!("auth_requests_total {}\n", snap.total_requests));
        output.push_str(&format!("auth_errors_total {}\n", snap.total_errors));
        output.push_str(&format!("auth_latency_p95_ms {}\n", snap.global_p95_ms));
        output.push_str(&format!("auth_latency_p99_ms {}\n", snap.global_p99_ms));

        for (name, s) in snap.scopes {
            output.push_str(&format!(
                "auth_service_requests_total{{service=\"{}\"}} {}\n",
                name, s.count
            ));
            output.push_str(&format!(
                "auth_service_errors_total{{service=\"{}\"}} {}\n",
                name, s.errors
            ));
            output.push_str(&format!(
                "auth_service_latency_p95_ms{{service=\"{}\"}} {}\n",
                name, s.p95_ms
            ));
            output.push_str(&format!(
                "auth_service_latency_p99_ms{{service=\"{}\"}} {}\n",
                name, s.p99_ms
            ));
        }

        output
    }
}
