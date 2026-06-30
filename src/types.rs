//! Board data types. Port of chessground `types.ts`.

use std::collections::HashMap;
use std::fmt;

/// Algebraic square name (`"a1"` … `"h8"`, or `"a0"` off-board).
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Key(String);

impl Key {
    pub fn new(s: &str) -> Option<Self> {
        if s == "a0" {
            return Some(Self(s.to_string()));
        }
        if s.len() != 2 {
            return None;
        }
        let mut chars = s.chars();
        let file = chars.next()?;
        let rank = chars.next()?;
        if !matches!(file, 'a'..='h') || !matches!(rank, '1'..='8') {
            return None;
        }
        Some(Self(s.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Square> for Key {
    fn from(sq: Square) -> Self {
        square_to_key(sq)
    }
}

/// Numeric square index 0..=63 (a1 = 0, h8 = 63).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Square(u8);

impl Square {
    pub const fn new(index: u8) -> Option<Self> {
        if index < 64 {
            Some(Self(index))
        } else {
            None
        }
    }

    pub fn index(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Color {
    White,
    Black,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Role {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Piece {
    pub role: Role,
    pub color: Color,
    pub promoted: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Pos {
    pub file: u8,
    pub rank: u8,
}

#[derive(Clone, Debug)]
pub struct PosAndKey {
    pub pos: Pos,
    pub key: Key,
}

/// Context for premove mobility checks (chessground `MobilityContext`).
#[derive(Clone, Debug)]
pub struct MobilityContext {
    pub orig: PosAndKey,
    pub dest: PosAndKey,
    pub role: Role,
    pub all_pieces: Pieces,
    pub friendlies: Pieces,
    pub enemies: Pieces,
    pub color: Color,
    pub rook_files_friendlies: Vec<u8>,
    pub last_move: Option<Vec<Key>>,
}

pub type MobilityFn = fn(&MobilityContext) -> bool;

pub type Pieces = HashMap<Key, Piece>;
pub type PiecesDiff = HashMap<Key, Option<Piece>>;
pub type Dests = HashMap<Key, Vec<Key>>;
pub type SquareClasses = HashMap<Key, String>;

pub type Fen = String;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RanksPosition {
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MovableColor {
    White,
    Black,
    Both,
}

#[derive(Clone, Debug, Default)]
pub struct MoveMetadata {
    pub premove: bool,
    pub ctrl_key: bool,
    pub hold_time: u64,
    pub captured: Option<Piece>,
    pub predrop: bool,
}

#[derive(Clone, Debug, Default)]
pub struct SetPremoveMetadata {
    pub ctrl_key: bool,
}

#[derive(Clone, Debug)]
pub struct UserMove {
    pub orig: Key,
    pub dest: Key,
    pub promotion: Option<Role>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Drop {
    pub role: Role,
    pub key: Key,
}

pub const FILES: [char; 8] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
pub const RANKS: [char; 8] = ['1', '2', '3', '4', '5', '6', '7', '8'];

pub fn square_file(sq: Square) -> u8 {
    sq.index() % 8
}

pub fn square_rank(sq: Square) -> u8 {
    sq.index() / 8
}

pub fn key_to_square(key: &Key) -> Option<Square> {
    let pos = key_to_pos(key)?;
    Square::new(pos.file + 8 * pos.rank)
}

pub fn square_to_key(sq: Square) -> Key {
    pos_to_key(Pos {
        file: square_file(sq),
        rank: square_rank(sq),
    })
    .expect("valid square")
}

pub fn key_to_pos(key: &Key) -> Option<Pos> {
    let s = key.as_str();
    if s == "a0" {
        return None;
    }
    let mut chars = s.chars();
    let file = chars.next()? as u8 - b'a';
    let rank = chars.next()? as u8 - b'1';
    if file > 7 || rank > 7 {
        return None;
    }
    Some(Pos { file, rank })
}

pub fn pos_to_key(pos: Pos) -> Option<Key> {
    if pos.file > 7 || pos.rank > 7 {
        return None;
    }
    Key::new(&format!("{}{}", FILES[pos.file as usize], RANKS[pos.rank as usize]))
}

pub fn opposite(color: Color) -> Color {
    match color {
        Color::White => Color::Black,
        Color::Black => Color::White,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_square_round_trip() {
        for file in 0..8u8 {
            for rank in 0..8u8 {
                let key = pos_to_key(Pos { file, rank }).unwrap();
                let sq = key_to_square(&key).unwrap();
                assert_eq!(square_file(sq), file);
                assert_eq!(square_rank(sq), rank);
                assert_eq!(Key::from(sq), key);
            }
        }
    }
}
