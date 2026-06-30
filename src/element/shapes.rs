//! Circles, arrows, and custom overlays.

use gpui::*;

use crate::draw::{resolve_shape_style, DrawShape};
use crate::geometry::{BoardGeometry, Chess8x8};
use crate::state::HeadlessState;

use super::layout::BoardPaintLayout;

pub fn paint_shapes(
    layout: &BoardPaintLayout,
    state: &HeadlessState,
    below: bool,
    window: &mut Window,
    _cx: &mut App,
) {
    if !state.drawable.visible {
        return;
    }
    let geometry = Chess8x8::new();
    for shape in state
        .drawable
        .shapes
        .iter()
        .chain(state.drawable.auto_shapes.iter())
    {
        if shape.below == below {
            paint_shape(layout, shape, false, &geometry, state, window);
        }
    }
    if !below
        && let Some(cur) = &state.drawable.current
        && cur.mouse_sq.is_some()
    {
        let preview = DrawShape {
            orig: cur.orig.clone(),
            dest: cur.dest.clone(),
            brush: Some(cur.brush.brush_key().to_string()),
            modifiers: None,
            label: None,
            below: false,
        };
        paint_shape(layout, &preview, true, &geometry, state, window);
    }
}

fn paint_shape(
    layout: &BoardPaintLayout,
    shape: &DrawShape,
    current: bool,
    geometry: &Chess8x8,
    state: &HeadlessState,
    window: &mut Window,
) {
    let Some(style) = resolve_shape_style(shape, state) else {
        return;
    };
    let opacity = if current {
        style.opacity * 0.75
    } else {
        style.opacity
    };
    let color = hex_to_hsla(&style.color, opacity);
    let square: f32 = layout.square.into();
    let line_width = px(style.line_width * square / 64.0);

    let Some(from) = square_center(layout, &shape.orig, geometry) else {
        return;
    };
    let to = shape
        .dest
        .as_ref()
        .and_then(|dest| square_center(layout, dest, geometry))
        .unwrap_or(from);

    if shape.dest.is_none() || (from.x == to.x && from.y == to.y) {
        paint_circle(from, layout.square, line_width, color, window);
    } else {
        let hilite = style
            .hilite
            .as_deref()
            .map(|h| hex_to_hsla(h, opacity));
        paint_arrow(from, to, line_width, color, hilite, window);
    }
}

fn square_center(
    layout: &BoardPaintLayout,
    key: &crate::types::Key,
    geometry: &Chess8x8,
) -> Option<Point<Pixels>> {
    let coord = geometry.coord_of(key)?;
    Some(layout.square_bounds(coord.file, coord.rank).center())
}

fn paint_circle(
    center: Point<Pixels>,
    square: Pixels,
    line_width: Pixels,
    color: Hsla,
    window: &mut Window,
) {
    let radius: f32 = (square * 0.43).into();
    let side = radius * 2.;
    let lw: f32 = line_width.into();
    let inset = (side * 0.5 - lw * 0.5).clamp(side * 0.08, side * 0.45);
    let bounds = Bounds::new(
        point(center.x - px(radius), center.y - px(radius)),
        size(px(side), px(side)),
    );
    let ring = Bounds::new(
        point(bounds.origin.x + px(inset), bounds.origin.y + px(inset)),
        size(px(side - inset * 2.), px(side - inset * 2.)),
    );
    window.paint_quad(outline(ring, color, BorderStyle::Solid));
}

fn paint_arrow(
    from: Point<Pixels>,
    to: Point<Pixels>,
    line_width: Pixels,
    color: Hsla,
    hilite: Option<Hsla>,
    window: &mut Window,
) {
    let dx: f32 = (to.x - from.x).into();
    let dy: f32 = (to.y - from.y).into();
    let len = (dx * dx + dy * dy).sqrt();
    if len < 1.0 {
        return;
    }
    let ux = dx / len;
    let uy = dy / len;
    let margin: f32 = line_width.into();
    let margin = margin * 1.2;
    let x1 = from.x + px(ux * margin);
    let y1 = from.y + px(uy * margin);
    let x2 = to.x - px(ux * margin);
    let y2 = to.y - px(uy * margin);

    if let Some(hilite_color) = hilite {
        let lw: f32 = line_width.into();
        paint_arrow_shaft(x1, y1, x2, y2, px(lw * 1.14), hilite_color, window);
    }
    paint_arrow_shaft(x1, y1, x2, y2, line_width, color, window);

    let head: f32 = line_width.into();
    let head = head * 2.2;
    let angle = dy.atan2(dx);
    let left = angle + std::f32::consts::PI * 0.82;
    let right = angle - std::f32::consts::PI * 0.82;
    let tip_x: f32 = to.x.into();
    let tip_y: f32 = to.y.into();
    let mut head_builder = PathBuilder::fill();
    head_builder.move_to(to);
    head_builder.line_to(point(
        px(tip_x + left.cos() * head),
        px(tip_y + left.sin() * head),
    ));
    head_builder.line_to(point(
        px(tip_x + right.cos() * head),
        px(tip_y + right.sin() * head),
    ));
    head_builder.close();
    if let Ok(path) = head_builder.build() {
        window.paint_path(path, color);
    }
}

fn paint_arrow_shaft(
    x1: Pixels,
    y1: Pixels,
    x2: Pixels,
    y2: Pixels,
    line_width: Pixels,
    color: Hsla,
    window: &mut Window,
) {
    let mut builder = PathBuilder::stroke(line_width);
    builder.move_to(point(x1, y1));
    builder.line_to(point(x2, y2));
    if let Ok(path) = builder.build() {
        window.paint_path(path, color);
    }
}

fn hex_to_hsla(hex: &str, opacity: f32) -> Hsla {
    let h = hex.trim_start_matches('#');
    let (r, g, b) = if h.len() >= 6 {
        (
            u8::from_str_radix(&h[0..2], 16).unwrap_or(0),
            u8::from_str_radix(&h[2..4], 16).unwrap_or(0),
            u8::from_str_radix(&h[4..6], 16).unwrap_or(0),
        )
    } else {
        (0, 0, 0)
    };
    rgb((u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b))
        .alpha(opacity)
        .into()
}
