use log::info;

use crate::game::board::apply::MoveApplier;
use crate::game::board::checks::CheckAnalyzer;
use crate::game::board::types::{Color, PieceType, Square};
use crate::game::board::utils::{index_to_square, square_add, square_to_index};
use crate::game::board::Board;

pub trait MoveCalculator {
    fn compute_allowed_moves(&mut self);
    fn compute_current_moves(&self, from: usize) -> Vec<Square>;
    fn is_move_allowed(&self, from: Square, to: Square) -> bool;
    fn has_legal_moves(&self, color: Color) -> bool;
    fn is_checkmate(&self, color: Color) -> bool;
}

impl MoveCalculator for Board {
    fn compute_allowed_moves(&mut self) {
        for i in 0..64 {
            if self.squares[i].is_some() {
                let current = self.compute_current_moves(i);
                let mut allowed = Vec::new();

                for target in &current {
                    if self.is_move_legal(i, *target) {
                        allowed.push(*target);
                    }
                }

                if !current.is_empty() && allowed.is_empty() {
                    let piece = self.squares[i].as_ref().unwrap();
                    info!(
                        "[MoveCalculator] piece at {} has {} current moves but 0 allowed (color={:?}, type={:?})",
                        index_to_square(i).to_coord(),
                        current.len(),
                        piece.color,
                        piece.piece_type
                    );
                }

                let piece = self.squares[i].as_mut().unwrap();
                piece.move_set.current = current;
                piece.move_set.allowed = allowed;
            }
        }
    }

    fn compute_current_moves(&self, from: usize) -> Vec<Square> {
        let piece = match self.squares[from].as_ref() {
            Some(p) => p,
            None => return Vec::new(),
        };

        let from_square = index_to_square(from);

        if piece.is_cannon() {
            return canon_moves(self, from_square, piece.color);
        }

        let mut moves = match piece.piece_type {
            PieceType::Pawn => pawn_moves(self, from_square, piece.color),
            PieceType::Knight => knight_moves(self, from_square, piece.color),
            PieceType::Bishop => {
                sliding_moves(self, from_square, &[(1, 1), (1, -1), (-1, 1), (-1, -1)])
            }
            PieceType::Rook => {
                sliding_moves(self, from_square, &[(1, 0), (-1, 0), (0, 1), (0, -1)])
            }
            PieceType::Queen => sliding_moves(
                self,
                from_square,
                &[
                    (1, 0),
                    (-1, 0),
                    (0, 1),
                    (0, -1),
                    (1, 1),
                    (1, -1),
                    (-1, 1),
                    (-1, -1),
                ],
            ),
            PieceType::King => king_moves(self, from_square, piece.color),
        };

        if piece.is_veteran_knight() {
            moves.extend(veteran_extra_moves(self, from_square, piece.color, &[(1, 0), (-1, 0), (0, 1), (0, -1)]));
        } else if piece.is_veteran_rook() {
            moves.extend(veteran_extra_moves(self, from_square, piece.color, &[(1, 1), (1, -1), (-1, 1), (-1, -1)]));
        } else if piece.is_veteran_bishop() {
            moves.extend(veteran_extra_moves(self, from_square, piece.color, &[(1, 0), (-1, 0), (0, 1), (0, -1)]));
        }

        if self.card_state.traitor == Some(piece.color.other()) && piece.piece_type == PieceType::Pawn {
            moves.extend(traitor_pawn_captures(self, from_square, piece.color));
        }

        moves.sort_by_key(|sq| (sq.file, sq.rank));
        moves.dedup();
        moves
    }

    fn is_move_allowed(&self, from: Square, to: Square) -> bool {
        let from_idx = square_to_index(from);
        let piece = match self.squares[from_idx].as_ref() {
            Some(p) => p,
            None => return false,
        };

        piece.move_set.allowed.iter().any(|sq| *sq == to)
    }

    fn has_legal_moves(&self, color: Color) -> bool {
        for i in 0..64 {
            if let Some(piece) = self.squares[i].as_ref() {
                if piece.color == color && !piece.move_set.allowed.is_empty() {
                    return true;
                }
            }
        }
        false
    }

    fn is_checkmate(&self, color: Color) -> bool {
        self.is_in_check(color) && !self.has_legal_moves(color)
    }
}

fn canon_moves(board: &Board, from: Square, color: Color) -> Vec<Square> {
    let mut moves = Vec::new();
    let directions = [(1, 0), (-1, 0), (0, 1), (0, -1)];

    for (df, dr) in directions {
        for step in 1..8 {
            let Some(target) = square_add(from, df * step, dr * step) else {
                break;
            };

            if board.is_deadly_zone(target) {
                break;
            }

            let target_idx = square_to_index(target);
            if let Some(piece) = board.squares[target_idx].as_ref() {
                if is_ninja_square(board, target) {
                    moves.push(target);
                    continue;
                }
                if piece.color != color && piece.piece_type != PieceType::King {
                    moves.push(target);
                }
            }
        }
    }

    moves
}

fn veteran_extra_moves(
    board: &Board,
    from: Square,
    color: Color,
    directions: &[(i8, i8)],
) -> Vec<Square> {
    directions
        .iter()
        .filter_map(|(df, dr)| square_add(from, *df, *dr))
        .filter(|target| is_empty_or_enemy(board, *target, color))
        .collect()
}

fn traitor_pawn_captures(board: &Board, from: Square, color: Color) -> Vec<Square> {
    let direction = if color == Color::White { 1 } else { -1 };
    let mut captures = Vec::new();

    for df in [-1, 1] {
        if let Some(target) = square_add(from, df, direction) {
            let target_idx = square_to_index(target);
            if let Some(piece) = board.squares[target_idx].as_ref() {
                if piece.color == color {
                    captures.push(target);
                }
            }
        }
    }

    captures
}

fn is_empty_or_enemy(board: &Board, square: Square, friendly_color: Color) -> bool {
    if board.is_deadly_zone(square) {
        return false;
    }
    match board.squares[square_to_index(square)].as_ref() {
        None => true,
        Some(piece) => piece.color != friendly_color,
    }
}

fn is_ninja_square(board: &Board, square: Square) -> bool {
    board
        .card_state
        .ninja
        .map_or(false, |(sq, _)| sq == square)
}

fn is_path_clear(board: &Board, from: Square, to: Square) -> bool {
    let df = (to.file as i8 - from.file as i8).signum();
    let dr = (to.rank as i8 - from.rank as i8).signum();

    let mut current = square_add(from, df, dr);
    while let Some(sq) = current {
        if sq == to {
            break;
        }
        let occupied = board.squares[square_to_index(sq)].is_some();
        let traversable = is_ninja_square(board, sq);
        if occupied && !traversable {
            return false;
        }
        if board.is_deadly_zone(sq) {
            return false;
        }
        current = square_add(sq, df, dr);
    }

    true
}

fn is_empty_square(board: &Board, square: Square) -> bool {
    !board.is_deadly_zone(square) && board.squares[square_to_index(square)].is_none()
}

fn pawn_moves(board: &Board, from: Square, color: Color) -> Vec<Square> {
    let direction = if color == Color::White { 1 } else { -1 };
    let start_rank = if color == Color::White { 2 } else { 7 };
    let mut moves = Vec::new();

    if let Some(one_forward) = square_add(from, 0, direction) {
        if is_empty_square(board, one_forward) {
            moves.push(one_forward);

            if from.rank == start_rank {
                if let Some(two_forward) = square_add(from, 0, 2 * direction) {
                    if is_empty_square(board, two_forward) {
                        moves.push(two_forward);
                    }
                }
            }
        }
    }

    for df in [-1, 1] {
        if let Some(capture) = square_add(from, df, direction) {
            if is_empty_or_enemy(board, capture, color) {
                let target_idx = square_to_index(capture);
                if let Some(target) = board.squares[target_idx].as_ref() {
                    if target.color != color {
                        moves.push(capture);
                    }
                }
            }
        }
    }

    moves.extend(en_passant_moves(board, from, color));
    moves
}

fn knight_moves(board: &Board, from: Square, color: Color) -> Vec<Square> {
    [
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
    .filter_map(|(df, dr)| square_add(from, *df, *dr))
    .filter(|target| is_empty_or_enemy(board, *target, color))
    .collect()
}

fn king_moves(board: &Board, from: Square, color: Color) -> Vec<Square> {
    let mut moves: Vec<Square> = [
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
    .filter_map(|(df, dr)| square_add(from, *df, *dr))
    .filter(|target| is_empty_or_enemy(board, *target, color))
    .collect();

    moves.extend(castling_moves(board, color));
    moves
}

fn sliding_moves(board: &Board, from: Square, directions: &[(i8, i8)]) -> Vec<Square> {
    let from_idx = square_to_index(from);
    let friendly_color = board.squares[from_idx].as_ref().unwrap().color;
    let mut moves = Vec::new();

    for (df, dr) in directions {
        for step in 1..8 {
            if let Some(target) = square_add(from, df * step, dr * step) {
                if board.is_deadly_zone(target) {
                    break;
                }
                let target_idx = square_to_index(target);
                if board.squares[target_idx].is_none() {
                    moves.push(target);
                } else {
                    if is_ninja_square(board, target) {
                        moves.push(target);
                        continue;
                    }
                    if board.squares[target_idx].as_ref().unwrap().color != friendly_color {
                        moves.push(target);
                    }
                    break;
                }
            } else {
                break;
            }
        }
    }

    moves
}

fn castling_moves(board: &Board, color: Color) -> Vec<Square> {
    let mut moves = Vec::new();
    if board.is_in_check(color) {
        return moves;
    }

    let (rank, rights) = match color {
        Color::White => (
            1,
            board.castling_rights.white_king_side || board.castling_rights.white_queen_side,
        ),
        Color::Black => (
            8,
            board.castling_rights.black_king_side || board.castling_rights.black_queen_side,
        ),
    };

    if !rights {
        return moves;
    }

    let king_square = Square { file: 5, rank };
    let enemy = color.other();

    let can_king_side = match color {
        Color::White => board.castling_rights.white_king_side,
        Color::Black => board.castling_rights.black_king_side,
    };
    if can_king_side && is_path_clear(board, king_square, Square { file: 8, rank }) {
        let f = Square { file: 6, rank };
        let g = Square { file: 7, rank };
        if !board.is_square_attacked(f, enemy) && !board.is_square_attacked(g, enemy) {
            moves.push(g);
        }
    }

    let can_queen_side = match color {
        Color::White => board.castling_rights.white_queen_side,
        Color::Black => board.castling_rights.black_queen_side,
    };
    if can_queen_side && is_path_clear(board, king_square, Square { file: 1, rank }) {
        let c = Square { file: 3, rank };
        let d = Square { file: 4, rank };
        if !board.is_square_attacked(c, enemy) && !board.is_square_attacked(d, enemy) {
            moves.push(c);
        }
    }

    moves
}

fn en_passant_moves(board: &Board, from: Square, color: Color) -> Vec<Square> {
    let mut moves = Vec::new();

    if let Some(target) = board.en_passant_target {
        let direction = if color == Color::White { 1 } else { -1 };
        let expected_rank = if color == Color::White { 5 } else { 4 };

        if from.rank == expected_rank && target.rank == from.rank + direction as u8 {
            if (from.file as i8 - target.file as i8).abs() == 1
                && !board.is_deadly_zone(target)
            {
                moves.push(target);
            }
        }
    }

    moves
}

pub trait MoveLegality {
    fn is_move_legal(&self, from: usize, to: Square) -> bool;
}

impl MoveLegality for Board {
    fn is_move_legal(&self, from: usize, to: Square) -> bool {
        let piece = match self.squares[from].as_ref() {
            Some(p) => p,
            None => return false,
        };

        if let Some(target) = self.squares[square_to_index(to)].as_ref() {
            if target.piece_type == PieceType::King {
                return false;
            }
        }

        if !self.battlefield_allows_capture(from, to) {
            return false;
        }

        if let Some((magnet_sq, magnet_color)) = self.card_state.magnetism {
            if piece.piece_type != PieceType::King {
                let from_sq = index_to_square(from);
                let is_around = magnet_sq != from_sq && is_adjacent(from_sq, magnet_sq);
                if is_around && to != magnet_sq {
                    return false;
                }
                if is_around && to == magnet_sq {
                    if self.squares[square_to_index(to)].is_none() {
                        return false;
                    }
                }
                let _ = magnet_color;
            }
        }

        let promotion = if piece.piece_type == PieceType::Pawn {
            let promotion_rank = if piece.color == Color::White { 8 } else { 1 };
            if to.rank == promotion_rank {
                Some(PieceType::Queen)
            } else {
                None
            }
        } else {
            None
        };

        let mut simulated = self.clone();
        if simulated
            .apply_move(index_to_square(from), to, promotion)
            .is_err()
        {
            return false;
        }

        !simulated.is_in_check(piece.color)
    }
}

fn is_adjacent(a: Square, b: Square) -> bool {
    (a.file as i8 - b.file as i8).abs() <= 1 && (a.rank as i8 - b.rank as i8).abs() <= 1
}

impl Board {
    fn battlefield_allows_capture(&self, from: usize, to: Square) -> bool {
        let to_idx = square_to_index(to);

        if !self.is_battlefield_square(to_idx) {
            return true;
        }

        let is_capture = self.squares[to_idx].is_some();

        if !is_capture {
            return true;
        }

        self.is_battlefield_square(from)
    }
}
