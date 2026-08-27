use serde_json::json;

use crate::game::board::{Board, Color, PieceType, Square};
use crate::game::cards::registry::is_card_playable;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardId {
    DeadlyZone,
    TimeBoost,
    RussianRoulette,
    Journey,
    Fog,
    CrazyKnight,
    BeastWork,
    FuriousMason,
    CatchTheKnightThief,
    BeastOfBurden,
    Bestification,
    HermitArchitect,
    Pyromaniac,
    Cannon,
    Sniper,
    Trash,
    PushBack,
    Battlefield,
    Garbage,
    Annihilation,
    VeteranKnight,
    VeteranRook,
    VeteranBishop,
    Frog,
    WheelOfFortune,
    // Magnetism,
    // Bastion,
    Ninja,
    Traitor,
    Breakthrough,
    DesperateRescue,
}

impl CardId {
    pub fn as_str(self) -> &'static str {
        match self {
            CardId::DeadlyZone => "0",
            CardId::TimeBoost => "1",
            CardId::RussianRoulette => "2",
            CardId::Journey => "3",
            CardId::Fog => "4",
            CardId::CrazyKnight => "5",
            CardId::BeastWork => "6",
            CardId::FuriousMason => "7",
            CardId::CatchTheKnightThief => "8",
            CardId::BeastOfBurden => "9",
            CardId::Bestification => "10",
            CardId::HermitArchitect => "11",
            CardId::Pyromaniac => "12",
            CardId::Cannon => "13",
            CardId::Sniper => "14",
            CardId::Trash => "15",
            CardId::PushBack => "16",
            CardId::Battlefield => "17",
            CardId::Garbage => "18",
            CardId::Annihilation => "19",
            CardId::VeteranKnight => "20",
            CardId::VeteranRook => "21",
            CardId::VeteranBishop => "22",
            CardId::Frog => "23",
            CardId::WheelOfFortune => "24",
            // CardId::Magnetism => "25",
            // CardId::Bastion => "26",
            CardId::Ninja => "27",
            CardId::Traitor => "28",
            CardId::Breakthrough => "29",
            CardId::DesperateRescue => "30",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "0" => Some(CardId::DeadlyZone),
            "1" => Some(CardId::TimeBoost),
            "2" => Some(CardId::RussianRoulette),
            "3" => Some(CardId::Journey),
            "4" => Some(CardId::Fog),
            "5" => Some(CardId::CrazyKnight),
            "6" => Some(CardId::BeastWork),
            "7" => Some(CardId::FuriousMason),
            "8" => Some(CardId::CatchTheKnightThief),
            "9" => Some(CardId::BeastOfBurden),
            "10" => Some(CardId::Bestification),
            "11" => Some(CardId::HermitArchitect),
            "12" => Some(CardId::Pyromaniac),
            "13" => Some(CardId::Cannon),
            "14" => Some(CardId::Sniper),
            "15" => Some(CardId::Trash),
            "16" => Some(CardId::PushBack),
            "17" => Some(CardId::Battlefield),
            "18" => Some(CardId::Garbage),
            "19" => Some(CardId::Annihilation),
            "20" => Some(CardId::VeteranKnight),
            "21" => Some(CardId::VeteranRook),
            "22" => Some(CardId::VeteranBishop),
            "23" => Some(CardId::Frog),
            "24" => Some(CardId::WheelOfFortune),
            // "25" => Some(CardId::Magnetism),
            // "26" => Some(CardId::Bastion),
            "27" => Some(CardId::Ninja),
            "28" => Some(CardId::Traitor),
            "29" => Some(CardId::Breakthrough),
            "30" => Some(CardId::DesperateRescue),
            _ => None,
        }
    }
}

pub fn hand_to_json(hand: &[(CardId, u8)], board: &Board, color: Color) -> serde_json::Value {
    serde_json::Value::Array(
        hand.iter()
            .map(|(c, rarity)| {
                json!({
                    "id": c.as_str(),
                    "rarity": rarity,
                    "playable": is_card_playable(board, color, *c),
                })
            })
            .collect(),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardTarget {
    None,
    Square(Square),
    #[allow(dead_code)]
    OwnPieceType(PieceType),
    #[allow(dead_code)]
    EnemyPieceType(PieceType),
    #[allow(dead_code)]
    PieceType(PieceType),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct EffectMarker {
    pub kind: String,
    pub square: String,
}

#[derive(Debug, Clone)]
pub struct CardResult {
    pub message_id: String,
    pub effects: Vec<EffectMarker>,
}

impl CardResult {
    pub fn new(message_id: impl Into<String>) -> Self {
        Self {
            message_id: message_id.into(),
            effects: Vec::new(),
        }
    }

    pub fn with_effect(mut self, kind: impl Into<String>, square: Square) -> Self {
        self.effects.push(EffectMarker {
            kind: kind.into(),
            square: square.to_coord(),
        });
        self
    }

    pub fn with_effects(mut self, kind: impl Into<String> + Copy, squares: Vec<Square>) -> Self {
        for square in squares {
            self.effects.push(EffectMarker {
                kind: kind.into(),
                square: square.to_coord(),
            });
        }
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceRequirement {
    Own(PieceType),
    Enemy(PieceType),
    AnyNonKing,
    OwnNonKing,
    FreeNonPawnSquare,
}

#[derive(Debug, Clone)]
pub struct CardDef {
    pub id: CardId,
    pub ends_turn: bool,
    pub playable_if: Vec<PieceRequirement>,
}
