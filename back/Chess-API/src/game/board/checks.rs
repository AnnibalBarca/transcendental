use crate::game::board::types::{Color, PieceType, Square};
use crate::game::board::utils::{index_to_square, square_add, square_to_index};
use crate::game::board::Board;

pub trait CheckAnalyzer {
    fn is_in_check(&self, color: Color) -> bool;
    fn find_king(&self, color: Color) -> Option<Square>;
    fn is_square_attacked(&self, square: Square, by_color: Color) -> bool;
    fn can_piece_attack(&self, from: usize, target: Square) -> bool;
}

impl CheckAnalyzer for Board {
    fn is_in_check(&self, color: Color) -> bool {
        let king_square = match self.find_king(color) {
            Some(sq) => sq,
            None => return false,
        };

        self.is_square_attacked(king_square, color.other())
    }

    fn find_king(&self, color: Color) -> Option<Square> {
        for i in 0..64 {
            if let Some(piece) = self.squares[i].as_ref() {
                if piece.piece_type == PieceType::King && piece.color == color {
                    return Some(index_to_square(i));
                }
            }
        }
        None
    }

    fn is_square_attacked(&self, square: Square, by_color: Color) -> bool {
        for i in 0..64 {
            if let Some(piece) = self.squares[i].as_ref() {
                if piece.color == by_color && self.can_piece_attack(i, square) {
                    return true;
                }
            }
        }
        false
    }

    fn can_piece_attack(&self, from: usize, target: Square) -> bool {
        let piece = match self.squares[from].as_ref() {
            Some(p) => p,
            None => return false,
        };

        let from_square = index_to_square(from);

        if piece.is_cannon() {
            return false;
        }

        match piece.piece_type {
            PieceType::Pawn => {
                let direction = if piece.color == Color::White { 1 } else { -1 };
                [-1, 1]
                    .iter()
                    .filter_map(|df| square_add(from_square, *df, direction))
                    .any(|sq| sq == target)
            }
            PieceType::Knight => [
                (1, 2),
                (2, 1),
                (-1, 2),
                (-2, 1),
                (1, -2),
                (2, -1),
                (-1, -2),
                (-2, -1),
            ]
            .iter()
            .filter_map(|(df, dr)| square_add(from_square, *df, *dr))
            .any(|sq| sq == target),
            PieceType::King => [
                (1, 0),
                (-1, 0),
                (0, 1),
                (0, -1),
                (1, 1),
                (1, -1),
                (-1, 1),
                (-1, -1),
            ]
            .iter()
            .filter_map(|(df, dr)| square_add(from_square, *df, *dr))
            .any(|sq| sq == target),
            PieceType::Bishop | PieceType::Rook | PieceType::Queen => {
                let directions = match piece.piece_type {
                    PieceType::Bishop => vec![(1, 1), (1, -1), (-1, 1), (-1, -1)],
                    PieceType::Rook => vec![(1, 0), (-1, 0), (0, 1), (0, -1)],
                    PieceType::Queen => vec![
                        (1, 0),
                        (-1, 0),
                        (0, 1),
                        (0, -1),
                        (1, 1),
                        (1, -1),
                        (-1, 1),
                        (-1, -1),
                    ],
                    _ => unreachable!(),
                };

                for (df, dr) in directions {
                    for step in 1..8 {
                        if let Some(sq) = square_add(from_square, df * step, dr * step) {
                            if sq == target {
                                return true;
                            }
                            if self.squares[square_to_index(sq)].is_some()
                                && !self.is_ninja_square(sq)
                            {
                                break;
                            }
                            if self.is_deadly_zone(sq) {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                }
                false
            }
        }
    }
}
