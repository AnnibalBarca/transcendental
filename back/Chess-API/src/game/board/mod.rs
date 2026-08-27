#![allow(unused_imports)]

pub mod apply;
pub mod checks;
pub mod moves;
pub mod types;
pub mod utils;

pub use apply::MoveApplier;
pub use checks::CheckAnalyzer;
pub use moves::{MoveCalculator, MoveLegality};
pub use types::*;

use crate::game::cards::CardState;
use utils::square_to_index;

#[derive(Debug, Clone)]
pub struct Board {
    pub squares: [SquareCell; 64],
    pub captured_pieces: Vec<Piece>,
    pub en_passant_target: Option<Square>,
    pub castling_rights: CastlingRights,
    pub card_state: CardState,
}

#[derive(Debug, Clone)]
pub struct Card {}

impl serde::Serialize for Board {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_json_for_color(None).serialize(serializer)
    }
}

impl Board {
    pub fn to_json_for_color(&self, color: Option<Color>) -> serde_json::Value {
        use serde_json::Map;

        let fog_active = color.map_or(false, |c| self.is_fog_active_for(c));
        let reachable_squares = if fog_active {
            self.reachable_squares_for(color.unwrap())
        } else {
            std::collections::HashSet::new()
        };

        let mut board_map = Map::new();

        for i in 0..64 {
            let square = utils::index_to_square(i);
            let key = square.to_coord();

            let mut value = if let Some(piece) = self.squares[i].as_ref() {
                let mut piece_value = serde_json::to_value(piece).unwrap();

                if let Some(player_color) = color {
                    if piece.color != player_color {
                        if fog_active && !reachable_squares.contains(&square) {
                            serde_json::Value::Null
                        } else {
                            let traitor_controlled = self.card_state.traitor == Some(player_color)
                                && piece.piece_type == PieceType::Pawn;
                            if !traitor_controlled {
                                if let Some(obj) = piece_value.as_object_mut() {
                                    obj.insert(
                                        "move_set".to_string(),
                                        serde_json::Value::Array(vec![]),
                                    );
                                }
                            }
                            piece_value
                        }
                    } else {
                        piece_value
                    }
                } else {
                    piece_value
                }
            } else {
                serde_json::Value::Null
            };

            if let Some(modifier) = self.squares[i].modifier {
                let modifier_value = serde_json::to_value(modifier).unwrap();
                if let Some(obj) = value.as_object_mut() {
                    obj.insert("square_modifier".to_string(), modifier_value);
                } else {
                    let mut map = Map::new();
                    map.insert("piece".to_string(), value);
                    map.insert("square_modifier".to_string(), modifier_value);
                    value = serde_json::Value::Object(map);
                }
            }

            board_map.insert(key, value);
        }

        let mut result = Map::new();
        result.insert("squares".to_string(), serde_json::Value::Object(board_map));
        result.insert(
            "captured_pieces".to_string(),
            serde_json::to_value(&self.captured_pieces).unwrap(),
        );
        result.insert(
            "en_passant_target".to_string(),
            serde_json::to_value(&self.en_passant_target).unwrap(),
        );
        result.insert(
            "castling_rights".to_string(),
            serde_json::to_value(&self.castling_rights).unwrap(),
        );
        result.insert(
            "card_state".to_string(),
            serde_json::to_value(&self.card_state).unwrap(),
        );

        serde_json::Value::Object(result)
    }

    pub fn get_piece_at(&self, square: Square) -> Option<&Piece> {
        self.squares[square_to_index(square)].as_ref()
    }

    pub fn is_battlefield_square(&self, index: usize) -> bool {
        matches!(
            self.squares[index].modifier,
            Some(SquareModifier::Battlefield { .. })
        )
    }

    pub fn is_deadly_zone_square(&self, index: usize) -> bool {
        matches!(
            self.squares[index].modifier,
            Some(SquareModifier::DeadlyZone { .. })
        )
    }

    pub fn is_ninja_square(&self, square: Square) -> bool {
        self.card_state.ninja.map_or(false, |(sq, _)| sq == square)
    }

    pub fn is_deadly_zone(&self, square: Square) -> bool {
        self.is_deadly_zone_square(square_to_index(square))
    }

    pub fn clear_battlefield(&mut self) {
        let Some((center, _)) = self.card_state.battlefield else {
            return;
        };
        self.card_state.battlefield = None;

        for df in -1i8..=1 {
            for dr in -1i8..=1 {
                let file = center.file as i8 + df;
                let rank = center.rank as i8 + dr;
                if !(1..=8).contains(&file) || !(1..=8).contains(&rank) {
                    continue;
                }
                let idx = square_to_index(Square {
                    file: file as u8,
                    rank: rank as u8,
                });
                if self.is_battlefield_square(idx) {
                    self.squares[idx].modifier = None;
                }
            }
        }
    }

    pub fn move_battlefield(&mut self, old_center: Square, new_center: Square) {
        let Some((_, rarity)) = self.card_state.battlefield else {
            return;
        };

        for df in -1i8..=1 {
            for dr in -1i8..=1 {
                let file = old_center.file as i8 + df;
                let rank = old_center.rank as i8 + dr;
                if !(1..=8).contains(&file) || !(1..=8).contains(&rank) {
                    continue;
                }
                let idx = square_to_index(Square {
                    file: file as u8,
                    rank: rank as u8,
                });
                if self.is_battlefield_square(idx) {
                    self.squares[idx].modifier = None;
                }
            }
        }

        for df in -1i8..=1 {
            for dr in -1i8..=1 {
                let file = new_center.file as i8 + df;
                let rank = new_center.rank as i8 + dr;
                if !(1..=8).contains(&file) || !(1..=8).contains(&rank) {
                    continue;
                }
                let idx = square_to_index(Square {
                    file: file as u8,
                    rank: rank as u8,
                });

                if !self.is_deadly_zone_square(idx) {
                    self.squares[idx].modifier = Some(SquareModifier::Battlefield { rarity });
                }
            }
        }

        self.card_state.battlefield = Some((new_center, rarity));
    }

    pub fn is_fog_active_for(&self, _color: Color) -> bool {
        self.card_state.is_fog_active()
    }

    pub fn reachable_squares_for(&self, color: Color) -> std::collections::HashSet<Square> {
        let mut reachable = std::collections::HashSet::new();
        for i in 0..64 {
            if let Some(piece) = self.squares[i].as_ref() {
                if piece.color == color {
                    for sq in &piece.move_set.allowed {
                        reachable.insert(*sq);
                    }
                }
            }
        }
        reachable
    }
}

impl Default for Board {
    fn default() -> Self {
        use types::Color::*;
        use types::PieceType::*;

        let mut squares: [SquareCell; 64] = std::array::from_fn(|_| SquareCell::new());

        let white_back = [Rook, Knight, Bishop, Queen, King, Bishop, Knight, Rook];
        let black_back = [Rook, Knight, Bishop, Queen, King, Bishop, Knight, Rook];

        for (i, &pt) in white_back.iter().enumerate() {
            squares[i].piece = Some(Piece::new(pt, White));
        }
        for i in 8..16 {
            squares[i].piece = Some(Piece::new(Pawn, White));
        }
        for i in 48..56 {
            squares[i].piece = Some(Piece::new(Pawn, Black));
        }
        for (i, &pt) in black_back.iter().enumerate() {
            squares[56 + i].piece = Some(Piece::new(pt, Black));
        }

        Self {
            squares,
            captured_pieces: Vec::new(),
            en_passant_target: None,
            castling_rights: CastlingRights::default(),
            card_state: CardState::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::board::apply::MoveApplier;
    use crate::game::board::checks::CheckAnalyzer;
    use crate::game::board::moves::MoveCalculator;
    use crate::game::board::utils::square_to_index;
    use crate::game::cards::effects::CardEffectApplier;
    use crate::game::cards::types::{CardId, CardTarget};

    fn square(file: u8, rank: u8) -> Square {
        Square { file, rank }
    }

    fn place(board: &mut Board, sq: Square, piece_type: PieceType, color: Color) {
        board.squares[square_to_index(sq)].piece = Some(Piece::new(piece_type, color));
    }

    #[test]
    fn test_veteran_knight_extra_moves() {
        let mut board = Board::default();
        for cell in board.squares.iter_mut() {
            cell.piece = None;
        }
        let e4 = square(5, 4);
        place(&mut board, e4, PieceType::Knight, Color::White);

        board.compute_allowed_moves();
        let idx = square_to_index(e4);
        let normal_count = board.squares[idx].as_ref().unwrap().move_set.allowed.len();

        board.squares[idx]
            .as_mut()
            .unwrap()
            .set_modifier(PieceModifier::VeteranKnight { rarity: Some(0) });
        board.compute_allowed_moves();
        let vet_count = board.squares[idx].as_ref().unwrap().move_set.allowed.len();

        assert!(
            vet_count > normal_count,
            "veteran knight should have more moves"
        );
    }

    #[test]
    fn test_percee_three_squares() {
        let mut board = Board::default();
        for cell in board.squares.iter_mut() {
            cell.piece = None;
        }
        let from = square(5, 2);
        place(&mut board, from, PieceType::Pawn, Color::White);
        let idx = square_to_index(from);
        // board.squares[idx]
        //     .as_mut()
        //     .unwrap()
        //     .set_modifier(PieceModifier::Percee);

        board.compute_allowed_moves();
        let moves = board.squares[idx]
            .as_ref()
            .unwrap()
            .move_set
            .allowed
            .clone();
        assert!(
            moves.contains(&square(5, 5)),
            "percee pawn should reach 3 squares ahead"
        );
        assert!(
            moves.contains(&square(5, 4)),
            "percee pawn should reach 2 squares ahead"
        );
        assert!(
            moves.contains(&square(5, 3)),
            "percee pawn should reach 1 square ahead"
        );
    }

    #[test]
    fn test_frog_jump() {
        let mut board = Board::default();
        for cell in board.squares.iter_mut() {
            cell.piece = None;
        }
        let from = square(5, 2);
        place(&mut board, from, PieceType::Pawn, Color::White);
        place(&mut board, square(5, 3), PieceType::Pawn, Color::Black);
        let idx = square_to_index(from);
        // board.squares[idx]
        //     .as_mut()
        //     .unwrap()
        //     .set_modifier(PieceModifier::Frog);

        board.compute_allowed_moves();
        let moves = board.squares[idx]
            .as_ref()
            .unwrap()
            .move_set
            .allowed
            .clone();
        assert!(
            moves.contains(&square(5, 4)),
            "frog pawn should jump over the piece in front"
        );
    }

    #[test]
    fn test_anihilation_removes_pawn() {
        let mut board = Board::default();
        let target = square(4, 7);
        place(&mut board, target, PieceType::Pawn, Color::Black);

        let result = board.apply_card_effect(
            CardId::Annihilation,
            0,
            Color::White,
            CardTarget::Square(target),
        );
        assert!(result.is_ok());
        assert!(
            board.squares[square_to_index(target)].is_none(),
            "pawn should be removed"
        );
    }

    #[test]
    fn test_ninja_traversable() {
        let mut board = Board::default();
        for cell in board.squares.iter_mut() {
            cell.piece = None;
        }
        let rook = square(1, 1);
        place(&mut board, rook, PieceType::Rook, Color::White);
        let ninja_sq = square(1, 4);
        place(&mut board, ninja_sq, PieceType::Pawn, Color::Black);
        board.card_state.ninja = Some((ninja_sq, Color::Black));

        board.compute_allowed_moves();
        let idx = square_to_index(rook);
        let moves = board.squares[idx]
            .as_ref()
            .unwrap()
            .move_set
            .allowed
            .clone();
        assert!(
            moves.contains(&square(1, 8)),
            "rook should pass through ninja piece"
        );
    }

    #[test]
    fn test_sauvetage_king_move() {
        let mut board = Board::default();
        for cell in board.squares.iter_mut() {
            cell.piece = None;
        }
        let king = square(5, 1);
        place(&mut board, king, PieceType::King, Color::White);
        let free = square(3, 1);
        place(&mut board, square(5, 8), PieceType::King, Color::Black);

        let result = board.apply_card_effect(
            CardId::DesperateRescue,
            0,
            Color::White,
            CardTarget::Square(free),
        );
        assert!(result.is_ok());
        assert!(
            board.squares[square_to_index(free)].is_some(),
            "king should move"
        );
    }

    #[test]
    fn test_traitor_pawn_captures_own_side() {
        let mut board = Board::default();
        for cell in board.squares.iter_mut() {
            cell.piece = None;
        }

        let pawn = square(4, 2);
        place(&mut board, pawn, PieceType::Pawn, Color::White);
        place(&mut board, square(3, 3), PieceType::Bishop, Color::White);

        let bpawn = square(5, 7);
        place(&mut board, bpawn, PieceType::Pawn, Color::Black);
        place(&mut board, square(4, 6), PieceType::Rook, Color::Black);
        board.card_state.traitor = Some(Color::White);

        board.compute_allowed_moves();
        let idx = square_to_index(bpawn);
        let moves = board.squares[idx]
            .as_ref()
            .unwrap()
            .move_set
            .allowed
            .clone();
        assert!(
            moves.contains(&square(4, 6)),
            "traitor pawn should capture own-side piece"
        );
    }

    #[test]
    fn test_bastion_saves_allied_piece() {
        let mut board = Board::default();
        for cell in board.squares.iter_mut() {
            cell.piece = None;
        }

        let bastion_sq = square(5, 4);
        place(&mut board, bastion_sq, PieceType::Rook, Color::White);
        let pawn_sq = square(4, 4);
        place(&mut board, pawn_sq, PieceType::Pawn, Color::White);
        let rook_sq = square(4, 8);
        place(&mut board, rook_sq, PieceType::Rook, Color::Black);

        board.card_state.bastion = Some((bastion_sq, Color::White));

        let result = board.apply_move(rook_sq, pawn_sq, None);
        assert!(result.is_ok());

        assert!(
            board.squares[square_to_index(bastion_sq)]
                .as_ref()
                .map_or(false, |p| p.piece_type == PieceType::Pawn),
            "captured pawn should be saved onto the bastion square"
        );

        assert!(
            board.squares[square_to_index(pawn_sq)]
                .as_ref()
                .map_or(false, |p| p.piece_type == PieceType::Rook
                    && p.color == Color::Black),
            "attacker should occupy the pawn square"
        );
    }

    // #[test]
    // fn test_magnetism_blocks_adjacent_pieces() {
    //     let mut board = Board::default();
    //     for cell in board.squares.iter_mut() {
    //         cell.piece = None;
    //     }

    //     let mag_sq = square(5, 4);
    //     place(&mut board, mag_sq, PieceType::Rook, Color::White);
    //     let pawn_sq = square(4, 4);
    //     place(&mut board, pawn_sq, PieceType::Pawn, Color::Black);
    //     board.card_state.magnetism = Some((mag_sq, Color::White));

    //     board.compute_allowed_moves();
    //     let idx = square_to_index(pawn_sq);
    //     let moves = board.squares[idx]
    //         .as_ref()
    //         .unwrap()
    //         .move_set
    //         .allowed
    //         .clone();
    //     assert!(
    //         !moves.contains(&square(4, 3)),
    //         "adjacent pawn should not move away from magnetised piece"
    //     );
    // }
}
