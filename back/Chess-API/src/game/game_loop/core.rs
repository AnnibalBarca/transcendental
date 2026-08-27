use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Instant;

use log::info;
use rand::seq::SliceRandom;

use crate::game::board::{Board, Card, Color};
use crate::game::cards::{
    CardEffectApplier, CardId, CardResult, CardTarget, hand_to_json, is_card_playable,
};
use crate::game::game_loop::GameTimer;

const INITIAL_TIME_MS: u64 = 600_000;
const FIRST_MOVE_TIMEOUT_SECS: u64 = 20;
const INITIAL_HAND_SIZE: usize = 5;

pub type CardInstance = (CardId, u8);

const USE_DEBUG_DECK: bool = false;

fn debug_deck() -> Vec<CardInstance> {
    vec![
        (CardId::Battlefield, 2),
        (CardId::Battlefield, 1),
        (CardId::Battlefield, 1),
        (CardId::Battlefield, 2),
        (CardId::Battlefield, 2),
        (CardId::Cannon, 2),
        (CardId::Sniper, 1),
        (CardId::Trash, 1),
        (CardId::PushBack, 2),
        (CardId::Battlefield, 2),
        (CardId::Cannon, 2),
        (CardId::Sniper, 1),
        (CardId::Trash, 1),
        (CardId::PushBack, 2),
        (CardId::Battlefield, 2),
    ]
}

fn default_common_deck() -> Vec<CardInstance> {
    let mut deck = Vec::new();
    for id in ["1", "2", "3", "5", "6", "7", "8", "9", "10", "11"] {
        if let Some(c) = CardId::from_str(id) {
            deck.push((c, 0));
            deck.push((c, 0));
        }
    }
    deck
}

#[derive(Debug)]
pub struct GameLoop {
    pub counter: AtomicU64,
    pub running: AtomicBool,
    pub started: AtomicBool,
    pub ended: AtomicBool,
    pub is_white_turn: AtomicBool,
    pub board: Mutex<Board>,
    #[allow(dead_code)]
    pub white_card: Mutex<Vec<Card>>,
    #[allow(dead_code)]
    pub black_card: Mutex<Vec<Card>>,
    #[allow(dead_code)]
    pub global_card: Mutex<Vec<Card>>,
    pub global_deck: Mutex<Vec<CardInstance>>,
    pub discard_pile: Mutex<Vec<CardInstance>>,
    pub forced_draws: Mutex<HashMap<Color, Vec<CardInstance>>>,
    pub white_hand: Mutex<Vec<CardInstance>>,
    pub black_hand: Mutex<Vec<CardInstance>>,
    pub card_played_this_turn: Mutex<Option<CardInstance>>,
    pub custom_deck: Mutex<Option<Vec<CardInstance>>>,
    pub white_time_ms: AtomicU64,
    pub black_time_ms: AtomicU64,
    pub last_move_time: Mutex<Instant>,
    pub timer_running: AtomicBool,
    pub first_move_played: AtomicBool,
    pub first_move_deadline: Mutex<Instant>,
    pub action_taken_this_turn: AtomicBool,
    pub initial_time_ms: AtomicU64,
    #[allow(dead_code)]
    pub white_vote_draw: AtomicBool,
    #[allow(dead_code)]
    pub black_vote_draw: AtomicBool,
}

impl Default for GameLoop {
    fn default() -> Self {
        Self {
            counter: AtomicU64::new(0),
            running: AtomicBool::new(false),
            started: AtomicBool::new(false),
            ended: AtomicBool::new(false),
            is_white_turn: AtomicBool::new(true),
            board: Mutex::new(Board::default()),
            white_card: Mutex::new(Vec::new()),
            black_card: Mutex::new(Vec::new()),
            global_card: Mutex::new(Vec::new()),
            global_deck: Mutex::new(Vec::new()),
            discard_pile: Mutex::new(Vec::new()),
            forced_draws: Mutex::new(HashMap::new()),
            white_hand: Mutex::new(Vec::new()),
            black_hand: Mutex::new(Vec::new()),
            card_played_this_turn: Mutex::new(None),
            custom_deck: Mutex::new(None),
            white_time_ms: AtomicU64::new(INITIAL_TIME_MS),
            black_time_ms: AtomicU64::new(INITIAL_TIME_MS),
            last_move_time: Mutex::new(Instant::now()),
            timer_running: AtomicBool::new(false),
            first_move_played: AtomicBool::new(false),
            first_move_deadline: Mutex::new(Instant::now()),
            action_taken_this_turn: AtomicBool::new(false),
            initial_time_ms: AtomicU64::new(INITIAL_TIME_MS),
            white_vote_draw: AtomicBool::new(false),
            black_vote_draw: AtomicBool::new(false),
        }
    }
}

impl GameLoop {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_initial_time(&self, ms: u64) {
        self.initial_time_ms.store(ms, Ordering::SeqCst);
        self.white_time_ms.store(ms, Ordering::SeqCst);
        self.black_time_ms.store(ms, Ordering::SeqCst);
    }

    pub fn set_custom_deck(&self, deck: Vec<CardInstance>) {
        *self.custom_deck.lock().unwrap() = Some(deck);
    }

    pub fn start(&self) {
        self.running
            .store(true, std::sync::atomic::Ordering::SeqCst);
        self.started
            .store(true, std::sync::atomic::Ordering::SeqCst);
        *self.first_move_deadline.lock().unwrap() =
            Instant::now() + std::time::Duration::from_secs(FIRST_MOVE_TIMEOUT_SECS);
        let hands_empty = self.white_hand.lock().unwrap().is_empty()
            && self.black_hand.lock().unwrap().is_empty();
        if hands_empty {
            self.shuffle_and_deal();
        }
    }

    pub fn end(&self) {
        self.running
            .store(false, std::sync::atomic::Ordering::SeqCst);
        self.ended
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn restart(&self) {
        self.counter.store(0, std::sync::atomic::Ordering::SeqCst);
        self.started
            .store(false, std::sync::atomic::Ordering::SeqCst);
        self.ended
            .store(false, std::sync::atomic::Ordering::SeqCst);
        self.is_white_turn
            .store(true, std::sync::atomic::Ordering::SeqCst);
        self.reset_timer();
        self.first_move_played
            .store(false, std::sync::atomic::Ordering::SeqCst);
        *self.board.lock().unwrap() = Board::default();
        *self.global_deck.lock().unwrap() = Vec::new();
        *self.discard_pile.lock().unwrap() = Vec::new();
        *self.forced_draws.lock().unwrap() = HashMap::new();
        *self.white_hand.lock().unwrap() = Vec::new();
        *self.black_hand.lock().unwrap() = Vec::new();
        *self.card_played_this_turn.lock().unwrap() = None;
        self.action_taken_this_turn
            .store(false, std::sync::atomic::Ordering::SeqCst);
    }

    fn shuffle_and_deal(&self) {
        let custom = self.custom_deck.lock().unwrap().take();
        let (deck, source) = match custom {
            Some(mut d) => {
                let mut rng = rand::thread_rng();
                d.shuffle(&mut rng);
                (d, "players_deck".to_string())
            }
            None => {
                if USE_DEBUG_DECK {
                    (debug_deck(), "debug_deck".to_string())
                } else {
                    let mut d = default_common_deck();
                    let mut rng = rand::thread_rng();
                    d.shuffle(&mut rng);
                    (d, "global_fallback".to_string())
                }
            }
        };

        info!(
            "[GameLoop] shuffle_and_deal source={} deck_size={} deck={:?}",
            source,
            deck.len(),
            deck
        );

        let white_initial: Vec<CardInstance> =
            deck.iter().take(INITIAL_HAND_SIZE).copied().collect();
        let black_initial: Vec<CardInstance> = deck
            .iter()
            .skip(INITIAL_HAND_SIZE)
            .take(INITIAL_HAND_SIZE)
            .copied()
            .collect();

        let remaining: Vec<CardInstance> = deck.into_iter().skip(INITIAL_HAND_SIZE * 2).collect();

        *self.white_hand.lock().unwrap() = white_initial;
        *self.black_hand.lock().unwrap() = black_initial;
        *self.global_deck.lock().unwrap() = remaining;
        *self.discard_pile.lock().unwrap() = Vec::new();

        info!(
            "[GameLoop] Deck dealt. Remaining in global deck: {}",
            self.global_deck.lock().unwrap().len()
        );
    }

    fn recycle_discard_if_needed(&self) {
        let mut deck = self.global_deck.lock().unwrap();
        if deck.is_empty() {
            let mut discard = self.discard_pile.lock().unwrap();
            if !discard.is_empty() {
                info!("[GameLoop] Global deck empty, recycling discard pile.");
                *deck = discard.drain(..).collect();
                let mut rng = rand::thread_rng();
                deck.shuffle(&mut rng);
            }
        }
    }

    pub fn draw_card(&self, color: Color) -> Option<CardInstance> {
        {
            let mut forced = self.forced_draws.lock().unwrap();
            if let Some(queue) = forced.get_mut(&color) {
                if !queue.is_empty() {
                    let card = queue.remove(0);
                    drop(forced);
                    self.hand_for(color).push(card);
                    info!("[GameLoop] {:?} drew forced card {:?}", color, card.0);
                    return Some(card);
                }
            }
        }

        self.recycle_discard_if_needed();
        let mut deck = self.global_deck.lock().unwrap();
        if deck.is_empty() {
            return None;
        }
        let (card, rarity) = deck.remove(0);
        match color {
            Color::White => self.white_hand.lock().unwrap().push((card, rarity)),
            Color::Black => self.black_hand.lock().unwrap().push((card, rarity)),
        }
        info!(
            "[GameLoop] {:?} drew card {:?} rarity {}",
            color, card, rarity
        );
        Some((card, rarity))
    }

    pub fn discard_card(&self, card_id: CardId, player: Color) -> Result<u8, &'static str> {
        let current_turn = if self.is_white_turn.load(Ordering::Relaxed) {
            Color::White
        } else {
            Color::Black
        };

        if player != current_turn {
            return Err("not your turn");
        }

        if self.action_taken_this_turn.load(Ordering::Relaxed) {
            return Err("you already used your card action this turn");
        }

        let card_rarity = {
            let hand = self.hand_for(player);
            let inst = hand.iter().find(|(id, _)| *id == card_id).copied();
            match inst {
                Some((_, rarity)) => rarity,
                None => return Err("card not in hand"),
            }
        };

        {
            let mut hand = self.hand_for(player);
            if let Some(pos) = hand.iter().position(|(id, _)| *id == card_id) {
                hand.remove(pos);
            }
        }

        if card_id != CardId::Garbage {
            self.discard_pile.lock().unwrap().push((card_id, card_rarity));
        }

        self.draw_card(player);
        self.action_taken_this_turn
            .store(true, Ordering::SeqCst);

        info!(
            "[GameLoop] {:?} discarded card {:?} rarity {}",
            player, card_id, card_rarity
        );
        Ok(card_rarity)
    }

    fn hand_for(&self, color: Color) -> std::sync::MutexGuard<'_, Vec<CardInstance>> {
        match color {
            Color::White => self.white_hand.lock().unwrap(),
            Color::Black => self.black_hand.lock().unwrap(),
        }
    }

    pub fn play_card(
        &self,
        card_id: CardId,
        player: Color,
        target: CardTarget,
    ) -> Result<(CardResult, u8), &'static str> {
        let current_turn = if self.is_white_turn.load(Ordering::Relaxed) {
            Color::White
        } else {
            Color::Black
        };

        if player != current_turn {
            return Err("not your turn");
        }

        if self.card_played_this_turn.lock().unwrap().is_some() {
            return Err("you already played a card this turn");
        }

        if self.action_taken_this_turn.load(Ordering::Relaxed) {
            return Err("you already used your card action this turn");
        }

        let card_rarity = {
            let hand = self.hand_for(player);
            let inst = hand.iter().find(|(id, _)| *id == card_id).copied();
            match inst {
                Some((_, rarity)) => rarity,
                None => return Err("card not in hand"),
            }
        };

        let mut board = self.board.lock().unwrap();
        let result = if is_card_playable(&board, player, card_id) {
            board.apply_card_effect(card_id, card_rarity, player, target)?
        } else {
            CardResult::new("card_no_effect")
        };
        drop(board);

        if card_id == CardId::Trash {
            let opponent = player.other();
            let mut forced = self.forced_draws.lock().unwrap();
            let queue = forced.entry(opponent).or_default();
            queue.push((CardId::Garbage, 0));
            queue.push((CardId::Garbage, 0));
            info!(
                "[GameLoop] {:?} played Poubelle: 2 garbage cards queued for {:?}",
                player, opponent
            );
        }

        if card_id == CardId::WheelOfFortune {
            self.discard_all_and_redraw(player);
        }

        {
            let mut hand = self.hand_for(player);
            if let Some(pos) = hand.iter().position(|(id, _)| *id == card_id) {
                hand.remove(pos);
            }
        }

        *self.card_played_this_turn.lock().unwrap() = Some((card_id, card_rarity));
        self.action_taken_this_turn
            .store(true, Ordering::SeqCst);

        info!(
            "[GameLoop] {:?} played card {:?} rarity {}",
            player, card_id, card_rarity
        );
        Ok((result, card_rarity))
    }

    pub fn discard_and_draw_if_needed(&self, color: Color) {
        const HAND_SIZE: usize = 5;
        if let Some((card_id, rarity)) = self.card_played_this_turn.lock().unwrap().take() {
            info!("[GameLoop] Discarding card {:?} for {:?}", card_id, color);
            if card_id != CardId::Garbage {
                self.discard_pile.lock().unwrap().push((card_id, rarity));
            }
        }

        self.action_taken_this_turn
            .store(false, std::sync::atomic::Ordering::SeqCst);

        let hand_len = match color {
            Color::White => self.white_hand.lock().unwrap().len(),
            Color::Black => self.black_hand.lock().unwrap().len(),
        };
        let to_draw = HAND_SIZE.saturating_sub(hand_len);
        info!(
            "[GameLoop] {:?} hand has {} cards, drawing {} cards",
            color, hand_len, to_draw
        );
        for _ in 0..to_draw {
            let drawn = self.draw_card(color);
            info!("[GameLoop] Drew card for {:?}: {:?}", color, drawn);
        }
    }

    pub fn discard_all_and_redraw(&self, _player: Color) {
        const HAND_SIZE: usize = 5;

        let all: Vec<(Color, CardInstance)> = {
            let mut white = self.white_hand.lock().unwrap();
            let mut black = self.black_hand.lock().unwrap();
            let mut cards = Vec::new();
            cards.extend(white.drain(..).map(|c| (Color::White, c)));
            cards.extend(black.drain(..).map(|c| (Color::Black, c)));
            cards
        };

        {
            let mut discard = self.discard_pile.lock().unwrap();
            discard.extend(all.into_iter().map(|(_, c)| c));
        }

        info!("[GameLoop] RoueDeLaFortune: both hands discarded");

        for _ in 0..HAND_SIZE {
            let drawn = self.draw_card(Color::White);
            info!("[GameLoop] RoueDeLaFortune drew for White: {:?}", drawn);
        }
        for _ in 0..HAND_SIZE {
            let drawn = self.draw_card(Color::Black);
            info!("[GameLoop] RoueDeLaFortune drew for Black: {:?}", drawn);
        }
    }

    pub fn get_hands(&self) -> (Vec<CardInstance>, Vec<CardInstance>) {
        (
            self.white_hand.lock().unwrap().clone(),
            self.black_hand.lock().unwrap().clone(),
        )
    }

    pub fn hand_json_for(&self, color: Color) -> serde_json::Value {
        let board = self.board.lock().unwrap();
        let hand = self.hand_for(color);
        hand_to_json(&hand, &board, color)
    }

    pub fn get_time_multipliers(&self) -> (u8, u8) {
        let board = self.board.lock().unwrap();
        (
            board.card_state.get_time_multiplier(Color::White),
            board.card_state.get_time_multiplier(Color::Black),
        )
    }
}
