use dashmap::DashMap;
use hdrhistogram::Histogram;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

pub mod http;

const LATENCY_MAX_US: u64 = 60_000_000;
const HISTOGRAM_SIG_FIGS: u8 = 3;

pub struct RouterMetrics {
    total_requests: AtomicU64,
    total_success: AtomicU64,
    total_errors: AtomicU64,
    global_latency_us: Mutex<Histogram<u64>>,
    routes: DashMap<String, Arc<RouteMetrics>>,
}

pub struct RouteMetrics {
    count: AtomicU64,
    errors: AtomicU64,
    latency_us: Mutex<Histogram<u64>>,
    status_codes: DashMap<u16, AtomicU64>,
}

#[derive(Serialize)]
pub struct RouteSnapshot {
    pub count: u64,
    pub errors: u64,
    pub p50_ms: u64,
    pub p95_ms: u64,
    pub p98_ms: u64,
    pub p99_ms: u64,
    pub status_codes: HashMap<u16, u64>,
}

#[derive(Serialize)]
pub struct MetricsSnapshot {
    pub total_requests: u64,
    pub total_success: u64,
    pub total_errors: u64,
    pub global_p50_ms: u64,
    pub global_p95_ms: u64,
    pub global_p98_ms: u64,
    pub global_p99_ms: u64,
    pub routes: HashMap<String, RouteSnapshot>,
}

impl RouterMetrics {
    pub fn new() -> Self {
        Self {
            total_requests: AtomicU64::new(0),
            total_success: AtomicU64::new(0),
            total_errors: AtomicU64::new(0),
            global_latency_us: Mutex::new(
                Histogram::new_with_bounds(1, LATENCY_MAX_US, HISTOGRAM_SIG_FIGS).unwrap(),
            ),
            routes: DashMap::new(),
        }
    }

    pub fn record(&self, route: &str, status: u16, latency_us: u64) {
        let is_error = status >= 400;
        let latency_us = latency_us.max(1);

        self.total_requests.fetch_add(1, Ordering::Relaxed);
        if is_error {
            self.total_errors.fetch_add(1, Ordering::Relaxed);
        } else {
            self.total_success.fetch_add(1, Ordering::Relaxed);
        }

        if let Ok(mut h) = self.global_latency_us.lock() {
            let _ = h.record(latency_us);
        }

        if let Some(entry) = self.routes.get(route) {
            entry.count.fetch_add(1, Ordering::Relaxed);
            if is_error {
                entry.errors.fetch_add(1, Ordering::Relaxed);
            }

            if let Some(code) = entry.status_codes.get(&status) {
                code.fetch_add(1, Ordering::Relaxed);
            } else {
                entry.status_codes.insert(status, AtomicU64::new(1));
            }

            if let Ok(mut h) = entry.latency_us.lock() {
                let _ = h.record(latency_us);
            }
        } else {
            let new_route = Arc::new(RouteMetrics {
                count: AtomicU64::new(1),
                errors: AtomicU64::new(u64::from(is_error)),
                latency_us: Mutex::new(
                    Histogram::new_with_bounds(1, LATENCY_MAX_US, HISTOGRAM_SIG_FIGS).unwrap(),
                ),
                status_codes: DashMap::from_iter([(status, AtomicU64::new(1))]),
            });

            if let Ok(mut h) = new_route.latency_us.lock() {
                let _ = h.record(latency_us);
            }

            self.routes.insert(route.to_string(), new_route);
        }
    }

    pub fn get_snapshot(&self) -> MetricsSnapshot {
        let mut routes_map = HashMap::new();

        for entry in self.routes.iter() {
            let name = entry.key().clone();
            let r = entry.value();

            if let Ok(l) = r.latency_us.lock() {
                let mut status_codes = HashMap::new();
                for code in r.status_codes.iter() {
                    status_codes.insert(*code.key(), code.value().load(Ordering::Relaxed));
                }

                routes_map.insert(
                    name,
                    RouteSnapshot {
                        count: r.count.load(Ordering::Relaxed),
                        errors: r.errors.load(Ordering::Relaxed),
                        p50_ms: l.value_at_quantile(0.50) / 1000,
                        p95_ms: l.value_at_quantile(0.95) / 1000,
                        p98_ms: l.value_at_quantile(0.98) / 1000,
                        p99_ms: l.value_at_quantile(0.99) / 1000,
                        status_codes,
                    },
                );
            }
        }

        let (g50, g95, g98, g99) = if let Ok(l) = self.global_latency_us.lock() {
            (
                l.value_at_quantile(0.50) / 1000,
                l.value_at_quantile(0.95) / 1000,
                l.value_at_quantile(0.98) / 1000,
                l.value_at_quantile(0.99) / 1000,
            )
        } else {
            (0, 0, 0, 0)
        };

        MetricsSnapshot {
            total_requests: self.total_requests.load(Ordering::Relaxed),
            total_success: self.total_success.load(Ordering::Relaxed),
            total_errors: self.total_errors.load(Ordering::Relaxed),
            global_p50_ms: g50,
            global_p95_ms: g95,
            global_p98_ms: g98,
            global_p99_ms: g99,
            routes: routes_map,
        }
    }

    pub fn gather(&self) -> String {
        let snap = self.get_snapshot();
        let mut output = String::new();

        output.push_str(&format!("router_requests_total {}\n", snap.total_requests));
        output.push_str(&format!("router_success_total {}\n", snap.total_success));
        output.push_str(&format!("router_errors_total {}\n", snap.total_errors));
        output.push_str(&format!("router_latency_p50_ms {}\n", snap.global_p50_ms));
        output.push_str(&format!("router_latency_p95_ms {}\n", snap.global_p95_ms));
        output.push_str(&format!("router_latency_p98_ms {}\n", snap.global_p98_ms));
        output.push_str(&format!("router_latency_p99_ms {}\n", snap.global_p99_ms));

        for (route, r) in snap.routes {
            output.push_str(&format!(
                "router_requests_total{{route=\"{}\"}} {}\n",
                route, r.count
            ));
            output.push_str(&format!(
                "router_errors_total{{route=\"{}\"}} {}\n",
                route, r.errors
            ));
            output.push_str(&format!(
                "router_latency_p50_ms{{route=\"{}\"}} {}\n",
                route, r.p50_ms
            ));
            output.push_str(&format!(
                "router_latency_p95_ms{{route=\"{}\"}} {}\n",
                route, r.p95_ms
            ));
            output.push_str(&format!(
                "router_latency_p98_ms{{route=\"{}\"}} {}\n",
                route, r.p98_ms
            ));
            output.push_str(&format!(
                "router_latency_p99_ms{{route=\"{}\"}} {}\n",
                route, r.p99_ms
            ));

            for (code, count) in r.status_codes {
                output.push_str(&format!(
                    "router_http_requests_total{{route=\"{}\",code=\"{}\"}} {}\n",
                    route, code, count
                ));
            }
        }

        output
    }
}

impl Default for RouterMetrics {
    fn default() -> Self {
        Self::new()
    }
}
