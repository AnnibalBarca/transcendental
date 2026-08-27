use std::sync::atomic::Ordering;

use log::info;

use crate::game::board::apply::MoveApplier;
use crate::game::board::checks::CheckAnalyzer;
use crate::game::board::moves::MoveCalculator;
use crate::game::board::{Color, PieceType, Square};
use crate::game::game_loop::{GameLoop, GameTimer};

#[derive(Debug, Clone)]
pub struct MoveResult {
    pub white_board_json: serde_json::Value,
    pub black_board_json: serde_json::Value,
    pub white_ms: u64,
    pub black_ms: u64,
    pub current_player: Color,
    pub check: bool,
    pub checkmate: bool,
    pub winner: Option<Color>,
}

pub trait MoveHandler {
    fn make_move(
        &self,
        from: Square,
        to: Square,
        player_color: Color,
        promotion: Option<PieceType>,
    ) -> Result<MoveResult, &'static str>;
}

impl MoveHandler for GameLoop {
    fn make_move(
        &self,
        from: Square,
        to: Square,
        player_color: Color,
        promotion: Option<PieceType>,
    ) -> Result<MoveResult, &'static str> {
        let current_turn = if self.is_white_turn.load(Ordering::Relaxed) {
            Color::White
        } else {
            Color::Black
        };

        if player_color != current_turn {
            info!(
                "[MoveHandler] rejected: not your turn (player={:?}, current={:?})",
                player_color, current_turn
            );
            return Err("not your turn");
        }

        {
            let board = self.board.lock().unwrap();

            let piece = board
                .get_piece_at(from)
                .ok_or("no piece at source square")?;
            if piece.color != player_color {
                let is_traitor = board.card_state.traitor == Some(player_color)
                    && piece.piece_type == PieceType::Pawn
                    && board
                        .get_piece_at(to)
                        .map_or(false, |t| t.color == piece.color);
                if !is_traitor {
                    info!(
                        "[MoveHandler] rejected: not your piece (piece={:?}, player={:?})",
                        piece.color, player_color
                    );
                    return Err("not your piece");
                }
            }

            if !board.is_move_allowed(from, to) {
                info!(
                    "[MoveHandler] rejected: move not allowed (from={:?}, to={:?}, piece={:?})",
                    from, to, piece.piece_type
                );
                return Err("move not allowed");
            }
        }

        {
            let mut board = self.board.lock().unwrap();
            crate::game::cards::state::record_last_move(&mut board, from, to);
            if let Err(e) = board.apply_move(from, to, promotion) {
                board.card_state.clear_last_move();
                return Err(e);
            }
        }

        self.record_move();
        self.discard_and_draw_if_needed(player_color);

        let mut board = self.board.lock().unwrap();
        board.card_state.decrement_fog();
        board.decrement_sniper(player_color);
        board.clear_traitor();

        if let Some((_, owner)) = board.card_state.ninja {
            if owner != player_color {
                board.card_state.ninja_expire_after_opponent_move = true;
            } else if board.card_state.ninja_expire_after_opponent_move {
                board.clear_ninja();
                board.card_state.ninja_expire_after_opponent_move = false;
            }
        }

        board.compute_allowed_moves();

        let opponent = player_color.other();
        let check = board.is_in_check(opponent);
        let checkmate = check && board.is_checkmate(opponent);

        let white_board_json = board.to_json_for_color(Some(Color::White));
        let black_board_json = board.to_json_for_color(Some(Color::Black));
        let (white_ms, black_ms) = self.get_times();

        Ok(MoveResult {
            white_board_json,
            black_board_json,
            white_ms,
            black_ms,
            current_player: opponent,
            check,
            checkmate,
            winner: if checkmate { Some(player_color) } else { None },
        })
    }
}
