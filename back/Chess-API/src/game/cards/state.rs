use std::collections::HashMap;

use crate::game::board::Board;
use crate::game::board::types::{Color, Piece, PieceType, Square};
use crate::game::board::utils::square_to_index;

#[derive(Debug, Clone, serde::Serialize)]
pub struct LastMove {
    pub from: Square,
    pub to: Square,
    pub moved: Piece,
    pub captured: Option<Piece>,
}

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct CardState {
    pub fog_remaining_turns: u8,
    pub fog_rarity: u8,
    pub fog_creator: Option<Color>,
    pub time_multiplier: HashMap<Color, u8>,
    pub sniper_remaining: HashMap<Color, u8>,
    pub battlefield: Option<(Square, u8)>,
    pub last_move: Option<LastMove>,
    pub magnetism: Option<(Square, Color)>,
    pub bastion: Option<(Square, Color)>,
    pub ninja: Option<(Square, Color)>,
    #[serde(skip)]
    pub ninja_expire_after_opponent_move: bool,
    pub traitor: Option<Color>,
}

impl CardState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_time_multiplier(&mut self, color: Color, multiplier: u8) {
        self.time_multiplier.insert(color, multiplier);
    }

    pub fn get_time_multiplier(&self, color: Color) -> u8 {
        self.time_multiplier.get(&color).copied().unwrap_or(1)
    }

    pub fn start_fog(&mut self, creator: Color, turns: u8, rarity: u8) {
        self.fog_creator = Some(creator);
        self.fog_remaining_turns = turns;
        self.fog_rarity = rarity;
    }

    pub fn is_fog_active(&self) -> bool {
        self.fog_remaining_turns > 0
    }

    pub fn decrement_fog(&mut self) {
        if self.fog_remaining_turns > 0 {
            self.fog_remaining_turns -= 1;
            if self.fog_remaining_turns == 0 {
                self.fog_creator = None;
            }
        }
    }

    pub fn set_sniper(&mut self, color: Color, turns: u8) {
        self.sniper_remaining.insert(color, turns);
    }

    pub fn set_last_move(&mut self, last_move: LastMove) {
        self.last_move = Some(last_move);
    }

    pub fn clear_last_move(&mut self) {
        self.last_move = None;
    }
}

impl Board {
    pub fn decrement_sniper(&mut self, color: Color) {
        let Some(turns) = self.card_state.sniper_remaining.get_mut(&color) else {
            return;
        };

        if *turns > 0 {
            *turns -= 1;
        }

        if *turns == 0 {
            self.card_state.sniper_remaining.remove(&color);
            for slot in self.squares.iter_mut() {
                if let Some(piece) = slot.piece.as_mut() {
                    if piece.color == color && piece.is_sniper() {
                        piece.piece_modifier = None;
                    }
                }
            }
        }
    }

    pub fn clear_traitor(&mut self) {
        self.card_state.traitor = None;
    }

    pub fn clear_ninja(&mut self) {
        if let Some((square, _)) = self.card_state.ninja {
            let idx = crate::game::board::utils::square_to_index(square);
            if let Some(piece) = self.squares[idx].as_mut() {
                if piece.is_ninja() {
                    piece.piece_modifier = None;
                }
            }
        }
        self.card_state.ninja = None;
    }
}

pub fn record_last_move(board: &mut Board, from: Square, to: Square) {
    let from_idx = square_to_index(from);
    let to_idx = square_to_index(to);

    let moved = board.squares[from_idx].as_ref().cloned();
    let mut captured = board.squares[to_idx].as_ref().cloned();

    if captured.is_none()
        && board.squares[from_idx]
            .as_ref()
            .map_or(false, |p| p.piece_type == PieceType::Pawn)
    {
        if let Some(ep) = board.en_passant_target {
            if to == ep && from.file != to.file {
                let victim = Square {
                    file: to.file,
                    rank: from.rank,
                };
                captured = board.squares[square_to_index(victim)].as_ref().cloned();
            }
        }
    }

    if let Some(moved) = moved {
        board.card_state.set_last_move(LastMove {
            from,
            to,
            moved,
            captured,
        });
    }
}
