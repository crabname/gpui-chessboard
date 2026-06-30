//! Piece-placement FEN read/write. Port of chessground `fen.ts`.

use crate::types::{Color, Fen, Key, Piece, Pieces, Role, Pos, FILES};
use crate::util::{pos_to_key, INV_RANKS};

pub const INITIAL_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";

fn role_from_letter(c: char) -> Option<Role> {
    match c {
        'p' | 'P' => Some(Role::Pawn),
        'r' | 'R' => Some(Role::Rook),
        'n' | 'N' => Some(Role::Knight),
        'b' | 'B' => Some(Role::Bishop),
        'q' | 'Q' => Some(Role::Queen),
        'k' | 'K' => Some(Role::King),
        _ => None,
    }
}

fn letter_from_role(role: Role) -> char {
    match role {
        Role::Pawn => 'p',
        Role::Rook => 'r',
        Role::Knight => 'n',
        Role::Bishop => 'b',
        Role::Queen => 'q',
        Role::King => 'k',
    }
}

/// Read piece placement from FEN (first field only).
pub fn read(fen: &str) -> Pieces {
    let fen = if fen == "start" { INITIAL_FEN } else { fen };
    let mut pieces = Pieces::new();
    let mut row: i32 = 7;
    let mut col: i32 = 0;

    for c in fen.chars() {
        match c {
            ' ' | '[' => break,
            '/' => {
                row -= 1;
                if row < 0 {
                    break;
                }
                col = 0;
            }
            '~' => {
                let key = pos_to_key(Pos {
                    file: (col - 1) as u8,
                    rank: row as u8,
                });
                if let Some(key) = key
                    && let Some(piece) = pieces.get_mut(&key)
                {
                    piece.promoted = true;
                }
            }
            c if c.is_ascii_digit() => {
                col += c.to_digit(10).unwrap() as i32;
            }
            c => {
                if let Some(role) = role_from_letter(c) {
                    let lower = c.to_ascii_lowercase();
                    if let Some(key) = pos_to_key(Pos {
                        file: col as u8,
                        rank: row as u8,
                    }) {
                        pieces.insert(
                            key,
                            Piece {
                                role,
                                color: if c == lower {
                                    Color::Black
                                } else {
                                    Color::White
                                },
                                promoted: false,
                            },
                        );
                    }
                    col += 1;
                }
            }
        }
    }

    pieces
}

/// Write piece placement to FEN (first field only).
pub fn write(pieces: &Pieces) -> Fen {
    INV_RANKS
        .iter()
        .map(|&rank| {
            let mut row = String::new();
            let mut empty = 0usize;
            for &file in &FILES {
                let key = Key::new(&format!("{file}{rank}")).unwrap();
                if let Some(piece) = pieces.get(&key) {
                    if empty > 0 {
                        row.push_str(&empty.to_string());
                        empty = 0;
                    }
                    let mut ch = letter_from_role(piece.role);
                    if piece.color == Color::White {
                        ch = ch.to_ascii_uppercase();
                    }
                    row.push(ch);
                    if piece.promoted {
                        row.push('~');
                    }
                } else {
                    empty += 1;
                }
            }
            if empty > 0 {
                row.push_str(&empty.to_string());
            }
            row
        })
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::all_keys;

    #[test]
    fn write_read_initial() {
        assert_eq!(write(&read(INITIAL_FEN)), INITIAL_FEN);
    }

    #[test]
    fn read_start_alias() {
        assert_eq!(read("start"), read(INITIAL_FEN));
    }

    #[test]
    fn invalid_position_keys_are_valid_squares() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPPRNBQKBNR w KQkq - 0 1";
        let keys: Vec<_> = all_keys();
        for key in read(fen).keys() {
            assert!(keys.contains(key));
        }
    }

    #[test]
    fn promoted_pawn_round_trip() {
        let mut pieces = read("8/8/8/8/4p~3/8/8/8");
        let e4 = Key::new("e4").unwrap();
        let piece = pieces.get(&e4).unwrap();
        assert_eq!(piece.role, Role::Pawn);
        assert!(piece.promoted);
        let fen = write(&pieces);
        assert!(fen.contains('~'));
        pieces = read(&fen);
        assert!(pieces.get(&e4).unwrap().promoted);
    }
}
