#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PieceType {
    pub fn from_promotion_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "queen" | "q" => Some(PieceType::Queen),
            "rook" | "r" => Some(PieceType::Rook),
            "bishop" | "b" => Some(PieceType::Bishop),
            "knight" | "n" => Some(PieceType::Knight),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn other(self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveSet {
    pub default: Vec<(i8, i8)>,
    pub current: Vec<Square>,
    pub allowed: Vec<Square>,
}

impl MoveSet {
    pub fn new(default: Vec<(i8, i8)>) -> Self {
        Self {
            default,
            current: Vec::new(),
            allowed: Vec::new(),
        }
    }
}

impl serde::Serialize for MoveSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let coords: Vec<String> = self.allowed.iter().map(|sq| sq.to_coord()).collect();
        coords.serialize(serializer)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PieceModifier {
    Cannon {
        #[serde(skip_serializing_if = "Option::is_none")]
        rarity: Option<u8>,
    },
    Sniper {
        #[serde(skip_serializing_if = "Option::is_none")]
        rarity: Option<u8>,
    },
    VeteranKnight {
        #[serde(skip_serializing_if = "Option::is_none")]
        rarity: Option<u8>,
    },
    VeteranRook {
        #[serde(skip_serializing_if = "Option::is_none")]
        rarity: Option<u8>,
    },
    VeteranBishop {
        #[serde(skip_serializing_if = "Option::is_none")]
        rarity: Option<u8>,
    },
    Ninja {
        #[serde(skip_serializing_if = "Option::is_none")]
        rarity: Option<u8>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SquareModifier {
    Battlefield { rarity: u8 },
    DeadlyZone { rarity: u8 },
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: Color,
    pub move_set: MoveSet,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub piece_modifier: Option<PieceModifier>,
    #[serde(skip_serializing)]
    pub has_moved: bool,
}

impl Piece {
    pub fn new(piece_type: PieceType, color: Color) -> Self {
        use PieceType::*;

        let default_moves = match (piece_type, color) {
            (Pawn, Color::White) => vec![(0, 1), (0, 2), (1, 1), (-1, 1)],
            (Pawn, Color::Black) => vec![(0, -1), (0, -2), (1, -1), (-1, -1)],
            (Knight, _) => vec![
                (1, 2),
                (2, 1),
                (-1, 2),
                (-2, 1),
                (1, -2),
                (2, -1),
                (-1, -2),
                (-2, -1),
            ],
            (Bishop, _) => vec![(1, 1), (1, -1), (-1, 1), (-1, -1)],
            (Rook, _) => vec![(1, 0), (-1, 0), (0, 1), (0, -1)],
            (Queen, _) => vec![
                (1, 0),
                (-1, 0),
                (0, 1),
                (0, -1),
                (1, 1),
                (1, -1),
                (-1, 1),
                (-1, -1),
            ],
            (King, _) => vec![
                (1, 0),
                (-1, 0),
                (0, 1),
                (0, -1),
                (1, 1),
                (1, -1),
                (-1, 1),
                (-1, -1),
            ],
        };

        Self {
            piece_type,
            color,
            move_set: MoveSet::new(default_moves),
            piece_modifier: None,
            has_moved: false,
        }
    }

    pub fn set_modifier(&mut self, modifier: PieceModifier) {
        self.piece_modifier = Some(modifier);
    }

    pub fn is_cannon(&self) -> bool {
        matches!(self.piece_modifier, Some(PieceModifier::Cannon { .. }))
    }

    pub fn is_sniper(&self) -> bool {
        matches!(self.piece_modifier, Some(PieceModifier::Sniper { .. }))
    }

    pub fn is_veteran_knight(&self) -> bool {
        matches!(
            self.piece_modifier,
            Some(PieceModifier::VeteranKnight { .. })
        )
    }

    pub fn is_veteran_rook(&self) -> bool {
        matches!(self.piece_modifier, Some(PieceModifier::VeteranRook { .. }))
    }

    pub fn is_veteran_bishop(&self) -> bool {
        matches!(
            self.piece_modifier,
            Some(PieceModifier::VeteranBishop { .. })
        )
    }

    pub fn is_ninja(&self) -> bool {
        matches!(self.piece_modifier, Some(PieceModifier::Ninja { .. }))
    }
}

#[derive(Debug, Clone)]
pub struct SquareCell {
    pub piece: Option<Piece>,
    pub modifier: Option<SquareModifier>,
}

impl SquareCell {
    pub fn new() -> Self {
        Self {
            piece: None,
            modifier: None,
        }
    }

    pub fn as_ref(&self) -> Option<&Piece> {
        self.piece.as_ref()
    }

    pub fn as_mut(&mut self) -> Option<&mut Piece> {
        self.piece.as_mut()
    }

    pub fn is_some(&self) -> bool {
        self.piece.is_some()
    }

    pub fn is_none(&self) -> bool {
        self.piece.is_none()
    }

    pub fn take(&mut self) -> Option<Piece> {
        self.piece.take()
    }
}

impl Default for SquareCell {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
pub struct Square {
    pub file: u8,
    pub rank: u8,
}

impl Square {
    pub fn from_coord(coord: &str) -> Option<Self> {
        let bytes = coord.as_bytes();
        if bytes.len() != 2 {
            return None;
        }
        let file = bytes[0];
        let rank = bytes[1];

        if file < b'a' || file > b'h' || rank < b'1' || rank > b'8' {
            return None;
        }

        Some(Self {
            file: file - b'a' + 1,
            rank: rank - b'0',
        })
    }

    pub fn to_coord(self) -> String {
        let file = (b'a' + self.file - 1) as char;
        format!("{}{}", file, self.rank)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct CastlingRights {
    pub white_king_side: bool,
    pub white_queen_side: bool,
    pub black_king_side: bool,
    pub black_queen_side: bool,
}

impl Default for CastlingRights {
    fn default() -> Self {
        Self {
            white_king_side: true,
            white_queen_side: true,
            black_king_side: true,
            black_queen_side: true,
        }
    }
}
