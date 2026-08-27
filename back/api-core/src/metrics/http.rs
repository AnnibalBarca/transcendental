use crate::metrics::RouterMetrics;
use log::{info, warn};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub fn find_free_port() -> Option<u16> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..100 {
        let port: u16 = rng.gen_range(10000..60000);
        if std::net::TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return Some(port);
        }
    }
    None
}

pub async fn start_metrics_server(metrics: Arc<RouterMetrics>, port: Option<u16>) -> Option<u16> {
    use tokio::net::TcpListener;

    let port = match port {
        Some(p) => p,
        None => find_free_port()?,
    };

    let listener = match TcpListener::bind(("0.0.0.0", port)).await {
        Ok(l) => l,
        Err(e) => {
            warn!("[Router Metrics] Failed to bind on {}: {}", port, e);
            return None;
        }
    };

    let actual_port = listener.local_addr().ok()?.port();
    info!("[Router Metrics] Server started on port {}", actual_port);

    tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((socket, _addr)) => {
                    let metrics = Arc::clone(&metrics);
                    tokio::spawn(async move {
                        handle_request(socket, metrics).await;
                    });
                }
                Err(e) => warn!("[Router Metrics] Accept error: {}", e),
            }
        }
    });

    Some(actual_port)
}

async fn handle_request(mut socket: tokio::net::TcpStream, metrics: Arc<RouterMetrics>) {
    let mut request = String::new();
    let mut buf = vec![0u8; 4096];

    loop {
        let n = socket.read(&mut buf).await.unwrap_or(0);

        if n == 0 {
            return;
        }

        request.push_str(&String::from_utf8_lossy(&buf[..n]));

        if request.contains("\r\n\r\n") {
            break;
        }

        if request.len() > 8192 {
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
        handle_get_metrics(&mut socket, metrics).await;
    } else if request.contains("GET /health") {
        let _ = socket
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nhealthy")
            .await;
    } else {
        let _ = socket.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n").await;
    }
}

async fn handle_get_metrics(socket: &mut tokio::net::TcpStream, metrics: Arc<RouterMetrics>) {
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
