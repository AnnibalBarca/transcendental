use std::sync::Arc;
use std::sync::atomic::Ordering;

use log::info;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::game::board::moves::MoveCalculator;
use crate::game::board::{Color, PieceType, Square};
use crate::game::cards::{CardId, CardTarget, card_def};
use crate::game::game_loop::{GameTimer, MoveHandler};
use crate::game::manager::GameInstance;
use crate::game::redis_helpers::{get_profile_picture, publish_game_result};
use crate::websocket::lobby::{LobbyState, OutgoingMessage, PlayerId, PlayerSlot};

const DEFAULT_COMMON_CARDS: &[&str] = &["1", "2", "3", "5", "6", "7", "8", "9", "10", "11"];

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum WsCommand {
    #[serde(rename = "message")]
    Message { text: String },
    #[serde(rename = "move_piece")]
    MovePiece {
        from: String,
        to: String,
        promotion: Option<String>,
    },
    #[serde(rename = "play_card")]
    PlayCard {
        card_id: String,
        target: Option<String>,
    },
    #[serde(rename = "discard_card")]
    DiscardCard { card_id: String },
    #[serde(rename = "get_hand")]
    GetHand,
    #[serde(rename = "set_picture")]
    SetPicture { picture_id: String },
    #[serde(rename = "pong")]
    Pong,
}

fn first_move_remaining_ms(instance: &GameInstance) -> Option<u64> {
    if !instance.game_loop.running.load(Ordering::Relaxed) {
        return None;
    }
    if instance.game_loop.first_move_played.load(Ordering::Relaxed) {
        return None;
    }
    let deadline = *instance.game_loop.first_move_deadline.lock().unwrap();
    let now = std::time::Instant::now();
    if now >= deadline {
        Some(0)
    } else {
        Some(deadline.duration_since(now).as_millis() as u64)
    }
}

pub(crate) fn turn_changed_msg(instance: &GameInstance, current_player: &str) -> Value {
    let (white_time, black_time) = instance.game_loop.get_times();
    let (white_mult, black_mult) = instance.game_loop.get_time_multipliers();
    json!({
        "current_player": current_player,
        "white_ms": white_time,
        "black_ms": black_time,
        "timer_running": instance.game_loop.timer_running.load(Ordering::Relaxed),
        "first_move_remaining_ms": first_move_remaining_ms(instance),
        "time_multiplier": {
            "white": white_mult,
            "black": black_mult,
        },
    })
}

async fn player_picture(
    slot: Option<&PlayerSlot>,
    redis_pool: &deadpool_redis::Pool,
) -> Option<String> {
    match slot {
        Some(p) if !p.picture_id.is_empty() => Some(p.picture_id.clone()),
        Some(p) => get_profile_picture(redis_pool, &p.user_id).await,
        None => None,
    }
}

pub(crate) async fn players_info_msg(
    lobby: &LobbyState,
    redis_pool: &deadpool_redis::Pool,
) -> Value {
    let player1_name = lobby.players[0]
        .as_ref()
        .map(|p| p.username.clone())
        .unwrap_or_else(|| "Player 1".to_string());
    let player2_name = lobby.players[1]
        .as_ref()
        .map(|p| p.username.clone())
        .unwrap_or_else(|| "Player 2".to_string());
    let player1_picture = player_picture(lobby.players[0].as_ref(), redis_pool).await;
    let player2_picture = player_picture(lobby.players[1].as_ref(), redis_pool).await;
    json!({
        "player1": player1_name,
        "player2": player2_name,
        "picture1": player1_picture,
        "picture2": player2_picture,
    })
}

pub async fn start_game(instance: &Arc<GameInstance>) {
    let mut lobby = instance.lobby.lock().await;
    if !lobby.both_connected() {
        return;
    }

    let already_running = instance
        .game_loop
        .running
        .load(std::sync::atomic::Ordering::Relaxed);
    if !already_running {
        instance.game_loop.restart();

        if let Some(db_pool) = &instance.db_pool {
            let (white_uid, black_uid) = lobby.color_user_ids();
            drop(lobby);
            let mut deck: Vec<(CardId, u8)> = Vec::new();
            for uid in [white_uid, black_uid].into_iter().flatten() {
                match crate::db::get_player_deck(db_pool, &uid).await {
                    Ok(cards) => {
                        info!(
                            "[Chess-WS] get_player_deck user={} cards={} raw={:?}",
                            uid,
                            cards.len(),
                            cards
                        );
                        for (card_id, rarity) in cards {
                            if let Some(c) = CardId::from_str(&card_id) {
                                deck.push((c, rarity as u8));
                            } else {
                                log::warn!("[Chess-WS] Unknown card id in deck: {}", card_id);
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!("[Chess-WS] get_player_deck error for {}: {}", uid, e);
                    }
                }
            }
            info!(
                "[Chess-WS] Merged custom deck size={} deck={:?}",
                deck.len(),
                deck
            );
            if deck.is_empty() {
                for id in DEFAULT_COMMON_CARDS {
                    if let Some(c) = CardId::from_str(id) {
                        deck.push((c, 0));
                    }
                }
            }
            instance.game_loop.set_custom_deck(deck);
            lobby = instance.lobby.lock().await;
        }
    }

    start_game_locked(instance, &lobby);
}

fn start_game_locked(instance: &GameInstance, lobby: &LobbyState) {
    instance.game_loop.start();
    let (white_time, black_time) = instance.game_loop.get_times();
    lobby.broadcast(OutgoingMessage {
        action: "started".to_string(),
        from: None,
        message: json!({
            "text": "Game started",
            "white_ms": white_time,
            "black_ms": black_time,
            "first_move_remaining_ms": first_move_remaining_ms(instance),
        }),
    });

    let (white_board, black_board) = {
        let mut board = instance.game_loop.board.lock().unwrap();
        board.compute_allowed_moves();
        (
            board.to_json_for_color(Some(Color::White)),
            board.to_json_for_color(Some(Color::Black)),
        )
    };

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

    let (white_hand, black_hand) = instance.game_loop.get_hands();
    info!(
        "[Chess-WS] Sending hands: white={:?}, black={:?}",
        white_hand, black_hand
    );
    lobby.send_to_player1(OutgoingMessage {
        action: "hand".to_string(),
        from: None,
        message: json!({ "cards": instance.game_loop.hand_json_for(Color::White) }),
    });
    lobby.send_to_player2(OutgoingMessage {
        action: "hand".to_string(),
        from: None,
        message: json!({ "cards": instance.game_loop.hand_json_for(Color::Black) }),
    });

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
        message: turn_changed_msg(instance, current_player_label),
    });
}

pub async fn handle_message<'a>(
    id: PlayerId,
    text: &str,
    instance: &'a Arc<GameInstance>,
    mut lobby: tokio::sync::MutexGuard<'a, LobbyState>,
) -> tokio::sync::MutexGuard<'a, LobbyState> {
    info!("[Chess-WS] Received from {}: {}", id.label(), text);

    match serde_json::from_str::<WsCommand>(text) {
        Ok(WsCommand::Message { text: message_text }) => {
            if id == PlayerId::Player1 {
                lobby.send_to_player2(OutgoingMessage {
                    action: "message".to_string(),
                    from: Some(id),
                    message: json!(message_text),
                });
            } else {
                lobby.send_to_player1(OutgoingMessage {
                    action: "message".to_string(),
                    from: Some(id),
                    message: json!(message_text),
                });
            }
        }

        Ok(WsCommand::MovePiece {
            from,
            to,
            promotion,
        }) => {
            let from_sq = match Square::from_coord(&from) {
                Some(sq) => sq,
                None => {
                    lobby.send_to(
                        id,
                        OutgoingMessage {
                            action: "move_result".to_string(),
                            from: Some(id),
                            message: json!({"valid": false, "reason": "invalid source square"}),
                        },
                    );
                    return lobby;
                }
            };

            let to_sq = match Square::from_coord(&to) {
                Some(sq) => sq,
                None => {
                    lobby.send_to(
                        id,
                        OutgoingMessage {
                            action: "move_result".to_string(),
                            from: Some(id),
                            message: json!({"valid": false, "reason": "invalid target square"}),
                        },
                    );
                    return lobby;
                }
            };

            if let Some(ref p) = promotion {
                if PieceType::from_promotion_str(p).is_none() {
                    lobby.send_to(
                        id,
                        OutgoingMessage {
                            action: "move_result".to_string(),
                            from: Some(id),
                            message: json!({"valid": false, "reason": "invalid promotion piece"}),
                        },
                    );
                    return lobby;
                }
            }

            let promotion_piece = promotion.as_deref().and_then(PieceType::from_promotion_str);

            let player_color = if id == PlayerId::Player1 {
                Color::White
            } else {
                Color::Black
            };

            match instance
                .game_loop
                .make_move(from_sq, to_sq, player_color, promotion_piece)
            {
                Ok(result) => {
                    let move_obj = json!({
                        "from": from,
                        "to": to,
                        "piece": format!("{:?}", player_color).to_lowercase()
                    });

                    lobby.send_to(
                        id,
                        OutgoingMessage {
                            action: "move_result".to_string(),
                            from: Some(id),
                            message: json!({
                                "valid": true,
                                "move": move_obj,
                                "white_ms": result.white_ms,
                                "black_ms": result.black_ms,
                            }),
                        },
                    );

                    lobby.send_to(
                        id.other(),
                        OutgoingMessage {
                            action: "opponent_move".to_string(),
                            from: Some(id),
                            message: move_obj,
                        },
                    );

                    lobby.send_to_player1(OutgoingMessage {
                        action: "game_state".to_string(),
                        from: None,
                        message: result.white_board_json,
                    });
                    lobby.send_to_player2(OutgoingMessage {
                        action: "game_state".to_string(),
                        from: None,
                        message: result.black_board_json,
                    });

                    lobby.send_to_player1(OutgoingMessage {
                        action: "hand".to_string(),
                        from: None,
                        message: json!({ "cards": instance.game_loop.hand_json_for(Color::White) }),
                    });
                    lobby.send_to_player2(OutgoingMessage {
                        action: "hand".to_string(),
                        from: None,
                        message: json!({ "cards": instance.game_loop.hand_json_for(Color::Black) }),
                    });

                    let current_player_label = if result.current_player == Color::White {
                        "white"
                    } else {
                        "black"
                    };
                    let turn_msg = turn_changed_msg(instance, current_player_label);
                    lobby.broadcast(OutgoingMessage {
                        action: "turn_changed".to_string(),
                        from: None,
                        message: turn_msg,
                    });

                    if result.check {
                        let checked_color = if result.current_player == Color::White {
                            "white"
                        } else {
                            "black"
                        };
                        lobby.broadcast(OutgoingMessage {
                            action: "check".to_string(),
                            from: None,
                            message: json!({"color": checked_color}),
                        });
                    }

                    if result.checkmate {
                        let winner_label = if result.winner == Some(Color::White) {
                            "white"
                        } else {
                            "black"
                        };
                        let checkmate_msg = json!({"winner": winner_label});
                        lobby.broadcast(OutgoingMessage {
                            action: "checkmate".to_string(),
                            from: None,
                            message: checkmate_msg,
                        });
                        let (white_uid, black_uid) = lobby.color_user_ids();
                        drop(lobby);
                        publish_game_result(
                            &instance.redis_pool,
                            &instance.game_id,
                            winner_label,
                            Some(winner_label),
                            white_uid.as_deref(),
                            black_uid.as_deref(),
                        )
                        .await;
                        instance.game_loop.end();
                        instance.end_game_cleanup().await;
                        return instance.lobby.lock().await;
                    }
                }
                Err(reason) => {
                    lobby.send_to(
                        id,
                        OutgoingMessage {
                            action: "move_result".to_string(),
                            from: Some(id),
                            message: json!({"valid": false, "reason": reason}),
                        },
                    );
                }
            }
        }
        Ok(WsCommand::PlayCard { card_id, target }) => {
            let parsed_card = match CardId::from_str(&card_id) {
                Some(c) => c,
                None => {
                    lobby.send_to(
                        id,
                        OutgoingMessage {
                            action: "card_result".to_string(),
                            from: Some(id),
                            message: json!({"valid": false, "reason": "unknown card"}),
                        },
                    );
                    return lobby;
                }
            };

            let player_color = if id == PlayerId::Player1 {
                Color::White
            } else {
                Color::Black
            };

            let card_target = match parse_card_target(parsed_card, target.as_deref()) {
                Ok(t) => t,
                Err(reason) => {
                    lobby.send_to(
                        id,
                        OutgoingMessage {
                            action: "card_result".to_string(),
                            from: Some(id),
                            message: json!({"valid": false, "reason": reason}),
                        },
                    );
                    return lobby;
                }
            };

            match instance
                .game_loop
                .play_card(parsed_card, player_color, card_target)
            {
                Ok((result, card_rarity)) => {
                    let ends_turn = card_def(parsed_card).ends_turn;
                    if ends_turn {
                        instance.game_loop.record_move();
                        instance.game_loop.discard_and_draw_if_needed(player_color);
                    }

                    let mut board = instance.game_loop.board.lock().unwrap();
                    board.compute_allowed_moves();
                    let white_board_json = board.to_json_for_color(Some(Color::White));
                    let black_board_json = board.to_json_for_color(Some(Color::Black));
                    let (white_hand, black_hand) = instance.game_loop.get_hands();
                    drop(board);

                    let hand_color = if id == PlayerId::Player1 {
                        Color::White
                    } else {
                        Color::Black
                    };
                    lobby.send_to(
                        id,
                        OutgoingMessage {
                            action: "card_result".to_string(),
                            from: Some(id),
                            message: json!({
                                "valid": true,
                                "card_id": card_id,
                                "rarity": card_rarity,
                                "message_id": result.message_id,
                                "effects": result.effects,
                                "ends_turn": ends_turn,
                                "hand": instance.game_loop.hand_json_for(hand_color),
                            }),
                        },
                    );

                    if parsed_card == CardId::WheelOfFortune {
                        lobby.send_to(
                            id.other(),
                            OutgoingMessage {
                                action: "hand".to_string(),
                                from: None,
                                message: json!({
                                    "cards": instance.game_loop.hand_json_for(player_color.other())
                                }),
                            },
                        );
                    }

                    lobby.send_to(
                        id.other(),
                        OutgoingMessage {
                            action: "opponent_card_played".to_string(),
                            from: Some(id),
                            message: json!({
                                "card_id": card_id,
                                "rarity": card_rarity,
                                "message_id": result.message_id,
                                "effects": result.effects,
                                "hand_size": if player_color == Color::White { black_hand.len() } else { white_hand.len() },
                            }),
                        },
                    );

                    lobby.send_to_player1(OutgoingMessage {
                        action: "game_state".to_string(),
                        from: None,
                        message: white_board_json,
                    });
                    lobby.send_to_player2(OutgoingMessage {
                        action: "game_state".to_string(),
                        from: None,
                        message: black_board_json,
                    });

                    if ends_turn {
                        lobby.send_to_player1(OutgoingMessage {
                            action: "hand".to_string(),
                            from: None,
                            message: json!({ "cards": instance.game_loop.hand_json_for(Color::White) }),
                        });
                        lobby.send_to_player2(OutgoingMessage {
                            action: "hand".to_string(),
                            from: None,
                            message: json!({ "cards": instance.game_loop.hand_json_for(Color::Black) }),
                        });

                        let current_player_label = if instance
                            .game_loop
                            .is_white_turn
                            .load(std::sync::atomic::Ordering::Relaxed)
                        {
                            "white"
                        } else {
                            "black"
                        };
                        let turn_msg = turn_changed_msg(instance, current_player_label);
                        lobby.broadcast(OutgoingMessage {
                            action: "turn_changed".to_string(),
                            from: None,
                            message: turn_msg,
                        });
                    }
                }
                Err(reason) => {
                    lobby.send_to(
                        id,
                        OutgoingMessage {
                            action: "card_result".to_string(),
                            from: Some(id),
                            message: json!({"valid": false, "reason": reason}),
                        },
                    );
                }
            }
        }
        Ok(WsCommand::DiscardCard { card_id }) => {
            let parsed_card = match CardId::from_str(&card_id) {
                Some(c) => c,
                None => {
                    lobby.send_to(
                        id,
                        OutgoingMessage {
                            action: "card_result".to_string(),
                            from: Some(id),
                            message: json!({"valid": false, "reason": "unknown card"}),
                        },
                    );
                    return lobby;
                }
            };

            let player_color = if id == PlayerId::Player1 {
                Color::White
            } else {
                Color::Black
            };

            match instance.game_loop.discard_card(parsed_card, player_color) {
                Ok(rarity) => {
                    let (white_hand, black_hand) = instance.game_loop.get_hands();
                    lobby.send_to(
                        id,
                        OutgoingMessage {
                            action: "card_result".to_string(),
                            from: Some(id),
                            message: json!({
                                "valid": true,
                                "card_id": card_id,
                                "rarity": rarity,
                                "discarded": true,
                                "hand": instance.game_loop.hand_json_for(player_color),
                            }),
                        },
                    );
                    lobby.send_to(
                        id.other(),
                        OutgoingMessage {
                            action: "opponent_card_played".to_string(),
                            from: Some(id),
                            message: json!({
                                "card_id": card_id,
                                "rarity": rarity,
                                "discarded": true,
                                "hand_size": if player_color == Color::White { black_hand.len() } else { white_hand.len() },
                            }),
                        },
                    );
                }
                Err(reason) => {
                    lobby.send_to(
                        id,
                        OutgoingMessage {
                            action: "card_result".to_string(),
                            from: Some(id),
                            message: json!({"valid": false, "reason": reason}),
                        },
                    );
                }
            }
        }
        Ok(WsCommand::GetHand) => {
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
                    message: json!({ "cards": instance.game_loop.hand_json_for(hand_color) }),
                },
            );
        }
        Ok(WsCommand::SetPicture { picture_id }) => {
            if let Some(slot) = &mut lobby.players[id.idx()] {
                slot.picture_id = picture_id.clone();
            }
            let color = if id == PlayerId::Player1 {
                "white"
            } else {
                "black"
            };
            lobby.broadcast(OutgoingMessage {
                action: "players_picture".to_string(),
                from: None,
                message: json!({
                    "color": color,
                    "picture_id": picture_id,
                }),
            });
        }
        Ok(WsCommand::Pong) => {}
        Err(_) => match id {
            PlayerId::Player1 => {
                lobby.send_to_player1(OutgoingMessage {
                    action: "echo".to_string(),
                    from: Some(id),
                    message: json!("Tu es Player1"),
                });
            }
            PlayerId::Player2 => {
                lobby.send_to_player2(OutgoingMessage {
                    action: "echo".to_string(),
                    from: Some(id),
                    message: json!("Tu es Player2"),
                });
            }
        },
    }
    lobby
}

fn parse_card_target(card: CardId, target: Option<&str>) -> Result<CardTarget, &'static str> {
    match card {
        CardId::DeadlyZone
        | CardId::Cannon
        | CardId::Battlefield
        | CardId::Annihilation
        | CardId::VeteranKnight
        | CardId::VeteranRook
        | CardId::VeteranBishop
        | CardId::Frog
        | CardId::Ninja
        | CardId::Breakthrough
        | CardId::DesperateRescue => {
            let coord = target.ok_or("err_need_target")?;
            let square = Square::from_coord(coord).ok_or("err_invalid_square")?;
            Ok(CardTarget::Square(square))
        }
        CardId::TimeBoost
        | CardId::RussianRoulette
        | CardId::Journey
        | CardId::Fog
        | CardId::CrazyKnight
        | CardId::BeastWork
        | CardId::FuriousMason
        | CardId::CatchTheKnightThief
        | CardId::BeastOfBurden
        | CardId::Bestification
        | CardId::HermitArchitect
        | CardId::Pyromaniac
        | CardId::Sniper
        | CardId::Trash
        | CardId::PushBack
        | CardId::Garbage
        | CardId::WheelOfFortune
        // | CardId::Magnetism
        // | CardId::Bastion
        | CardId::Traitor => Ok(CardTarget::None),
    }
}
