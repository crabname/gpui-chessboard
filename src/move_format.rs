//! Move string helpers (`e2e4`, `e7e8q`). Optional host convenience — not a UCI engine client.

use crate::types::{Key, Role, UserMove};

/// Format a move as `"e2e4"` or `"e7e8q"`.
pub fn format_move(orig: &Key, dest: &Key, promotion: Option<Role>) -> String {
    let mut s = format!("{}{}", orig.as_str(), dest.as_str());
    if let Some(role) = promotion
        && let Some(ch) = role_to_promo_char(role)
    {
        s.push(ch);
    }
    s
}

/// Format a [`UserMove`] as `"e2e4"` or `"e7e8q"`.
pub fn format_user_move(mv: &UserMove) -> String {
    format_move(&mv.orig, &mv.dest, mv.promotion)
}

/// Parse `"e2e4"`, `"e7e8q"`, or predrop `"@e4"` (orig = dest).
pub fn parse_move(s: &str) -> Option<UserMove> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    if let Some(rest) = s.strip_prefix('@') {
        if rest.len() < 2 {
            return None;
        }
        let key = Key::new(&rest[0..2])?;
        return Some(UserMove {
            orig: key.clone(),
            dest: key,
            promotion: None,
        });
    }
    if s.len() < 4 {
        return None;
    }
    let orig = Key::new(&s[0..2])?;
    let dest = Key::new(&s[2..4])?;
    let promotion = s
        .get(4..)
        .and_then(|tail| promo_char_to_role(tail.chars().next()?));
    Some(UserMove {
        orig,
        dest,
        promotion,
    })
}

fn role_to_promo_char(role: Role) -> Option<char> {
    match role {
        Role::Queen => Some('q'),
        Role::Rook => Some('r'),
        Role::Bishop => Some('b'),
        Role::Knight => Some('n'),
        Role::Pawn | Role::King => None,
    }
}

fn promo_char_to_role(ch: char) -> Option<Role> {
    match ch {
        'q' => Some(Role::Queen),
        'r' => Some(Role::Rook),
        'b' => Some(Role::Bishop),
        'n' => Some(Role::Knight),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_and_parse_basic_move() {
        let e2 = Key::new("e2").unwrap();
        let e4 = Key::new("e4").unwrap();
        assert_eq!(format_move(&e2, &e4, None), "e2e4");
        let mv = parse_move("e2e4").unwrap();
        assert_eq!(mv.orig, e2);
        assert_eq!(mv.dest, e4);
        assert_eq!(mv.promotion, None);
    }

    #[test]
    fn format_and_parse_promotion() {
        let e7 = Key::new("e7").unwrap();
        let e8 = Key::new("e8").unwrap();
        assert_eq!(format_move(&e7, &e8, Some(Role::Queen)), "e7e8q");
        let mv = parse_move("e7e8q").unwrap();
        assert_eq!(mv.promotion, Some(Role::Queen));
    }

    #[test]
    fn parse_predrop() {
        let mv = parse_move("@e4").unwrap();
        assert_eq!(mv.orig.as_str(), "e4");
        assert_eq!(mv.dest.as_str(), "e4");
    }
}
