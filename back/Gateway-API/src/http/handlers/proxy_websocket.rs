use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    http::HeaderMap,
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::time::Instant;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as TungsteniteMessage};
use tokio_tungstenite::tungstenite::handshake::client::generate_key;
use log::{error, info, warn};

use crate::http::handlers::AppState;

pub fn proxy_websocket(
    ws: WebSocketUpgrade,
    state: AppState,
    service_url: &str,
    path: &str,
    query: Option<&str>,
    headers: HeaderMap,
) -> Response {
    let ws_path = format!("/{}", path);

    let target_url = if let Some(q) = query {
        format!("{}{}?{}", service_url.replace("http", "ws"), ws_path, q)
    } else {
        format!("{}{}", service_url.replace("http", "ws"), ws_path)
    };

    info!("[Gateway] Upgrading WebSocket connection to {}", target_url);

    let cookie = headers
        .get(axum::http::header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    ws.on_upgrade(move |client_socket| handle_websocket_proxy(client_socket, target_url, cookie, state))
}

async fn handle_websocket_proxy(client_socket: WebSocket, target_url: String, cookie: Option<String>, state: AppState) {
    let start_time = Instant::now();

    let ws_url = target_url
        .replace("http://", "ws://")
        .replace("https://", "wss://");

    info!("[Gateway] Connecting to backend WebSocket at {}", ws_url);

    let host = ws_url
        .trim_start_matches("ws://")
        .trim_start_matches("wss://")
        .split('/')
        .next()
        .unwrap_or("");

    let mut request_builder = tokio_tungstenite::tungstenite::http::Request::builder()
        .uri(&ws_url)
        .header("Host", host)
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Version", "13")
        .header("Sec-WebSocket-Key", generate_key());
    if let Some(cookie_val) = cookie {
        request_builder = request_builder.header("Cookie", cookie_val);
    }
    let request = request_builder.body(()).unwrap();

    let backend_result = connect_async(request).await;

    let (backend_ws_stream, _) = match backend_result {
        Ok(conn) => conn,
        Err(e) => {
            error!("[Gateway] Failed to connect to backend WebSocket: {}", e);
            let latency_ms = start_time.elapsed().as_millis() as u64;
            state.metrics.record("websocket", latency_ms, true);
            return;
        }
    };

    info!("[Gateway] Backend WebSocket connected, starting proxy");

    let (mut client_sender, mut client_receiver) = client_socket.split();
    let (mut backend_sender, mut backend_receiver) = backend_ws_stream.split();

    let client_to_backend = tokio::spawn(async move {
        while let Some(msg_result) = client_receiver.next().await {
            match msg_result {
                Ok(msg) => {
                    let backend_msg = convert_axum_to_tungstenite(msg);
                    if let Err(e) = backend_sender.send(backend_msg).await {
                        warn!("[Gateway] Error sending to backend: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    warn!("[Gateway] Error receiving from client: {}", e);
                    break;
                }
            }
        }
    });

    let backend_to_client = tokio::spawn(async move {
        while let Some(msg_result) = backend_receiver.next().await {
            match msg_result {
                Ok(msg) => {
                    let client_msg = convert_tungstenite_to_axum(msg);
                    if let Err(e) = client_sender.send(client_msg).await {
                        warn!("[Gateway] Error sending to client: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    warn!("[Gateway] Error receiving from backend: {}", e);
                    break;
                }
            }
        }
    });

    tokio::select! {
        _ = client_to_backend => {
            info!("[Gateway] Client -> Backend channel closed");
        }
        _ = backend_to_client => {
            info!("[Gateway] Backend -> Client channel closed");
        }
    }

    let latency_ms = start_time.elapsed().as_millis() as u64;
    state.metrics.record("websocket", latency_ms, false);

    info!("[Gateway] WebSocket proxy session ended");
}

fn convert_axum_to_tungstenite(msg: Message) -> TungsteniteMessage {
    match msg {
        Message::Text(text) => TungsteniteMessage::Text(text),
        Message::Binary(bin) => TungsteniteMessage::Binary(bin),
        Message::Ping(ping) => TungsteniteMessage::Ping(ping),
        Message::Pong(pong) => TungsteniteMessage::Pong(pong),
        Message::Close(close_frame) => TungsteniteMessage::Close(close_frame.map(|cf| {
            tokio_tungstenite::tungstenite::protocol::CloseFrame {
                code: cf.code.into(),
                reason: cf.reason,
            }
        })),
    }
}

fn convert_tungstenite_to_axum(msg: TungsteniteMessage) -> Message {
    match msg {
        TungsteniteMessage::Text(text) => Message::Text(text),
        TungsteniteMessage::Binary(bin) => Message::Binary(bin),
        TungsteniteMessage::Ping(ping) => Message::Ping(ping),
        TungsteniteMessage::Pong(pong) => Message::Pong(pong),
        TungsteniteMessage::Close(close_frame) => {
            Message::Close(close_frame.map(|cf| axum::extract::ws::CloseFrame {
                code: cf.code.into(),
                reason: cf.reason,
            }))
        }
        TungsteniteMessage::Frame(_) => Message::Binary(vec![]),
    }
}
