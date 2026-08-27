use std::sync::atomic::Ordering;
use std::time::Instant;

use crate::game::board::Color;
use crate::game::game_loop::GameLoop;

pub trait GameTimer {
    fn record_move(&self);
    fn get_times(&self) -> (u64, u64);
    fn reset_timer(&self);

    fn tick(&self) -> Option<Color>;
}

impl GameTimer for GameLoop {
    fn reset_timer(&self) {
        let initial = self.initial_time_ms.load(Ordering::SeqCst);
        self.white_time_ms.store(initial, Ordering::SeqCst);
        self.black_time_ms.store(initial, Ordering::SeqCst);
        self.timer_running.store(false, Ordering::SeqCst);
        *self.last_move_time.lock().unwrap() = Instant::now();
    }

    fn record_move(&self) {
        let mut last = self.last_move_time.lock().unwrap();
        let now = Instant::now();

        let timer_started = self.timer_running.load(Ordering::Relaxed);
        let is_white = self.is_white_turn.load(Ordering::Relaxed);

        if !timer_started && is_white {
            *last = now;
            self.timer_running.store(true, Ordering::Relaxed);
            self.first_move_played.store(true, Ordering::Relaxed);
        } else {
            let elapsed_ms = now.duration_since(*last).as_millis() as u64;
            let active_color = if is_white { Color::White } else { Color::Black };
            let multiplier = {
                let board = self.board.lock().unwrap();
                board.card_state.get_time_multiplier(active_color)
            };
            let elapsed_ms = elapsed_ms.saturating_mul(multiplier as u64);
            {
                let mut board = self.board.lock().unwrap();
                board.card_state.set_time_multiplier(active_color, 1);
            }
            *last = now;

            if is_white {
                let remaining = self
                    .white_time_ms
                    .load(Ordering::Relaxed)
                    .saturating_sub(elapsed_ms);
                self.white_time_ms.store(remaining, Ordering::Relaxed);
            } else {
                let remaining = self
                    .black_time_ms
                    .load(Ordering::Relaxed)
                    .saturating_sub(elapsed_ms);
                self.black_time_ms.store(remaining, Ordering::Relaxed);
            }
        }

        self.is_white_turn.store(!is_white, Ordering::Relaxed);
    }

    fn get_times(&self) -> (u64, u64) {
        (
            self.white_time_ms.load(Ordering::Relaxed),
            self.black_time_ms.load(Ordering::Relaxed),
        )
    }

    fn tick(&self) -> Option<Color> {
        let timer_started = self.timer_running.load(Ordering::Relaxed);
        if !timer_started {
            return None;
        }

        let mut last = self.last_move_time.lock().unwrap();
        let now = Instant::now();
        let elapsed_ms = now.duration_since(*last).as_millis() as u64;
        let active_color = if self.is_white_turn.load(Ordering::Relaxed) {
            Color::White
        } else {
            Color::Black
        };
        let multiplier = {
            let board = self.board.lock().unwrap();
            board.card_state.get_time_multiplier(active_color)
        };
        let elapsed_ms = elapsed_ms.saturating_mul(multiplier as u64);
        *last = now;

        let is_white = self.is_white_turn.load(Ordering::Relaxed);
        let (store, loser) = if is_white {
            (&self.white_time_ms, Color::White)
        } else {
            (&self.black_time_ms, Color::Black)
        };

        let remaining = store.load(Ordering::Relaxed).saturating_sub(elapsed_ms);
        store.store(remaining, Ordering::Relaxed);

        if remaining == 0 {
            Some(loser)
        } else {
            None
        }
    }
}
