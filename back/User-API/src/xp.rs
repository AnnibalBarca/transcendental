pub const MAX_LEVEL: i64 = 99;
pub const MAX_XP: i64 = 2_425_500;

pub fn xp_for_level(level: i64) -> i64 {
    500 * level * (level - 1) / 2
}

pub fn level_from_xp(xp: i64) -> i64 {
    if xp <= 0 {
        return 1;
    }
    let n = (1.0 + (1.0 + 8.0 * xp as f64 / 500.0).sqrt()) / 2.0;
    (n.floor() as i64).min(MAX_LEVEL)
}

pub fn xp_for_next_level(level: i64) -> i64 {
    500 * level
}

pub fn level_progress(xp: i64, level: i64) -> f64 {
    if level >= MAX_LEVEL {
        return 100.0;
    }
    let current_xp = xp - xp_for_level(level);
    let needed = xp_for_next_level(level);
    if needed <= 0 {
        return 0.0;
    }
    (current_xp as f64 / needed as f64 * 100.0).clamp(0.0, 100.0)
}
