use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;

use log::warn;

use crate::http::response::json_error;
use crate::metrics::RouterMetrics;
use crate::types::ServiceRequest;

pub type HandlerFuture = Pin<Box<dyn Future<Output = serde_json::Value> + Send>>;
pub type HandlerFn<Ctx> = Arc<dyn Fn(Ctx, ServiceRequest) -> HandlerFuture + Send + Sync>;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Access {
    Public,
    Internal,
    Any,
}

struct Route<Ctx> {
    handler: HandlerFn<Ctx>,
    access: Access,
}

#[derive(Clone)]
enum PatternPart {
    Literal(String),
    Wildcard,
    WildcardRest,
}

#[derive(Clone)]
struct PatternRoute<Ctx> {
    method: String,
    pattern: String,
    parts: Vec<PatternPart>,
    route: Arc<Route<Ctx>>,
}

#[derive(Clone)]
pub struct Router<Ctx> {
    routes: HashMap<(String, String), Arc<Route<Ctx>>>,
    patterns: Vec<PatternRoute<Ctx>>,
    metrics: Arc<RouterMetrics>,
}

impl<Ctx> Router<Ctx> {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
            patterns: Vec::new(),
            metrics: Arc::new(RouterMetrics::new()),
        }
    }

    pub fn metrics(&self) -> Arc<RouterMetrics> {
        Arc::clone(&self.metrics)
    }

    pub fn with_metrics(mut self, metrics: Arc<RouterMetrics>) -> Self {
        self.metrics = metrics;
        self
    }

    pub async fn start_metrics_server(&self, port: Option<u16>) -> Option<u16> {
        crate::metrics::http::start_metrics_server(Arc::clone(&self.metrics), port).await
    }

    pub fn register<F, Fut>(&mut self, method: &str, action: &str, handler: F)
    where
        F: Fn(Ctx, ServiceRequest) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = serde_json::Value> + Send + 'static,
    {
        self.register_with_access(method, action, Access::Any, handler);
    }

    pub fn register_public<F, Fut>(&mut self, method: &str, action: &str, handler: F)
    where
        F: Fn(Ctx, ServiceRequest) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = serde_json::Value> + Send + 'static,
    {
        self.register_with_access(method, action, Access::Public, handler);
    }

    pub fn register_internal<F, Fut>(&mut self, method: &str, action: &str, handler: F)
    where
        F: Fn(Ctx, ServiceRequest) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = serde_json::Value> + Send + 'static,
    {
        self.register_with_access(method, action, Access::Internal, handler);
    }

    fn register_with_access<F, Fut>(
        &mut self,
        method: &str,
        action: &str,
        access: Access,
        handler: F,
    ) where
        F: Fn(Ctx, ServiceRequest) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = serde_json::Value> + Send + 'static,
    {
        let method_upper = method.to_uppercase();
        let route = Arc::new(Route {
            handler: Arc::new(move |ctx, req| Box::pin(handler(ctx, req))),
            access,
        });

        if action.contains('*') {
            self.patterns.push(PatternRoute {
                method: method_upper,
                pattern: action.to_string(),
                parts: parse_pattern(action),
                route,
            });
        } else {
            self.routes
                .insert((method_upper, action.to_string()), route);
        }
    }

    pub async fn route(&self, ctx: Ctx, request: ServiceRequest) -> serde_json::Value {
        let started = Instant::now();
        let method = request.method.to_uppercase();
        let key = (method.clone(), request.action.clone());

        let (handler, label) = match self.routes.get(&key) {
            Some(route) => (Some(route.clone()), key),
            None => {
                let segments: Vec<&str> = request.action.split('/').collect();

                let pattern = self
                    .patterns
                    .iter()
                    .filter(|p| p.method == method)
                    .filter_map(|p| {
                        if matches_pattern(&p.parts, &segments) {
                            Some((p.route.clone(), p.pattern.clone(), score_pattern(&p.parts)))
                        } else {
                            None
                        }
                    })
                    .max_by_key(|(_, _, score)| *score)
                    .map(|(route, pattern, _)| (route, pattern));

                match pattern {
                    Some((route, pattern)) => (Some(route), (method, pattern)),
                    None => (None, key),
                }
            }
        };

        let metric_label = if handler.is_some() {
            format!("{} {}", label.0, label.1)
        } else {
            "unmatched".to_string()
        };

        let response = match handler {
            Some(route) => {
                let allowed = match route.access {
                    Access::Any => true,
                    Access::Public => !request.internal,
                    Access::Internal => request.internal,
                };

                if !allowed {
                    warn!(
                        "Forbidden route access for request id={}: {} {} (internal={}, access={:?})",
                        request.id, request.method, request.action, request.internal, route.access
                    );
                    json_error(403, "Forbidden")
                } else {
                    (route.handler)(ctx, request).await
                }
            }
            None => {
                warn!(
                    "Unknown route for request id={}: {} {}",
                    request.id, request.method, request.action
                );
                json_error(404, "Unknown route")
            }
        };

        let status = response
            .get("status")
            .and_then(|v| v.as_u64())
            .map(|s| s as u16)
            .unwrap_or(200);

        let latency_us = started.elapsed().as_micros().min(u64::MAX as u128) as u64;
        self.metrics.record(&metric_label, status, latency_us);

        response
    }
}

fn parse_pattern(action: &str) -> Vec<PatternPart> {
    let segments: Vec<&str> = action.split('/').collect();
    let mut parts = Vec::with_capacity(segments.len());

    for (i, seg) in segments.iter().enumerate() {
        if *seg == "*" {
            if i == segments.len() - 1 {
                parts.push(PatternPart::WildcardRest);
            } else {
                parts.push(PatternPart::Wildcard);
            }
        } else {
            parts.push(PatternPart::Literal((*seg).to_string()));
        }
    }

    parts
}

fn matches_pattern(pattern: &[PatternPart], action: &[&str]) -> bool {
    let mut i = 0;
    let n = action.len();

    for part in pattern {
        match part {
            PatternPart::Literal(lit) => {
                if i >= n || action[i] != lit.as_str() {
                    return false;
                }
                i += 1;
            }
            PatternPart::Wildcard => {
                if i >= n {
                    return false;
                }
                i += 1;
            }
            PatternPart::WildcardRest => return i < n,
        }
    }

    i == n
}

fn score_pattern(pattern: &[PatternPart]) -> usize {
    pattern
        .iter()
        .filter(|p| matches!(p, PatternPart::Literal(_)))
        .count()
}

impl<Ctx> Default for Router<Ctx> {
    fn default() -> Self {
        Self::new()
    }
}
