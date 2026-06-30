//! Last-move, check, selection, and destination highlights.

use gpui::*;

use crate::board::selection_dests;
use crate::geometry::{BoardGeometry, Chess8x8};
use crate::state::HeadlessState;
use crate::types::Key;

use super::layout::BoardPaintLayout;

pub fn paint_highlights(
    layout: &BoardPaintLayout,
    state: &HeadlessState,
    window: &mut Window,
    _cx: &mut App,
) {
    let geometry = Chess8x8::new();

    if state.highlight.last_move
        && let Some(keys) = &state.last_move
    {
        for key in keys {
            paint_square_fill(
                layout,
                key,
                rgb(0x9bc700).alpha(0.41).into(),
                &geometry,
                window,
            );
        }
    }

    if state.highlight.check
        && let Some(key) = &state.check
    {
        paint_square_fill(layout, key, rgb(0xff0000).alpha(0.45).into(), &geometry, window);
    }

    if let Some(selected) = &state.selected {
        paint_square_fill(
            layout,
            selected,
            rgb(0x14551e).alpha(0.5).into(),
            &geometry,
            window,
        );
    }

    if let Some((orig, dest)) = &state.premovable.current {
        let premove_color = rgb(0x203085).alpha(0.45).into();
        paint_square_fill(layout, orig, premove_color, &geometry, window);
        paint_square_fill(layout, dest, premove_color, &geometry, window);
    }

    if (state.movable.show_dests || state.premovable.show_dests)
        && let Some(selected) = &state.selected
    {
        let premove = state.turn_color
            != state
                .pieces
                .get(selected)
                .map(|p| p.color)
                .unwrap_or(state.turn_color);
        let dest_color: Hsla = if premove {
            rgb(0x203085).alpha(0.5).into()
        } else {
            rgb(0x208530).alpha(0.5).into()
        };
        for dest in selection_dests(state, selected) {
            let occupied = state.pieces.contains_key(&dest);
            paint_move_dest(layout, &dest, occupied, dest_color, &geometry, window);
        }
    }
}

fn square_bounds(
    layout: &BoardPaintLayout,
    key: &Key,
    geometry: &Chess8x8,
) -> Option<Bounds<Pixels>> {
    let coord = geometry.coord_of(key)?;
    Some(layout.square_bounds(coord.file, coord.rank))
}

fn paint_square_fill(
    layout: &BoardPaintLayout,
    key: &Key,
    color: Hsla,
    geometry: &Chess8x8,
    window: &mut Window,
) {
    if let Some(sq) = square_bounds(layout, key, geometry) {
        window.paint_quad(fill(sq, color));
    }
}

fn paint_move_dest(
    layout: &BoardPaintLayout,
    key: &Key,
    occupied: bool,
    color: Hsla,
    geometry: &Chess8x8,
    window: &mut Window,
) {
    let Some(sq) = square_bounds(layout, key, geometry) else {
        return;
    };
    let center = sq.center();
    let side = sq.size.width.min(sq.size.height);

    if occupied {
        let inset = side * 0.12;
        let ring = Bounds::new(
            point(sq.origin.x + inset, sq.origin.y + inset),
            size(side - inset * 2., side - inset * 2.),
        );
        window.paint_quad(outline(ring, color.alpha(0.35), BorderStyle::Solid));
    } else {
        let dot = side * 0.22;
        window.paint_quad(fill(
            Bounds::from_corners(
                point(center.x - dot / 2., center.y - dot / 2.),
                point(center.x + dot / 2., center.y + dot / 2.),
            ),
            color,
        ));
    }
}
