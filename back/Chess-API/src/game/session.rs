use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::interval;

use log::info;
use serde_json::json;

use crate::game::board::moves::MoveCalculator;
use crate::game::board::Color;
use crate::game::commands::{handle_message, players_info_msg, start_game, turn_changed_msg};
use crate::game::manager::GameInstance;
use crate::game::redis_helpers::{cleanup_user_session, persist_user_session};
use crate::websocket::lobby::{OutgoingMessage, PlayerId};
use api_core::auth::TokenClaims;

const MAX_MSGS_PER_WINDOW: usize = 15;
const RATE_WINDOW: Duration = Duration::from_secs(1);

fn is_pong(text: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(text)
        .ok()
        .and_then(|v| v.get("action").and_then(|a| a.as_str()).map(str::to_string))
        .map(|a| a == "pong")
        .unwrap_or(false)
}

pub async fn handle_player_session(
    socket: WebSocket,
    instance: Arc<GameInstance>,
    claims: TokenClaims,
    redis_pool: deadpool_redis::Pool,
    picture_id: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("[Chess-WS] New connection, user={:?}", claims.username);

    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<OutgoingMessage>();

    let username = claims.username.clone();
    let user_id = claims.sub.clone();
    let id = {
        let mut lobby = instance.lobby.lock().await;
        lobby.connect(tx.clone(), username, user_id.clone(), picture_id)
    };

    let Some(id) = id else {
        let payload = serde_json::json!({
            "action": "game_full",
            "message": "Game is full, try again later"
        });
        let _ = sender.send(Message::Text(payload.to_string())).await;
        let _ = sender.close().await;
        return Ok(());
    };

    let mut ids = instance.player_user_ids.lock().await;
    if !ids.contains(&user_id) {
        ids.push(user_id.clone());
    }
    persist_user_session(&redis_pool, &user_id, &instance.game_id).await;
    drop(ids);

    info!("[Chess-WS] Assigned {}", id.label());

    let mut forward_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let text = serde_json::to_string(&msg).unwrap_or_default();
            if sender.send(Message::Text(text)).await.is_err() {
                break;
            }
        }
    });

    {
        let lobby = instance.lobby.lock().await;
        lobby.send_to(
            id,
            OutgoingMessage {
                action: "connected".to_string(),
                from: None,
                message: json!(format!("You are {}", id.label())),
            },
        );

        if lobby.both_connected() {
            let info_msg = players_info_msg(&lobby, &redis_pool).await;
            lobby.broadcast(OutgoingMessage {
                action: "ready".to_string(),
                from: None,
                message: json!("Both players are connected"),
            });
            lobby.broadcast(OutgoingMessage {
                action: "players_info".to_string(),
                from: None,
                message: info_msg,
            });
        } else {
            lobby.send_to(
                id,
                OutgoingMessage {
                    action: "waiting".to_string(),
                    from: None,
                    message: json!("Waiting for opponent..."),
                },
            );
        }

        if instance
            .game_loop
            .running
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            let (white_board, black_board) = {
                let mut board = instance.game_loop.board.lock().unwrap();
                board.compute_allowed_moves();
                (
                    board.to_json_for_color(Some(Color::White)),
                    board.to_json_for_color(Some(Color::Black)),
                )
            };
            let (white_hand, black_hand) = instance.game_loop.get_hands();
            info!(
                "[Chess-WS] Reconnection: sending hands to {} (white={:?}, black={:?})",
                id.label(),
                white_hand,
                black_hand
            );
            let info_msg = players_info_msg(&lobby, &redis_pool).await;
            lobby.broadcast(OutgoingMessage {
                action: "players_info".to_string(),
                from: None,
                message: info_msg,
            });
            lobby.send_to_player1(OutgoingMessage {
                action: "game_state".to_string(),
                from: None,
                message: white_board,
            });
            lobby.send_to_player2(OutgoingMessage {
                action: "game_state".to_string(),
                from: None,
                message: black_board,
            });
            let hand_color = if id == PlayerId::Player1 {
                Color::White
            } else {
                Color::Black
            };
            lobby.send_to(
                id,
                OutgoingMessage {
                    action: "hand".to_string(),
                    from: None,
                    message: json!({
                        "cards": instance.game_loop.hand_json_for(hand_color)
                    }),
                },
            );
            let current_player_label = if instance
                .game_loop
                .is_white_turn
                .load(std::sync::atomic::Ordering::Relaxed)
            {
                "white"
            } else {
                "black"
            };
            lobby.broadcast(OutgoingMessage {
                action: "turn_changed".to_string(),
                from: None,
                message: turn_changed_msg(&instance, current_player_label),
            });
        }
    }

    let should_auto_start = {
        let lobby = instance.lobby.lock().await;
        lobby.both_connected()
            && !instance
                .game_loop
                .running
                .load(std::sync::atomic::Ordering::Relaxed)
            && !instance
                .game_loop
                .started
                .load(std::sync::atomic::Ordering::Relaxed)
    };
    if should_auto_start {
        info!(
            "[Chess-WS] Both players connected, auto-starting game {}",
            instance.game_id
        );
        start_game(&instance).await;
    }

    let mut heartbeat_interval = interval(Duration::from_secs(15));
    let mut last_pong = Instant::now();
    let mut rate_window_start = Instant::now();
    let mut rate_count = 0usize;

    loop {
        tokio::select! {
            _ = heartbeat_interval.tick() => {
                if last_pong.elapsed() > Duration::from_secs(30) {
                    info!("[Chess-WS] Heartbeat missed for {}, disconnecting", id.label());
                    break;
                }

                let lobby = instance.lobby.lock().await;
                lobby.send_to(
                    id,
                    OutgoingMessage {
                        action: "ping".to_string(),
                        from: None,
                        message: json!({}),
                    },
                );
            }
            msg_result = receiver.next() => {
                match msg_result {
                    Some(Ok(Message::Text(text))) => {
                        if is_pong(&text) {
                            last_pong = Instant::now();
                        } else {
                            let now = Instant::now();
                            if now.duration_since(rate_window_start) > RATE_WINDOW {
                                rate_window_start = now;
                                rate_count = 0;
                            }
                            rate_count += 1;
                            if rate_count > MAX_MSGS_PER_WINDOW {
                                let lobby = instance.lobby.lock().await;
                                lobby.send_to(
                                    id,
                                    OutgoingMessage {
                                        action: "rate_limited".to_string(),
                                        from: None,
                                        message: json!("Too many messages, slow down"),
                                    },
                                );
                            } else {
                                let _ = handle_message(
                                    id,
                                    &text,
                                    &instance,
                                    instance.lobby.lock().await,
                                )
                                .await;
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | Some(Err(_)) | None => {
                        info!("[Chess-WS] Connection closed for {}", id.label());
                        break;
                    }
                    Some(Ok(_)) => {}
                }
            }
            _ = &mut forward_task => {
                info!("[Chess-WS] Forward task ended for {}", id.label());
                break;
            }
        }
    }

    let left_for_good = {
        let mut lobby = instance.lobby.lock().await;
        let started = instance
            .game_loop
            .started
            .load(std::sync::atomic::Ordering::Relaxed);
        let running = instance
            .game_loop
            .running
            .load(std::sync::atomic::Ordering::Relaxed);
        let left = !started && !running;
        lobby.disconnect(id, left);
        left
    };
    if left_for_good {
        cleanup_user_session(&redis_pool, &user_id).await;
    }
    drop(tx);
    let _ = forward_task.await;

    info!(
        "[Chess-WS] Session ended for {}, game continues",
        id.label()
    );
    Ok(())
}