use crate::game::board::utils::square_to_index;
use crate::game::board::{Board, Color, PieceType};
use crate::game::cards::types::{CardDef, CardId, PieceRequirement};

pub fn all_card_defs() -> Vec<CardDef> {
    vec![
        CardDef {
            id: CardId::DeadlyZone,
            ends_turn: false,
            playable_if: vec![PieceRequirement::FreeNonPawnSquare],
        },
        CardDef {
            id: CardId::TimeBoost,
            ends_turn: false,
            playable_if: vec![],
        },
        CardDef {
            id: CardId::RussianRoulette,
            ends_turn: true,
            playable_if: vec![PieceRequirement::AnyNonKing],
        },
        CardDef {
            id: CardId::Journey,
            ends_turn: true,
            playable_if: vec![
                PieceRequirement::Own(PieceType::Rook),
                PieceRequirement::Enemy(PieceType::Knight),
            ],
        },
        CardDef {
            id: CardId::Fog,
            ends_turn: false,
            playable_if: vec![],
        },
        CardDef {
            id: CardId::CrazyKnight,
            ends_turn: true,
            playable_if: vec![
                PieceRequirement::Own(PieceType::Knight),
                PieceRequirement::Own(PieceType::Bishop),
            ],
        },
        CardDef {
            id: CardId::BeastWork,
            ends_turn: true,
            playable_if: vec![
                PieceRequirement::Own(PieceType::Knight),
                PieceRequirement::Own(PieceType::Rook),
            ],
        },
        CardDef {
            id: CardId::FuriousMason,
            ends_turn: true,
            playable_if: vec![
                PieceRequirement::Own(PieceType::Bishop),
                PieceRequirement::Own(PieceType::Rook),
            ],
        },
        CardDef {
            id: CardId::CatchTheKnightThief,
            ends_turn: true,
            playable_if: vec![
                PieceRequirement::Own(PieceType::Knight),
                PieceRequirement::Enemy(PieceType::Bishop),
            ],
        },
        CardDef {
            id: CardId::BeastOfBurden,
            ends_turn: true,
            playable_if: vec![
                PieceRequirement::Own(PieceType::Knight),
                PieceRequirement::Enemy(PieceType::Rook),
            ],
        },
        CardDef {
            id: CardId::Bestification,
            ends_turn: true,
            playable_if: vec![
                PieceRequirement::Own(PieceType::Bishop),
                PieceRequirement::Enemy(PieceType::Knight),
            ],
        },
        CardDef {
            id: CardId::HermitArchitect,
            ends_turn: true,
            playable_if: vec![
                PieceRequirement::Own(PieceType::Bishop),
                PieceRequirement::Enemy(PieceType::Rook),
            ],
        },
        CardDef {
            id: CardId::Pyromaniac,
            ends_turn: true,
            playable_if: vec![
                PieceRequirement::Own(PieceType::Rook),
                PieceRequirement::Enemy(PieceType::Bishop),
            ],
        },
        CardDef {
            id: CardId::Cannon,
            ends_turn: true,
            playable_if: vec![PieceRequirement::Own(PieceType::Rook)],
        },
        CardDef {
            id: CardId::Sniper,
            ends_turn: true,
            playable_if: vec![PieceRequirement::Own(PieceType::Bishop)],
        },
        CardDef {
            id: CardId::Trash,
            ends_turn: false,
            playable_if: vec![],
        },
        CardDef {
            id: CardId::PushBack,
            ends_turn: false,
            playable_if: vec![],
        },
        CardDef {
            id: CardId::Battlefield,
            ends_turn: true,
            playable_if: vec![PieceRequirement::OwnNonKing],
        },
        CardDef {
            id: CardId::Garbage,
            ends_turn: false,
            playable_if: vec![],
        },
        CardDef {
            id: CardId::Annihilation,
            ends_turn: true,
            playable_if: vec![PieceRequirement::AnyNonKing],
        },
        CardDef {
            id: CardId::VeteranKnight,
            ends_turn: false,
            playable_if: vec![PieceRequirement::Own(PieceType::Knight)],
        },
        CardDef {
            id: CardId::VeteranRook,
            ends_turn: false,
            playable_if: vec![PieceRequirement::Own(PieceType::Rook)],
        },
        CardDef {
            id: CardId::VeteranBishop,
            ends_turn: false,
            playable_if: vec![PieceRequirement::Own(PieceType::Bishop)],
        },
        CardDef {
            id: CardId::Frog,
            ends_turn: true,
            playable_if: vec![PieceRequirement::Own(PieceType::Pawn)],
        },
        CardDef {
            id: CardId::WheelOfFortune,
            ends_turn: false,
            playable_if: vec![],
        },
        // CardDef {
        //     id: CardId::Magnetism,
        //     ends_turn: false,
        //     playable_if: vec![],
        // },
        // CardDef {
        //     id: CardId::Bastion,
        //     ends_turn: false,
        //     playable_if: vec![],
        // },
        CardDef {
            id: CardId::Ninja,
            ends_turn: false,
            playable_if: vec![PieceRequirement::OwnNonKing],
        },
        CardDef {
            id: CardId::Traitor,
            ends_turn: false,
            playable_if: vec![],
        },
        CardDef {
            id: CardId::Breakthrough,
            ends_turn: true,
            playable_if: vec![PieceRequirement::Own(PieceType::Pawn)],
        },
        CardDef {
            id: CardId::DesperateRescue,
            ends_turn: true,
            playable_if: vec![],
        },
    ]
}

pub fn card_def(id: CardId) -> CardDef {
    all_card_defs()
        .into_iter()
        .find(|def| def.id == id)
        .expect("unknown card id")
}

pub fn is_card_playable(board: &Board, player: Color, card: CardId) -> bool {
    if card == CardId::PushBack {
        return board.card_state.last_move.as_ref().map_or(false, |lm| {
            lm.captured.as_ref().map_or(false, |p| p.color == player)
        });
    }

    // if card == CardId::Magnetism || card == CardId::Bastion {
    //     return board.card_state.last_move.as_ref().map_or(false, |lm| {
    //         lm.moved.color == player
    //             && board.squares[square_to_index(lm.to)]
    //                 .as_ref()
    //                 .map_or(false, |p| p.color == player)
    //     });
    // }

    let def = card_def(card);
    if def.playable_if.is_empty() {
        return true;
    }
    def.playable_if.iter().all(|req| match req {
        PieceRequirement::Own(piece) => board.squares.iter().any(|s| {
            s.as_ref()
                .map_or(false, |p| p.color == player && p.piece_type == *piece)
        }),
        PieceRequirement::Enemy(piece) => {
            let enemy = player.other();
            board.squares.iter().any(|s| {
                s.as_ref()
                    .map_or(false, |p| p.color == enemy && p.piece_type == *piece)
            })
        }
        PieceRequirement::AnyNonKing => board.squares.iter().any(|s| {
            s.as_ref()
                .map_or(false, |p| p.piece_type != PieceType::King)
        }),
        PieceRequirement::OwnNonKing => board.squares.iter().any(|s| {
            s.as_ref().map_or(false, |p| {
                p.color == player && p.piece_type != PieceType::King
            })
        }),
        PieceRequirement::FreeNonPawnSquare => board
            .squares
            .iter()
            .any(|s| s.as_ref().map_or(true, |p| p.piece_type != PieceType::Pawn)),
    })
}
