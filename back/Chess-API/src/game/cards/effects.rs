use log::info;
use rand::seq::SliceRandom;

use crate::game::board::apply::MoveApplier;
use crate::game::board::checks::CheckAnalyzer;
use crate::game::board::moves::MoveCalculator;
use crate::game::board::utils::{index_to_square, square_add, square_to_index};
use crate::game::board::{Board, Color, PieceModifier, PieceType, Square, SquareModifier};
use crate::game::cards::state::record_last_move;
use crate::game::cards::types::{CardId, CardResult, CardTarget};

pub trait CardEffectApplier {
    fn apply_card_effect(
        &mut self,
        card: CardId,
        rarity: u8,
        player: Color,
        target: CardTarget,
    ) -> Result<CardResult, &'static str>;
}

impl CardEffectApplier for Board {
    fn apply_card_effect(
        &mut self,
        card: CardId,
        rarity: u8,
        player: Color,
        target: CardTarget,
    ) -> Result<CardResult, &'static str> {
        match card {
            CardId::DeadlyZone => apply_deadly_zone(self, rarity, target),
            CardId::TimeBoost => apply_time_boost(self, player),
            CardId::RussianRoulette => apply_russian_roulette(self),
            CardId::Journey => apply_journey(self, player),
            CardId::Fog => apply_fog(self, player, rarity),
            CardId::CrazyKnight => apply_swap(
                self,
                player,
                PieceType::Knight,
                PieceType::Bishop,
                false,
                "swap_knight_bishop",
            ),
            CardId::BeastWork => apply_swap(
                self,
                player,
                PieceType::Knight,
                PieceType::Rook,
                false,
                "swap_knight_rook",
            ),
            CardId::FuriousMason => apply_swap(
                self,
                player,
                PieceType::Bishop,
                PieceType::Rook,
                false,
                "swap_bishop_rook",
            ),
            CardId::CatchTheKnightThief => apply_swap(
                self,
                player,
                PieceType::Knight,
                PieceType::Bishop,
                true,
                "swap_knight_enemy_bishop",
            ),
            CardId::BeastOfBurden => apply_swap(
                self,
                player,
                PieceType::Knight,
                PieceType::Rook,
                true,
                "swap_knight_enemy_rook",
            ),
            CardId::Bestification => apply_swap(
                self,
                player,
                PieceType::Bishop,
                PieceType::Knight,
                true,
                "swap_bishop_enemy_knight",
            ),
            CardId::HermitArchitect => apply_swap(
                self,
                player,
                PieceType::Bishop,
                PieceType::Rook,
                true,
                "swap_bishop_enemy_rook",
            ),
            CardId::Pyromaniac => apply_swap(
                self,
                player,
                PieceType::Rook,
                PieceType::Bishop,
                true,
                "swap_rook_enemy_bishop",
            ),
            CardId::Cannon => apply_cannon(self, player, rarity, target),
            CardId::Sniper => apply_sniper(self, player, rarity),
            CardId::Trash => Ok(CardResult::new("trash_played")),
            CardId::PushBack => apply_push_back(self, player),
            CardId::Battlefield => apply_battlefield(self, player, rarity, target),
            CardId::Garbage => Ok(CardResult::new("card_no_effect")),
            CardId::Annihilation => apply_annihilation(self, player, target),
            CardId::VeteranKnight => apply_veteran(
                self,
                player,
                target,
                PieceType::Knight,
                rarity,
                "veteran_knight",
            ),
            CardId::VeteranRook => apply_veteran(
                self,
                player,
                target,
                PieceType::Rook,
                rarity,
                "veteran_rook",
            ),
            CardId::VeteranBishop => apply_veteran(
                self,
                player,
                target,
                PieceType::Bishop,
                rarity,
                "veteran_bishop",
            ),
            CardId::Frog => apply_frog(self, player, target),
            CardId::WheelOfFortune => Ok(CardResult::new("wheel_of_fortune")),
            // CardId::Magnetism => apply_magnetism(self, player),
            // CardId::Bastion => apply_bastion(self, player),
            CardId::Ninja => apply_ninja(self, player, target, rarity),
            CardId::Traitor => apply_traitor(self, player),
            CardId::Breakthrough => apply_breakthrough(self, player, target),
            CardId::DesperateRescue => apply_desperate_rescue(self, player, target),
        }
    }
}

fn apply_deadly_zone(
    board: &mut Board,
    rarity: u8,
    target: CardTarget,
) -> Result<CardResult, &'static str> {
    use crate::game::board::utils::square_to_index;

    let CardTarget::Square(square) = target else {
        return Err("err_need_target");
    };

    if square.file < 1 || square.file > 8 || square.rank < 1 || square.rank > 8 {
        return Err("err_invalid_square");
    }

    if let Some(piece) = board.squares[square_to_index(square)].as_ref() {
        if piece.piece_type == PieceType::Pawn {
            return Err("err_pawn_square");
        }
    }

    board.squares[square_to_index(square)].modifier = Some(SquareModifier::DeadlyZone { rarity });
    Ok(CardResult::new("deadly_zone_placed").with_effect("zone", square))
}

fn apply_time_boost(board: &mut Board, player: Color) -> Result<CardResult, &'static str> {
    let opponent = player.other();
    board.card_state.set_time_multiplier(opponent, 2);
    Ok(CardResult::new("time_boost_activated"))
}

fn apply_russian_roulette(board: &mut Board) -> Result<CardResult, &'static str> {
    let mut candidates: Vec<usize> = (0..64)
        .filter(|&idx| {
            if let Some(piece) = board.squares[idx].as_ref() {
                piece.piece_type != PieceType::King
            } else {
                false
            }
        })
        .collect();

    if candidates.is_empty() {
        return Err("err_no_eligible_piece");
    }

    let mut rng = rand::thread_rng();
    candidates.shuffle(&mut rng);
    let victim_idx = candidates[0];
    let victim_square = index_to_square(victim_idx);

    if let Some(victim) = board.squares[victim_idx].take() {
        board.captured_pieces.push(victim);
        info!(
            "[RouletteRusse] piece removed at {}",
            victim_square.to_coord()
        );
    }

    Ok(
        CardResult::new(format!("russian_roulette_fired {}", victim_square.to_coord()))
            .with_effect("kill", victim_square),
    )
}

fn apply_journey(board: &mut Board, player: Color) -> Result<CardResult, &'static str> {
    let enemy = player.other();

    let own_rook_idx = find_piece_index(board, player, PieceType::Rook);
    let enemy_knight_idx = find_piece_index(board, enemy, PieceType::Knight);

    if own_rook_idx.is_none() {
        return Err("err_no_rook");
    }
    if enemy_knight_idx.is_none() {
        return Err("err_no_knight");
    }

    let own_rook_idx = own_rook_idx.unwrap();
    let enemy_knight_idx = enemy_knight_idx.unwrap();

    let own_rook_square = index_to_square(own_rook_idx);
    let enemy_knight_square = index_to_square(enemy_knight_idx);

    let own_rook = board.squares[own_rook_idx].take().unwrap();
    let enemy_knight = board.squares[enemy_knight_idx].take().unwrap();

    board.squares[own_rook_idx].piece = Some(enemy_knight);
    board.squares[enemy_knight_idx].piece = Some(own_rook);

    info!(
        "[Voyage] swapped positions between {} (rook) and {} (enemy knight)",
        own_rook_square.to_coord(),
        enemy_knight_square.to_coord()
    );

    Ok(CardResult::new(format!(
        "journey_swapped {} {}",
        own_rook_square.to_coord(),
        enemy_knight_square.to_coord()
    ))
    .with_effects("swap", vec![own_rook_square, enemy_knight_square]))
}

fn apply_fog(board: &mut Board, player: Color, rarity: u8) -> Result<CardResult, &'static str> {
    board.card_state.start_fog(player, 4, rarity);
    Ok(CardResult::new(format!("fog_activated {}", rarity)))
}

fn apply_swap(
    board: &mut Board,
    player: Color,
    own_type: PieceType,
    other_type: PieceType,
    other_is_enemy: bool,
    message_id: &'static str,
) -> Result<CardResult, &'static str> {
    let other_color = if other_is_enemy {
        player.other()
    } else {
        player
    };

    let own_idx = find_piece_index(board, player, own_type).ok_or("err_no_own_piece")?;
    let other_idx =
        find_piece_index(board, other_color, other_type).ok_or("err_no_target_piece")?;

    let own = board.squares[own_idx].take().unwrap();
    let other = board.squares[other_idx].take().unwrap();
    board.squares[own_idx].piece = Some(other);
    board.squares[other_idx].piece = Some(own);

    let own_square = index_to_square(own_idx);
    let other_square = index_to_square(other_idx);

    Ok(
        CardResult::new(format!("{} {} {}", message_id, own_idx, other_idx))
            .with_effects("swap", vec![own_square, other_square]),
    )
}

fn find_piece_index(board: &Board, color: Color, piece_type: PieceType) -> Option<usize> {
    for idx in 0..64 {
        if let Some(piece) = board.squares[idx].as_ref() {
            if piece.color == color && piece.piece_type == piece_type {
                return Some(idx);
            }
        }
    }
    None
}

fn apply_cannon(
    board: &mut Board,
    player: Color,
    rarity: u8,
    target: CardTarget,
) -> Result<CardResult, &'static str> {
    use crate::game::board::utils::square_to_index;

    let CardTarget::Square(square) = target else {
        return Err("err_need_target");
    };

    let idx = square_to_index(square);

    let slot = board.squares[idx]
        .as_ref()
        .ok_or("err_no_piece_on_square")?;
    if slot.color != player {
        return Err("err_not_your_piece");
    }
    if slot.piece_type != PieceType::Rook {
        return Err("err_canon_requires_rook");
    }
    if slot.is_cannon() {
        return Err("err_already_cannon");
    }

    board.squares[idx]
        .as_mut()
        .unwrap()
        .set_modifier(PieceModifier::Cannon {
            rarity: Some(rarity),
        });
    info!("[Canon] piece at {} became a cannon", square.to_coord());
    Ok(CardResult::new("cannon_activated").with_effect("upgrade", square))
}

fn apply_sniper(board: &mut Board, player: Color, rarity: u8) -> Result<CardResult, &'static str> {
    for slot in board.squares.iter_mut() {
        if let Some(piece) = slot.piece.as_mut() {
            if piece.color == player && piece.piece_type == PieceType::Bishop {
                piece.set_modifier(PieceModifier::Sniper {
                    rarity: Some(rarity),
                });
            }
        }
    }
    board.card_state.set_sniper(player, 3);
    info!("[Sniper] {:?} bishops are now snipers", player);
    Ok(CardResult::new("sniper_activated"))
}

fn apply_push_back(board: &mut Board, player: Color) -> Result<CardResult, &'static str> {
    use crate::game::board::utils::square_to_index;

    let last = board
        .card_state
        .last_move
        .clone()
        .ok_or("err_no_last_move")?;

    let captured = last.captured.as_ref().ok_or("err_no_piece_captured")?;
    if captured.color != player {
        return Err("err_no_piece_captured");
    }

    let from_idx = square_to_index(last.from);
    let to_idx = square_to_index(last.to);

    board.squares[to_idx].piece = board.captured_pieces.pop().or(Some(captured.clone()));
    board.squares[from_idx].piece = Some(last.moved.clone());

    board.card_state.clear_last_move();
    info!(
        "[Repousser] undid last move {} -> {}",
        last.from.to_coord(),
        last.to.to_coord()
    );
    Ok(CardResult::new("push_back_undone")
        .with_effect("undo_from", last.from)
        .with_effect("undo_to", last.to))
}

fn apply_battlefield(
    board: &mut Board,
    player: Color,
    rarity: u8,
    target: CardTarget,
) -> Result<CardResult, &'static str> {
    use crate::game::board::utils::square_to_index;

    let CardTarget::Square(square) = target else {
        return Err("err_need_target");
    };

    let idx = square_to_index(square);

    let slot = board.squares[idx]
        .as_ref()
        .ok_or("err_no_piece_on_square")?;
    if slot.color != player {
        return Err("err_not_your_piece");
    }
    if slot.piece_type == PieceType::King {
        return Err("err_cannot_battlefield_king");
    }

    board.card_state.battlefield = Some((square, rarity));

    for df in -1i8..=1 {
        for dr in -1i8..=1 {
            let file = square.file as i8 + df;
            let rank = square.rank as i8 + dr;
            if file < 1 || file > 8 || rank < 1 || rank > 8 {
                continue;
            }
            let target = Square {
                file: file as u8,
                rank: rank as u8,
            };
            let target_idx = square_to_index(target);

            if !board.is_deadly_zone_square(target_idx) {
                board.squares[target_idx].modifier = Some(SquareModifier::Battlefield { rarity });
            }
        }
    }

    info!(
        "[ChampDeBataille] battlefield created at {}",
        square.to_coord()
    );
    let mut markers = Vec::new();
    for df in -1i8..=1 {
        for dr in -1i8..=1 {
            let file = square.file as i8 + df;
            let rank = square.rank as i8 + dr;
            if file < 1 || file > 8 || rank < 1 || rank > 8 {
                continue;
            }
            markers.push(Square {
                file: file as u8,
                rank: rank as u8,
            });
        }
    }
    Ok(CardResult::new("battlefield_activated").with_effects("battlefield", markers))
}

fn apply_annihilation(
    board: &mut Board,
    player: Color,
    target: CardTarget,
) -> Result<CardResult, &'static str> {
    use crate::game::board::utils::square_to_index;

    let CardTarget::Square(square) = target else {
        return Err("err_need_target");
    };

    let idx = square_to_index(square);
    let slot = board.squares[idx]
        .as_ref()
        .ok_or("err_no_piece_on_square")?;
    if slot.piece_type != PieceType::Pawn {
        return Err("err_anihilation_requires_pawn");
    }

    let pawn = board.squares[idx].take().unwrap();
    board.captured_pieces.push(pawn);
    info!(
        "[Anihilation] {} removed a pawn at {}",
        if player == Color::White {
            "White"
        } else {
            "Black"
        },
        square.to_coord()
    );
    Ok(CardResult::new("annihilation_done").with_effect("kill", square))
}

fn apply_veteran(
    board: &mut Board,
    player: Color,
    target: CardTarget,
    piece_type: PieceType,
    rarity: u8,
    message_id: &'static str,
) -> Result<CardResult, &'static str> {
    use crate::game::board::utils::square_to_index;

    let CardTarget::Square(square) = target else {
        return Err("err_need_target");
    };

    let idx = square_to_index(square);
    let slot = board.squares[idx]
        .as_ref()
        .ok_or("err_no_piece_on_square")?;
    if slot.color != player {
        return Err("err_not_your_piece");
    }
    if slot.piece_type != piece_type {
        return Err("err_veteran_wrong_piece");
    }
    if slot.has_moved {
        return Err("err_veteran_piece_moved");
    }

    let modifier = match piece_type {
        PieceType::Knight => PieceModifier::VeteranKnight {
            rarity: Some(rarity),
        },
        PieceType::Rook => PieceModifier::VeteranRook {
            rarity: Some(rarity),
        },
        PieceType::Bishop => PieceModifier::VeteranBishop {
            rarity: Some(rarity),
        },
        _ => return Err("err_veteran_wrong_piece"),
    };
    board.squares[idx].as_mut().unwrap().set_modifier(modifier);
    info!(
        "[Veteran] {} piece upgraded at {}",
        if player == Color::White {
            "White"
        } else {
            "Black"
        },
        square.to_coord()
    );
    Ok(CardResult::new(message_id).with_effect("upgrade", square))
}

fn apply_frog(
    board: &mut Board,
    player: Color,
    target: CardTarget,
) -> Result<CardResult, &'static str> {
    let CardTarget::Square(from) = target else {
        return Err("err_need_target");
    };

    let from_idx = square_to_index(from);
    let slot = board.squares[from_idx]
        .as_ref()
        .ok_or("err_no_piece_on_square")?;
    if slot.color != player {
        return Err("err_not_your_piece");
    }
    if slot.piece_type != PieceType::Pawn {
        return Err("err_frog_requires_pawn");
    }

    let direction = if player == Color::White { 1 } else { -1 };

    let one_forward = square_add(from, 0, direction).ok_or("err_frog_no_move")?;
    if board.squares[square_to_index(one_forward)].is_none() {
        return Err("err_frog_no_move");
    }
    let dest = square_add(from, 0, 2 * direction).ok_or("err_frog_no_move")?;
    if board.squares[square_to_index(dest)].is_some() || board.is_deadly_zone(dest) {
        return Err("err_frog_no_move");
    }

    let mut sim = board.clone();
    sim.apply_move(from, dest, None)
        .map_err(|_| "err_frog_no_move")?;
    if sim.is_in_check(player) {
        return Err("err_frog_no_move");
    }

    record_last_move(board, from, dest);
    board
        .apply_move(from, dest, None)
        .map_err(|_| "err_frog_no_move")?;

    info!(
        "[Frog] {} pawn jumped from {} to {}",
        if player == Color::White {
            "White"
        } else {
            "Black"
        },
        from.to_coord(),
        dest.to_coord()
    );
    Ok(CardResult::new("frog_moved").with_effect("frog_move", dest))
}

// fn apply_magnetism(board: &mut Board, player: Color) -> Result<CardResult, &'static str> {
//     let last = board
//         .card_state
//         .last_move
//         .clone()
//         .ok_or("err_no_last_move")?;
//     board.card_state.magnetism = Some((last.to, player));
//     info!(
//         "[Magnetisme] {} magnetised square {}",
//         if player == Color::White {
//             "White"
//         } else {
//             "Black"
//         },
//         last.to.to_coord()
//     );
//     Ok(CardResult::new("magnetism_activated").with_effect("magnet", last.to))
// }

// fn apply_bastion(board: &mut Board, player: Color) -> Result<CardResult, &'static str> {
//     let last = board
//         .card_state
//         .last_move
//         .clone()
//         .ok_or("err_no_last_move")?;
//     let idx = crate::game::board::utils::square_to_index(last.to);
//     let slot = board.squares[idx]
//         .as_ref()
//         .ok_or("err_no_piece_on_square")?;
//     if slot.piece_type == PieceType::King {
//         return Err("err_bastion_king");
//     }
//     board.card_state.bastion = Some((last.to, player));
//     info!(
//         "[Bastion] {} bastion at {}",
//         if player == Color::White {
//             "White"
//         } else {
//             "Black"
//         },
//         last.to.to_coord()
//     );
//     Ok(CardResult::new("bastion_activated").with_effect("bastion", last.to))
// }

fn apply_ninja(
    board: &mut Board,
    player: Color,
    target: CardTarget,
    rarity: u8,
) -> Result<CardResult, &'static str> {
    use crate::game::board::utils::square_to_index;

    let CardTarget::Square(square) = target else {
        return Err("err_need_target");
    };

    let idx = square_to_index(square);
    let slot = board.squares[idx]
        .as_ref()
        .ok_or("err_no_piece_on_square")?;
    if slot.color != player {
        return Err("err_not_your_piece");
    }
    if slot.piece_type == PieceType::King {
        return Err("err_ninja_king");
    }

    board.squares[idx]
        .as_mut()
        .unwrap()
        .set_modifier(PieceModifier::Ninja {
            rarity: Some(rarity),
        });
    board.card_state.ninja = Some((square, player));
    info!(
        "[Ninja] {} piece is now a ninja at {}",
        if player == Color::White {
            "White"
        } else {
            "Black"
        },
        square.to_coord()
    );
    Ok(CardResult::new("ninja_activated").with_effect("ninja", square))
}

fn apply_traitor(board: &mut Board, player: Color) -> Result<CardResult, &'static str> {
    board.card_state.traitor = Some(player);
    info!(
        "[Traitor] {} activates traitor",
        if player == Color::White {
            "White"
        } else {
            "Black"
        }
    );
    Ok(CardResult::new("traitor_activated"))
}

fn apply_breakthrough(
    board: &mut Board,
    player: Color,
    target: CardTarget,
) -> Result<CardResult, &'static str> {
    let CardTarget::Square(from) = target else {
        return Err("err_need_target");
    };

    let from_idx = square_to_index(from);
    let slot = board.squares[from_idx]
        .as_ref()
        .ok_or("err_no_piece_on_square")?;
    if slot.color != player {
        return Err("err_not_your_piece");
    }
    if slot.piece_type != PieceType::Pawn {
        return Err("err_percee_requires_pawn");
    }

    let direction = if player == Color::White { 1 } else { -1 };
    let promotion_rank = if player == Color::White { 8 } else { 1 };

    let mut dest = None;
    for distance in [3, 2, 1] {
        let mut ok = true;
        let mut landing = None;
        for step in 1..=distance {
            let Some(sq) = square_add(from, 0, step * direction) else {
                ok = false;
                break;
            };
            if board.squares[square_to_index(sq)].is_some() || board.is_deadly_zone(sq) {
                ok = false;
                break;
            }
            landing = Some(sq);
        }
        if ok && landing.is_some() {
            dest = landing;
            break;
        }
    }

    let dest = dest.ok_or("err_percee_no_move")?;

    let promotion = if dest.rank == promotion_rank {
        Some(PieceType::Queen)
    } else {
        None
    };

    let mut sim = board.clone();
    sim.apply_move(from, dest, promotion)
        .map_err(|_| "err_percee_no_move")?;
    if sim.is_in_check(player) {
        return Err("err_percee_no_move");
    }

    record_last_move(board, from, dest);
    board
        .apply_move(from, dest, promotion)
        .map_err(|_| "err_percee_no_move")?;

    info!(
        "[Percee] {} pawn advanced from {} to {}",
        if player == Color::White {
            "White"
        } else {
            "Black"
        },
        from.to_coord(),
        dest.to_coord()
    );
    Ok(CardResult::new("breakthrough_moved").with_effect("breakthrough_move", dest))
}

fn apply_desperate_rescue(
    board: &mut Board,
    player: Color,
    target: CardTarget,
) -> Result<CardResult, &'static str> {
    use crate::game::board::utils::square_to_index;

    let king_square = board.find_king(player).ok_or("err_no_king")?;

    if board.is_in_check(player) && board.is_checkmate(player) {
        return Err("err_sauvetage_checkmate");
    }

    let CardTarget::Square(square) = target else {
        return Err("err_need_target");
    };

    let idx = square_to_index(square);
    if board.squares[idx].is_some() {
        return Err("err_sauvetage_not_free");
    }

    if board.is_square_attacked(square, player.other()) {
        return Err("err_sauvetage_attacked");
    }

    let king = board.squares[square_to_index(king_square)].take().unwrap();
    board.squares[idx].piece = Some(king);
    info!(
        "[SauvetageInespere] {} king moved {} -> {}",
        if player == Color::White {
            "White"
        } else {
            "Black"
        },
        king_square.to_coord(),
        square.to_coord()
    );
    Ok(CardResult::new("desperate_rescue_done")
        .with_effect("move_from", king_square)
        .with_effect("move_to", square))
}
