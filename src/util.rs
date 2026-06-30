//! Square geometry and helpers. Port of chessground `util.ts`.

use crate::types::{Color, Key, Piece, Pos, PosAndKey, FILES, RANKS};

pub fn same_piece(a: &Piece, b: &Piece) -> bool {
    a.color == b.color && a.role == b.role && a.promoted == b.promoted
}

pub const INV_RANKS: [char; 8] = ['8', '7', '6', '5', '4', '3', '2', '1'];

pub fn all_keys() -> Vec<Key> {
    let mut keys = Vec::with_capacity(64);
    for file in FILES {
        for rank in RANKS {
            keys.push(Key::new(&format!("{file}{rank}")).unwrap());
        }
    }
    keys
}

pub fn key_to_pos(key: &Key) -> Pos {
    crate::types::key_to_pos(key).expect("valid key")
}

pub fn pos_to_key(pos: Pos) -> Option<Key> {
    crate::types::pos_to_key(pos)
}

pub fn pos_to_key_unsafe(pos: Pos) -> Key {
    pos_to_key(pos).expect("valid pos")
}

pub fn all_pos() -> Vec<Pos> {
    all_keys().iter().map(key_to_pos).collect()
}

pub fn all_pos_and_key() -> Vec<PosAndKey> {
    all_keys()
        .into_iter()
        .map(|key| PosAndKey {
            pos: key_to_pos(&key),
            key,
        })
        .collect()
}

pub fn diff(a: i32, b: i32) -> i32 {
    (a - b).abs()
}

pub fn knight_dir(x1: i32, y1: i32, x2: i32, y2: i32) -> bool {
    diff(x1, x2) * diff(y1, y2) == 2
}

pub fn rook_dir(x1: i32, y1: i32, x2: i32, y2: i32) -> bool {
    (x1 == x2) != (y1 == y2)
}

pub fn bishop_dir(x1: i32, y1: i32, x2: i32, y2: i32) -> bool {
    diff(x1, x2) == diff(y1, y2) && x1 != x2
}

pub fn queen_dir(x1: i32, y1: i32, x2: i32, y2: i32) -> bool {
    rook_dir(x1, y1, x2, y2) || bishop_dir(x1, y1, x2, y2)
}

pub fn king_dir_non_castling(x1: i32, y1: i32, x2: i32, y2: i32) -> bool {
    diff(x1, x2).max(diff(y1, y2)) == 1
}

pub fn pawn_dir_advance(x1: i32, y1: i32, x2: i32, y2: i32, white: bool) -> bool {
    let step = if white { 1 } else { -1 };
    x1 == x2
        && (y2 == y1 + step
            || (y2 == y1 + 2 * step && (if white { y1 <= 1 } else { y1 >= 6 })))
}

pub fn distance_sq(pos1: Pos, pos2: Pos) -> i32 {
    let dx = pos1.file as i32 - pos2.file as i32;
    let dy = pos1.rank as i32 - pos2.rank as i32;
    dx * dx + dy * dy
}

/// Board bounds in pixel space.
#[derive(Clone, Copy, Debug)]
pub struct BoardBounds {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

/// Map a pointer position to a board key (chessground `getKeyAtDomPos`).
pub fn get_key_at_pos(pos: (f32, f32), as_white: bool, bounds: BoardBounds) -> Option<Key> {
    let mut file = ((8.0 * (pos.0 - bounds.left)) / bounds.width).floor() as i32;
    if !as_white {
        file = 7 - file;
    }
    let mut rank = 7 - ((8.0 * (pos.1 - bounds.top)) / bounds.height).floor() as i32;
    if !as_white {
        rank = 7 - rank;
    }
    if (0..8).contains(&file) && (0..8).contains(&rank) {
        pos_to_key(Pos {
            file: file as u8,
            rank: rank as u8,
        })
    } else {
        None
    }
}

pub fn white_pov(orientation: Color) -> bool {
    orientation == Color::White
}

fn same_pos(a: Pos, b: Pos) -> bool {
    a.file == b.file && a.rank == b.rank
}

pub fn square_center(key: &Key, as_white: bool, bounds: BoardBounds) -> (f32, f32) {
    let pos = key_to_pos(key);
    let mut file = pos.file as f32;
    let mut rank = pos.rank as f32;
    if !as_white {
        file = 7.0 - file;
        rank = 7.0 - rank;
    }
    let screen_rank = 7.0 - rank;
    let sq = bounds.width / 8.0;
    (
        bounds.left + (file + 0.5) * sq,
        bounds.top + (screen_rank + 0.5) * sq,
    )
}

/// Nearest valid arrow snap square (chessground `getSnappedKeyAtDomPos`).
pub fn get_snapped_key_at_pos(
    orig: &Key,
    pos: (f32, f32),
    as_white: bool,
    bounds: BoardBounds,
) -> Option<Key> {
    let orig_pos = key_to_pos(orig);
    let valid: Vec<Pos> = all_pos()
        .into_iter()
        .filter(|p| {
            same_pos(orig_pos, *p)
                || queen_dir(
                    orig_pos.file as i32,
                    orig_pos.rank as i32,
                    p.file as i32,
                    p.rank as i32,
                )
                || knight_dir(
                    orig_pos.file as i32,
                    orig_pos.rank as i32,
                    p.file as i32,
                    p.rank as i32,
                )
        })
        .collect();
    let (best_idx, _) = valid
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let key = pos_to_key(*p).expect("valid pos");
            let center = square_center(&key, as_white, bounds);
            let dx = pos.0 - center.0;
            let dy = pos.1 - center.1;
            (i, dx * dx + dy * dy)
        })
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or((0, 0.0));
    pos_to_key(valid[best_idx])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Key;

    #[test]
    fn get_key_at_center_of_a1() {
        let bounds = BoardBounds {
            left: 0.0,
            top: 0.0,
            width: 800.0,
            height: 800.0,
        };
        // center of a1 square
        let key = get_key_at_pos((50.0, 750.0), true, bounds).unwrap();
        assert_eq!(key, Key::new("a1").unwrap());
    }
}
