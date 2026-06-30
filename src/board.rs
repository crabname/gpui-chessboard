//! Selection, moves, premove playback. Port of chessground `board.ts`.

use crate::premove;
use crate::state::{HeadlessState, Predrop};
use crate::types::{
    Color, Drop, Key, MovableColor, MoveMetadata, Piece, PiecesDiff, Role, SetPremoveMetadata,
};
use crate::util::{key_to_pos, pos_to_key_unsafe, white_pov};

/// Board operation failed (invalid square, occupied square, etc.).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoardError {
    Invalid,
}

pub fn toggle_orientation(state: &mut HeadlessState) {
    state.orientation = crate::types::opposite(state.orientation);
    state.selected = None;
}

pub fn reset(state: &mut HeadlessState) {
    state.last_move = None;
    unselect(state);
    unset_premove(state);
    unset_predrop(state);
}

pub fn set_pieces(state: &mut HeadlessState, pieces: PiecesDiff) {
    for (key, piece) in pieces {
        if let Some(p) = piece {
            state.pieces.insert(key, p);
        } else {
            state.pieces.remove(&key);
        }
    }
}

pub fn set_check(state: &mut HeadlessState, color: Option<Color>) {
    state.check = None;
    if let Some(color) = color {
        for (k, p) in &state.pieces {
            if p.role == Role::King && p.color == color {
                state.check = Some(k.clone());
            }
        }
    }
}

fn set_premove(
    state: &mut HeadlessState,
    orig: Key,
    dest: Key,
    _meta: SetPremoveMetadata,
) {
    unset_predrop(state);
    state.premovable.current = Some((orig, dest));
}

pub fn unset_premove(state: &mut HeadlessState) {
    state.premovable.current = None;
}

#[allow(dead_code)]
fn set_predrop(state: &mut HeadlessState, role: Role, key: Key) {
    unset_premove(state);
    state.predroppable.current = Some(Predrop { role, key });
}

pub fn unset_predrop(state: &mut HeadlessState) {
    state.predroppable.current = None;
}

fn try_auto_castle(state: &mut HeadlessState, orig: &Key, dest: &mut Key) -> bool {
    if !state.auto_castle {
        return false;
    }
    let Some(king) = state.pieces.get(orig).copied() else {
        return false;
    };
    if king.role != Role::King {
        return false;
    }
    let orig_pos = key_to_pos(orig);
    let dest_pos = key_to_pos(dest);
    if (orig_pos.rank != 0 && orig_pos.rank != 7) || orig_pos.rank != dest_pos.rank {
        return false;
    }
    if orig_pos.file == 4 && !state.pieces.contains_key(dest) {
        if dest_pos.file == 6 {
            *dest = pos_to_key_unsafe(crate::types::Pos {
                file: 7,
                rank: dest_pos.rank,
            });
        } else if dest_pos.file == 2 {
            *dest = pos_to_key_unsafe(crate::types::Pos {
                file: 0,
                rank: dest_pos.rank,
            });
        }
    }
    let Some(rook) = state.pieces.get(dest).copied() else {
        return false;
    };
    if rook.color != king.color || rook.role != Role::Rook {
        return false;
    }

    state.pieces.remove(orig);
    state.pieces.remove(dest);

    if orig_pos.file < dest_pos.file {
        state.pieces.insert(
            pos_to_key_unsafe(crate::types::Pos {
                file: 6,
                rank: dest_pos.rank,
            }),
            king,
        );
        state.pieces.insert(
            pos_to_key_unsafe(crate::types::Pos {
                file: 5,
                rank: dest_pos.rank,
            }),
            rook,
        );
    } else {
        state.pieces.insert(
            pos_to_key_unsafe(crate::types::Pos {
                file: 2,
                rank: dest_pos.rank,
            }),
            king,
        );
        state.pieces.insert(
            pos_to_key_unsafe(crate::types::Pos {
                file: 3,
                rank: dest_pos.rank,
            }),
            rook,
        );
    }
    true
}

pub fn base_move(
    state: &mut HeadlessState,
    orig: Key,
    dest: Key,
) -> Result<Option<Piece>, BoardError> {
    let Some(orig_piece) = state.pieces.get(&orig).copied() else {
        return Err(BoardError::Invalid);
    };
    let dest_piece = state.pieces.get(&dest).copied();
    if orig == dest {
        return Err(BoardError::Invalid);
    }
    let captured = dest_piece.filter(|p| p.color != orig_piece.color);
    if state.selected.as_ref() == Some(&dest) {
        unselect(state);
    }
    let mut dest = dest;
    if !try_auto_castle(state, &orig, &mut dest) {
        state.pieces.insert(dest.clone(), orig_piece);
        state.pieces.remove(&orig);
    }
    state.last_move = Some(vec![orig, dest]);
    state.check = None;
    Ok(captured)
}

fn base_user_move(state: &mut HeadlessState, orig: Key, dest: Key) -> Result<Option<Piece>, BoardError> {
    let result = base_move(state, orig.clone(), dest.clone())?;
    state.movable.dests = None;
    state.turn_color = crate::types::opposite(state.turn_color);
    Ok(result)
}

/// Result of a click on a square (for GPUI layer callbacks).
#[derive(Clone, Debug)]
pub enum SelectResult {
    Moved {
        orig: Key,
        dest: Key,
        metadata: MoveMetadata,
    },
    Selected(Key),
    Unselected,
    Premoved {
        orig: Key,
        dest: Key,
    },
    None,
}

pub fn user_move(state: &mut HeadlessState, orig: Key, dest: Key) -> Option<MoveMetadata> {
    if can_move(state, &orig, &dest) {
        if let Ok(captured) = base_user_move(state, orig.clone(), dest.clone()) {
            let hold_time = state.hold.stop();
            unselect(state);
            return Some(MoveMetadata {
                premove: false,
                ctrl_key: state.stats.ctrl_key,
                hold_time,
                captured,
                predrop: false,
            });
        }
    } else if can_premove(state, &orig, &dest) {
        set_premove(
            state,
            orig,
            dest,
            SetPremoveMetadata {
                ctrl_key: state.stats.ctrl_key,
            },
        );
        unselect(state);
        return None;
    }
    unselect(state);
    None
}

pub fn select_square(state: &mut HeadlessState, key: Key, force: bool) -> SelectResult {
    if let Some(selected) = state.selected.clone() {
        if selected == key && !state.draggable.enabled {
            unselect(state);
            state.hold.cancel();
            return SelectResult::Unselected;
        } else if (state.selectable.enabled || force) && selected != key {
            if let Some(metadata) = user_move(state, selected.clone(), key.clone()) {
                state.stats.dragged = false;
                return SelectResult::Moved {
                    orig: selected,
                    dest: key,
                    metadata,
                };
            } else if let Some((orig, dest)) = state.premovable.current.clone() {
                return SelectResult::Premoved { orig, dest };
            }
        }
    }
    if (state.selectable.enabled || state.draggable.enabled)
        && (is_movable(state, &key) || is_premovable(state, &key))
    {
        set_selected(state, key.clone());
        state.hold.start();
        return SelectResult::Selected(key);
    }
    SelectResult::None
}

/// Destination squares to highlight for the current or given selection.
pub fn selection_dests(state: &HeadlessState, orig: &Key) -> Vec<Key> {
    if is_premovable(state, orig) {
        state
            .premovable
            .custom_dests
            .as_ref()
            .and_then(|d| d.get(orig))
            .cloned()
            .or_else(|| state.premovable.dests.clone())
            .unwrap_or_default()
    } else if state.movable.free {
        Vec::new()
    } else {
        state
            .movable
            .dests
            .as_ref()
            .and_then(|d| d.get(orig))
            .cloned()
            .unwrap_or_default()
    }
}

pub fn set_selected(state: &mut HeadlessState, key: Key) {
    state.selected = Some(key.clone());
    if !is_premovable(state, &key) {
        state.premovable.dests = None;
    } else if state.premovable.custom_dests.is_none() {
        state.premovable.dests = Some(premove::premove(state, &key));
    }
}

pub fn unselect(state: &mut HeadlessState) {
    state.selected = None;
    state.premovable.dests = None;
    state.hold.cancel();
}

fn color_to_movable(color: Color) -> MovableColor {
    match color {
        Color::White => MovableColor::White,
        Color::Black => MovableColor::Black,
    }
}

fn is_movable(state: &HeadlessState, orig: &Key) -> bool {
    let Some(piece) = state.pieces.get(orig) else {
        return false;
    };
    match state.movable.color {
        Some(MovableColor::Both) => true,
        Some(color) => color == color_to_movable(piece.color) && state.turn_color == piece.color,
        None => false,
    }
}

pub fn can_move(state: &HeadlessState, orig: &Key, dest: &Key) -> bool {
    orig != dest
        && is_movable(state, orig)
        && (state.movable.free
            || state
                .movable
                .dests
                .as_ref()
                .and_then(|d| d.get(orig))
                .is_some_and(|dests| dests.contains(dest)))
}

fn is_premovable(state: &HeadlessState, orig: &Key) -> bool {
    let Some(piece) = state.pieces.get(orig) else {
        return false;
    };
    state.premovable.enabled
        && state.movable.color == Some(color_to_movable(piece.color))
        && state.turn_color != piece.color
}

fn can_premove(state: &HeadlessState, orig: &Key, dest: &Key) -> bool {
    if orig == dest || !is_premovable(state, orig) {
        return false;
    }
    let dests = state
        .premovable
        .custom_dests
        .as_ref()
        .and_then(|d| d.get(orig))
        .cloned()
        .unwrap_or_else(|| premove::premove(state, orig));
    dests.iter().any(|d| d == dest)
}

pub fn play_premove(state: &mut HeadlessState) -> bool {
    let Some((orig, dest)) = state.premovable.current.clone() else {
        return false;
    };
    let mut success = false;
    if can_move(state, &orig, &dest)
        && base_user_move(state, orig.clone(), dest.clone()).is_ok()
    {
        let _meta = MoveMetadata {
            premove: true,
            ..Default::default()
        };
        success = true;
    }
    unset_premove(state);
    success
}

pub fn play_predrop(state: &mut HeadlessState, validate: impl FnOnce(&Drop) -> bool) -> bool {
    let Some(drop) = state.predroppable.current.clone() else {
        return false;
    };
    let mut success = false;
    if validate(&Drop {
        role: drop.role,
        key: drop.key.clone(),
    }) {
        let piece = Piece {
            role: drop.role,
            color: state
                .movable
                .color
                .and_then(|c| match c {
                    MovableColor::White => Some(Color::White),
                    MovableColor::Black => Some(Color::Black),
                    MovableColor::Both => None,
                })
                .unwrap_or(state.turn_color),
            promoted: false,
        };
        if base_new_piece(state, piece, drop.key).is_ok() {
            success = true;
        }
    }
    unset_predrop(state);
    success
}

pub fn new_piece(state: &mut HeadlessState, piece: Piece, key: Key) -> Result<(), BoardError> {
    base_new_piece(state, piece, key)
}

fn base_new_piece(state: &mut HeadlessState, piece: Piece, key: Key) -> Result<(), BoardError> {
    if state.pieces.contains_key(&key) {
        return Err(BoardError::Invalid);
    }
    state.pieces.insert(key.clone(), piece);
    state.last_move = Some(vec![key]);
    state.check = None;
    state.movable.dests = None;
    state.turn_color = crate::types::opposite(state.turn_color);
    Ok(())
}

pub fn cancel_move(state: &mut HeadlessState) {
    state.draggable.current = None;
    unset_premove(state);
    unset_predrop(state);
    unselect(state);
}

pub fn stop(state: &mut HeadlessState) {
    state.movable.color = None;
    state.movable.dests = None;
    state.draggable.current = None;
    state.animation.current = None;
    cancel_move(state);
}

pub fn board_white_pov(state: &HeadlessState) -> bool {
    white_pov(state.orientation)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{configure, Config, MovableConfigPatch};
    use crate::state::HeadlessState;
    use crate::types::Dests;

    #[test]
    fn can_move_respects_dests() {
        let mut state = HeadlessState::defaults();
        let e2 = Key::new("e2").unwrap();
        let e4 = Key::new("e4").unwrap();
        let e5 = Key::new("e5").unwrap();
        let mut dests = Dests::new();
        dests.insert(e2.clone(), vec![e4.clone()]);
        configure(
            &mut state,
            &Config {
                movable: Some(MovableConfigPatch {
                    free: Some(false),
                    color: Some(Some(MovableColor::White)),
                    dests: Some(Some(dests)),
                    ..Default::default()
                }),
                ..Default::default()
            },
        );
        assert!(can_move(&state, &e2, &e4));
        assert!(!can_move(&state, &e2, &e5));
    }

    #[test]
    fn base_move_updates_pieces() {
        let mut state = HeadlessState::defaults();
        let e2 = Key::new("e2").unwrap();
        let e4 = Key::new("e4").unwrap();
        assert!(state.pieces.contains_key(&e2));
        assert!(!state.pieces.contains_key(&e4));
        base_move(&mut state, e2.clone(), e4.clone()).unwrap();
        assert!(!state.pieces.contains_key(&e2));
        assert!(state.pieces.contains_key(&e4));
        assert_eq!(state.last_move, Some(vec![e2, e4]));
    }

    #[test]
    fn user_move_with_free_movable() {
        let mut state = HeadlessState::defaults();
        configure(
            &mut state,
            &Config {
                movable: Some(MovableConfigPatch {
                    free: Some(true),
                    color: Some(Some(MovableColor::White)),
                    ..Default::default()
                }),
                ..Default::default()
            },
        );
        let e2 = Key::new("e2").unwrap();
        let e4 = Key::new("e4").unwrap();
        assert!(user_move(&mut state, e2, e4).is_some());
        assert_eq!(state.turn_color, Color::Black);
    }
}
