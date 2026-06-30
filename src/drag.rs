//! Drag-and-drop state machine. Port of chessground `drag.ts`.

use gpui::{Point, Pixels};

use crate::board::{self, SelectResult};
use crate::element::BoardPaintLayout;
use crate::state::HeadlessState;
use crate::types::{Key, Piece};
use crate::util::same_piece;

#[derive(Clone, Debug)]
pub struct DragCurrent {
    pub orig: Key,
    pub piece: Piece,
    pub orig_pos: Point<Pixels>,
    pub pos: Point<Pixels>,
    pub started: bool,
    pub previously_selected: Option<Key>,
    pub key_has_changed: bool,
}

pub fn is_draggable(state: &HeadlessState, orig: &Key) -> bool {
    let Some(piece) = state.pieces.get(orig) else {
        return false;
    };
    if !state.draggable.enabled {
        return false;
    }
    match state.movable.color {
        Some(crate::types::MovableColor::Both) => true,
        Some(color) => {
            let piece_color = match piece.color {
                crate::types::Color::White => crate::types::MovableColor::White,
                crate::types::Color::Black => crate::types::MovableColor::Black,
            };
            color == piece_color
                && (state.turn_color == piece.color || state.premovable.enabled)
        }
        None => false,
    }
}

pub fn start(
    state: &mut HeadlessState,
    orig: Key,
    pos: Point<Pixels>,
    previously_selected: Option<Key>,
) -> bool {
    let Some(piece) = state.pieces.get(&orig).copied() else {
        return false;
    };
    if state.selected.as_ref() != Some(&orig) || !is_draggable(state, &orig) {
        return false;
    }
    state.draggable.current = Some(DragCurrent {
        orig: orig.clone(),
        piece,
        orig_pos: pos,
        pos,
        started: false,
        previously_selected,
        key_has_changed: false,
    });
    true
}

pub fn update(state: &mut HeadlessState, pos: Point<Pixels>, layout: &BoardPaintLayout) {
    let Some(cur) = state.draggable.current.as_mut() else {
        return;
    };

    if let Some(anim) = &mut state.animation.current {
        anim.plan.anims.remove(&cur.orig);
    }

    let Some(orig_piece) = state.pieces.get(&cur.orig).copied() else {
        cancel(state);
        return;
    };
    if !same_piece(&orig_piece, &cur.piece) {
        cancel(state);
        return;
    }

    cur.pos = pos;
    if !cur.started {
        let dx: f32 = (cur.pos.x - cur.orig_pos.x).into();
        let dy: f32 = (cur.pos.y - cur.orig_pos.y).into();
        if dx * dx + dy * dy >= state.draggable.distance * state.draggable.distance {
            cur.started = true;
        }
    }
    if cur.started
        && let Some(hovered) = layout.key_at_window_point(pos)
    {
        cur.key_has_changed |= cur.orig != hovered;
    }
}

pub fn end(
    state: &mut HeadlessState,
    pos: Point<Pixels>,
    layout: &BoardPaintLayout,
    ctrl_key: bool,
) -> Option<SelectResult> {
    let cur = state.draggable.current.take()?;
    board::unset_premove(state);
    board::unset_predrop(state);

    let dest = layout.key_at_window_point(pos);
    let mut result = None;

    if let Some(dest_key) = dest.clone() {
        if cur.started && cur.orig != dest_key {
            state.stats.ctrl_key = ctrl_key;
            if let Some(metadata) = board::user_move(state, cur.orig.clone(), dest_key.clone()) {
                state.stats.dragged = true;
                result = Some(SelectResult::Moved {
                    orig: cur.orig.clone(),
                    dest: dest_key,
                    metadata,
                });
            } else if let Some((orig, dest)) = state.premovable.current.clone() {
                result = Some(SelectResult::Premoved { orig, dest });
            }
        }
    } else if state.draggable.delete_on_drop_off && cur.started {
        state.pieces.remove(&cur.orig);
        board::unselect(state);
    }

    if ((cur.previously_selected.as_ref() == Some(&cur.orig) || cur.key_has_changed)
        && (dest.as_ref() == Some(&cur.orig) || dest.is_none()))
        || !state.selectable.enabled
    {
        board::unselect(state);
    }

    result
}

pub fn cancel(state: &mut HeadlessState) {
    if state.draggable.current.is_some() {
        state.draggable.current = None;
        board::unselect(state);
    }
}

pub fn active_drag(state: &HeadlessState) -> Option<&DragCurrent> {
    state.draggable.current.as_ref()
}
