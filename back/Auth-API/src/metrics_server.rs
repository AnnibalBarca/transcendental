use crate::{
    config::service::ServiceConfig, metrics::app_metrics::AppMetrics, port_utils::find_free_port,
};
use log::{info, warn};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub fn get_metrics_port(config: &mut ServiceConfig) -> Option<u16> {
    let port: Option<u16>;

    if config.metrics_manager_connected == false {
        if let Some(p) = config.metrics_port {
            port = Some(p);
        } else {
            port = find_free_port();
        }

        if let Some(p) = port {
            config.metrics_port = Some(p);
        } else {
            warn!("[Auth Metrics] No free port found");
            config.metrics_port = None;
            config.metrics_manager_connected = false;
            return None;
        }
    } else {
        port = config.metrics_port;
        if port.is_none() {
            warn!("[Auth Metrics] No metrics port set");
            return None;
        }
    }
    return port;
}

pub async fn start_metrics_server(metrics: Arc<AppMetrics>, config: &mut ServiceConfig) -> bool {
    use tokio::net::TcpListener;

    let Some(port) = get_metrics_port(config) else {
        return false;
    };

    let addr = format!("0.0.0.0:{}", port);

    let listener = match TcpListener::bind(&format!("0.0.0.0:{}", port)).await {
        Ok(l) => {
            info!("[Auth Metrics] Server started on {}", addr);
            l
        }
        Err(e) => {
            warn!("[Auth Metrics] Failed to bind: {}", e);
            return false;
        }
    };

    info!("[Auth] Metrics server started on {}", addr);

    loop {
        match listener.accept().await {
            Ok((socket, _addr)) => {
                let metrics = Arc::clone(&metrics);
                tokio::spawn(async move {
                    handle_auth_metrics_request(socket, metrics).await;
                });
            }
            Err(e) => warn!("[Auth Metrics] Accept error: {}", e),
        }
    }
}

async fn handle_auth_metrics_request(mut socket: tokio::net::TcpStream, metrics: Arc<AppMetrics>) {
    let mut request = String::new();
    let mut buf = vec![0u8; 4096];
    loop {
        let n = socket.read(&mut buf).await.unwrap_or(0);

        if n == 0 {
            warn!("[Auth] Connection closed by client prematurely");
            return;
        }

        request.push_str(&String::from_utf8_lossy(&buf[..n]));

        if request.contains("\r\n\r\n") {
            break;
        }

        if request.len() > 8192 {
            warn!("[Auth] Metrics request too large");
            let _ = socket
                .write_all(b"HTTP/1.1 431 Request Header Fields Too Large\r\n\r\n")
                .await;
            return;
        }
    }

    if request.is_empty() {
        return;
    }

    if request.contains("GET /metrics") {
        handle_get_request(&mut socket, metrics).await;
    } else if request.contains("GET /health") {
        handle_health_request(&mut socket).await;
    } else {
        let response = "HTTP/1.1 404 Not Found\r\n\r\n";
        let _ = socket.write_all(response.as_bytes()).await;
    }
}

async fn handle_get_request(socket: &mut tokio::net::TcpStream, metrics: Arc<AppMetrics>) {
    let data = metrics.gather();
    let response = format!(
        "HTTP/1.1 200 OK\r\n\
        Content-Type: text/plain; charset=utf-8\r\n\
        Content-Length: {}\r\n\
        Connection: close\r\n\
        \r\n\
        {}",
        data.len(),
        data
    );
    let _ = socket.write_all(response.as_bytes()).await;
}

async fn handle_health_request(socket: &mut tokio::net::TcpStream) {
    let response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nhealthy";
    let _ = socket.write_all(response.as_bytes()).await;
}
