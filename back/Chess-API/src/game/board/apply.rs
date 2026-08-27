use crate::game::board::Board;
use crate::game::board::types::{Color, Piece, PieceType, Square};
use crate::game::board::utils::{square_add, square_to_index};

pub trait MoveApplier {
    fn apply_move(
        &mut self,
        from: Square,
        to: Square,
        promotion: Option<PieceType>,
    ) -> Result<(), &'static str>;
}

impl MoveApplier for Board {
    fn apply_move(
        &mut self,
        from: Square,
        to: Square,
        promotion: Option<PieceType>,
    ) -> Result<(), &'static str> {
        let from_idx = square_to_index(from);
        let to_idx = square_to_index(to);

        let battlefield_before = self.card_state.battlefield;
        let bastion_before = self.card_state.bastion;

        let piece = self.squares[from_idx]
            .take()
            .ok_or("no piece at source square")?;

        let from_was_cannon = piece.is_cannon();
        let capture_happened = self.squares[to_idx].is_some();

        let bastion_saved = if capture_happened && bastion_before.is_some() {
            let (bastion_sq, bastion_color) = bastion_before.unwrap();
            let captured = self.squares[to_idx].as_ref().unwrap().clone();
            let bastion_idx = square_to_index(bastion_sq);

            if captured.color == bastion_color
                && captured.piece_type != PieceType::King
                && bastion_sq != to
                && self.squares[bastion_idx].is_some()
            {
                let bastion_piece = self.squares[bastion_idx].take().unwrap();
                if bastion_piece.piece_type != PieceType::King {
                    self.squares[to_idx].take();
                    self.captured_pieces.push(bastion_piece);
                    self.squares[bastion_idx].piece = Some(captured);
                    true
                } else {
                    self.squares[bastion_idx].piece = Some(bastion_piece);
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        let capture_happened = capture_happened && !bastion_saved;

        if let Some(captured) = self.squares[to_idx].take() {
            self.captured_pieces.push(captured);
        }

        if piece.piece_type == PieceType::Pawn {
            if let Some(ep_target) = self.en_passant_target {
                if to == ep_target && from.file != to.file {
                    let captured_square = Square {
                        file: to.file,
                        rank: from.rank,
                    };
                    let captured_idx = square_to_index(captured_square);
                    if let Some(captured) = self.squares[captured_idx].take() {
                        self.captured_pieces.push(captured);
                        if let Some((center, _)) = battlefield_before {
                            if center == captured_square {
                                self.clear_battlefield();
                            }
                        }
                    }
                }
            }
        }

        let stays_in_place = from_was_cannon || (piece.is_sniper() && capture_happened);

        if stays_in_place {
            self.squares[from_idx].piece = Some(piece);
            return Ok(());
        }

        update_castling_rights(self, from, &piece);

        if piece.piece_type == PieceType::King && (from.file as i8 - to.file as i8).abs() == 2 {
            apply_castling_rook_move(self, &piece, to);
        }

        update_en_passant(self, from, to, &piece);

        let final_piece = if piece.piece_type == PieceType::Pawn {
            let promotion_rank = if piece.color == Color::White { 8 } else { 1 };
            if to.rank == promotion_rank {
                let promote_to = promotion.ok_or("promotion required")?;
                Piece::new(promote_to, piece.color)
            } else {
                piece
            }
        } else {
            piece
        };

        let mut final_piece = final_piece;
        final_piece.has_moved = true;

        self.squares[to_idx].piece = Some(final_piece);

        if let Some((center, _)) = battlefield_before {
            if center == from {
                self.move_battlefield(center, to);
            } else if center == to && capture_happened {
                self.clear_battlefield();
            }
        }

        // if bastion_saved {
        //     self.card_state.bastion = None;
        // } else if let Some((center, _)) = bastion_before {
        //     if center == from {
        //         self.card_state.bastion = None;
        //     } else if center == to {
        //         self.card_state.bastion = None;
        //     }
        // }

        // if let Some((center, _)) = self.card_state.magnetism {
        //     if center == from || center == to {
        //         self.card_state.magnetism = None;
        //     }
        // }

        if let Some((center, _)) = self.card_state.ninja {
            if center == to {
                self.card_state.ninja = None;
            }
        }

        Ok(())
    }
}

fn update_castling_rights(board: &mut Board, from: Square, piece: &Piece) {
    use Color::*;
    use PieceType::*;

    match (piece.piece_type, piece.color) {
        (King, White) => {
            board.castling_rights.white_king_side = false;
            board.castling_rights.white_queen_side = false;
        }
        (King, Black) => {
            board.castling_rights.black_king_side = false;
            board.castling_rights.black_queen_side = false;
        }
        (Rook, White) => {
            if from == (Square { file: 1, rank: 1 }) {
                board.castling_rights.white_queen_side = false;
            } else if from == (Square { file: 8, rank: 1 }) {
                board.castling_rights.white_king_side = false;
            }
        }
        (Rook, Black) => {
            if from == (Square { file: 1, rank: 8 }) {
                board.castling_rights.black_queen_side = false;
            } else if from == (Square { file: 8, rank: 8 }) {
                board.castling_rights.black_king_side = false;
            }
        }
        _ => {}
    }
}

fn apply_castling_rook_move(board: &mut Board, king: &Piece, king_to: Square) {
    let rank = if king.color == Color::White { 1 } else { 8 };

    let (from_file, to_file) = if king_to.file == 7 { (8, 6) } else { (1, 4) };

    let from_idx = square_to_index(Square {
        file: from_file,
        rank,
    });
    let to_idx = square_to_index(Square {
        file: to_file,
        rank,
    });

    if let Some(rook) = board.squares[from_idx].take() {
        board.squares[to_idx].piece = Some(rook);
    }
}

fn update_en_passant(board: &mut Board, from: Square, to: Square, piece: &Piece) {
    if piece.piece_type == PieceType::Pawn && (from.rank as i8 - to.rank as i8).abs() == 2 {
        let direction = if piece.color == Color::White { 1 } else { -1 };
        board.en_passant_target = square_add(from, 0, direction);
    } else {
        board.en_passant_target = None;
    }
}
