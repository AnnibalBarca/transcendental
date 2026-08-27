use crate::game::board::types::Square;

pub fn square_add(square: Square, df: i8, dr: i8) -> Option<Square> {
    let file = square.file as i8 + df;
    let rank = square.rank as i8 + dr;

    if file >= 1 && file <= 8 && rank >= 1 && rank <= 8 {
        Some(Square {
            file: file as u8,
            rank: rank as u8,
        })
    } else {
        None
    }
}

pub fn index_to_square(index: usize) -> Square {
    Square {
        file: (index % 8) as u8 + 1,
        rank: (index / 8) as u8 + 1,
    }
}

pub fn square_to_index(square: Square) -> usize {
    ((square.rank - 1) * 8 + (square.file - 1)) as usize
}
