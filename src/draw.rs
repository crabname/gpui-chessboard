//! User-drawn circles and arrows. Port of chessground `draw.ts`.

use crate::board::{cancel_move, unselect};
use crate::element::BoardPaintLayout;
use crate::state::HeadlessState;
use crate::types::{Key, Role};
use crate::util::{get_key_at_pos, get_snapped_key_at_pos, white_pov, BoardBounds};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DrawModifiers {
    /// `#rrggbb` or `#rgb` — overrides the brush color.
    pub color: Option<String>,
    /// `0.0`–`1.0` — overrides brush opacity.
    pub opacity: Option<f32>,
    /// Stroke width in chessground units (scaled by square size / 64 when painting).
    pub line_width: Option<f32>,
    /// Optional highlight stroke for arrows (`#rrggbb`).
    pub hilite: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DrawShape {
    pub orig: Key,
    pub dest: Option<Key>,
    pub brush: Option<String>,
    pub modifiers: Option<DrawModifiers>,
    pub label: Option<String>,
    pub below: bool,
}

#[derive(Clone, Debug)]
pub struct DrawBrush {
    pub key: String,
    pub color: String,
    pub opacity: f32,
    pub line_width: f32,
}

#[derive(Clone, Debug)]
pub struct DrawBrushes {
    pub green: DrawBrush,
    pub red: DrawBrush,
    pub blue: DrawBrush,
    pub yellow: DrawBrush,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrushColor {
    Green,
    Red,
    Blue,
    Yellow,
}

impl BrushColor {
    pub fn brush_key(self) -> &'static str {
        match self {
            Self::Green => "g",
            Self::Red => "r",
            Self::Blue => "b",
            Self::Yellow => "y",
        }
    }
}

const BRUSHES: [BrushColor; 4] = [
    BrushColor::Green,
    BrushColor::Red,
    BrushColor::Blue,
    BrushColor::Yellow,
];

#[derive(Clone, Debug)]
pub struct DrawCurrent {
    pub orig: Key,
    pub dest: Option<Key>,
    pub mouse_sq: Option<Key>,
    pub pos: (f32, f32),
    pub brush: BrushColor,
    pub snap_to_valid_move: bool,
}

#[derive(Clone, Debug)]
pub struct Drawable {
    pub enabled: bool,
    pub visible: bool,
    pub default_snap_to_valid_move: bool,
    pub erase_on_movable_piece_click: bool,
    pub shapes: Vec<DrawShape>,
    pub auto_shapes: Vec<DrawShape>,
    pub current: Option<DrawCurrent>,
    pub brushes: DrawBrushes,
    pub prev_svg_hash: String,
}

/// Placeholder for draw-shape piece overlays.
#[derive(Clone, Debug)]
pub struct DrawShapePiece {
    pub role: Role,
    pub color: crate::types::Color,
    pub scale: Option<f32>,
}

impl Default for DrawBrushes {
    fn default() -> Self {
        Self {
            green: DrawBrush {
                key: "g".into(),
                color: "#15781B".into(),
                opacity: 1.0,
                line_width: 10.0,
            },
            red: DrawBrush {
                key: "r".into(),
                color: "#882020".into(),
                opacity: 1.0,
                line_width: 10.0,
            },
            blue: DrawBrush {
                key: "b".into(),
                color: "#003088".into(),
                opacity: 1.0,
                line_width: 10.0,
            },
            yellow: DrawBrush {
                key: "y".into(),
                color: "#e68f00".into(),
                opacity: 1.0,
                line_width: 10.0,
            },
        }
    }
}

impl Default for Drawable {
    fn default() -> Self {
        Self {
            enabled: true,
            visible: true,
            default_snap_to_valid_move: true,
            erase_on_movable_piece_click: true,
            shapes: Vec::new(),
            auto_shapes: Vec::new(),
            current: None,
            brushes: DrawBrushes::default(),
            prev_svg_hash: String::new(),
        }
    }
}

pub fn event_brush(shift: bool, ctrl: bool, right: bool, alt: bool, meta: bool) -> BrushColor {
    let mod_a = (shift || ctrl) && right;
    let mod_b = alt || meta;
    BRUSHES[(mod_a as usize) + (mod_b as usize) * 2]
}

fn board_bounds(layout: &BoardPaintLayout) -> BoardBounds {
    let board = layout.board;
    BoardBounds {
        left: board.origin.x.into(),
        top: board.origin.y.into(),
        width: board.size.width.into(),
        height: board.size.height.into(),
    }
}

pub fn start(
    state: &mut HeadlessState,
    position: (f32, f32),
    layout: &BoardPaintLayout,
    brush: BrushColor,
    ctrl_key: bool,
) {
    let bounds = board_bounds(layout);
    let as_white = white_pov(state.orientation);
    let Some(orig) = get_key_at_pos(position, as_white, bounds) else {
        return;
    };
    if ctrl_key {
        unselect(state);
    } else {
        cancel_move(state);
    }
    state.drawable.current = Some(DrawCurrent {
        orig,
        dest: None,
        mouse_sq: None,
        pos: position,
        brush,
        snap_to_valid_move: state.drawable.default_snap_to_valid_move,
    });
    update_current(state, layout);
}

fn update_current(state: &mut HeadlessState, layout: &BoardPaintLayout) {
    let Some(cur) = state.drawable.current.as_mut() else {
        return;
    };
    let bounds = board_bounds(layout);
    let as_white = white_pov(state.orientation);
    let pos = cur.pos;

    if get_key_at_pos(pos, as_white, bounds).is_none() {
        cur.snap_to_valid_move = false;
    }
    let mouse_sq = if cur.snap_to_valid_move {
        get_snapped_key_at_pos(&cur.orig, pos, as_white, bounds)
    } else {
        get_key_at_pos(pos, as_white, bounds)
    };
    if mouse_sq != cur.mouse_sq {
        cur.mouse_sq = mouse_sq.clone();
        cur.dest = mouse_sq.filter(|sq| sq != &cur.orig);
    }
}

pub fn move_draw(state: &mut HeadlessState, position: (f32, f32), layout: &BoardPaintLayout) {
    let Some(cur) = state.drawable.current.as_mut() else {
        return;
    };
    cur.pos = position;
    update_current(state, layout);
}

pub fn end_draw(state: &mut HeadlessState) -> bool {
    let Some(cur) = state.drawable.current.take() else {
        return false;
    };
    if cur.mouse_sq.is_some() {
        add_shape(state, &cur);
        return true;
    }
    false
}

pub fn cancel_draw(state: &mut HeadlessState) {
    state.drawable.current = None;
}

pub fn clear_shapes(state: &mut HeadlessState) -> bool {
    if state.drawable.shapes.is_empty() {
        return false;
    }
    state.drawable.shapes.clear();
    true
}

fn same_endpoints(shape: &DrawShape, cur: &DrawCurrent) -> bool {
    shape.orig == cur.orig && shape.dest == cur.dest
}

fn same_color(shape: &DrawShape, cur: &DrawCurrent) -> bool {
    shape.brush.as_deref() == Some(cur.brush.brush_key())
}

fn add_shape(state: &mut HeadlessState, cur: &DrawCurrent) {
    let similar = state
        .drawable
        .shapes
        .iter()
        .find(|s| same_endpoints(s, cur))
        .cloned();
    if let Some(similar) = similar {
        state
            .drawable
            .shapes
            .retain(|s| !same_endpoints(s, cur));
        if same_color(&similar, cur) {
            return;
        }
    }
    state.drawable.shapes.push(DrawShape {
        orig: cur.orig.clone(),
        dest: cur.dest.clone(),
        brush: Some(cur.brush.brush_key().to_string()),
        modifiers: None,
        label: None,
        below: false,
    });
}

/// Resolved paint style for a shape (brush defaults merged with per-shape modifiers).
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedShapeStyle {
    pub color: String,
    pub opacity: f32,
    pub line_width: f32,
    pub hilite: Option<String>,
}

pub fn resolve_shape_style(shape: &DrawShape, state: &HeadlessState) -> Option<ResolvedShapeStyle> {
    let base = shape
        .brush
        .as_deref()
        .and_then(|key| brush_for_shape(state, key));
    let mods = shape.modifiers.as_ref();

    let color = mods
        .and_then(|m| m.color.clone())
        .or_else(|| base.map(|b| b.color.clone()))?;

    let opacity = mods
        .and_then(|m| m.opacity)
        .or_else(|| base.map(|b| b.opacity))
        .unwrap_or(1.0);

    let line_width = mods
        .and_then(|m| m.line_width)
        .or_else(|| base.map(|b| b.line_width))
        .unwrap_or(10.0);

    Some(ResolvedShapeStyle {
        color,
        opacity,
        line_width,
        hilite: mods.and_then(|m| m.hilite.clone()),
    })
}

fn apply_brush_patch(brush: &mut DrawBrush, patch: &DrawBrushPatch) {
    if let Some(color) = &patch.color {
        brush.color = color.clone();
    }
    if let Some(opacity) = patch.opacity {
        brush.opacity = opacity;
    }
    if let Some(line_width) = patch.line_width {
        brush.line_width = line_width;
    }
}

/// Partial update for a single preset brush (used by [`DrawableConfigPatch`]).
#[derive(Clone, Debug, Default)]
pub struct DrawBrushPatch {
    pub color: Option<String>,
    pub opacity: Option<f32>,
    pub line_width: Option<f32>,
}

/// Partial update for all preset brushes.
#[derive(Clone, Debug, Default)]
pub struct DrawBrushesPatch {
    pub green: Option<DrawBrushPatch>,
    pub red: Option<DrawBrushPatch>,
    pub blue: Option<DrawBrushPatch>,
    pub yellow: Option<DrawBrushPatch>,
}

pub fn apply_brushes_patch(brushes: &mut DrawBrushes, patch: &DrawBrushesPatch) {
    if let Some(p) = &patch.green {
        apply_brush_patch(&mut brushes.green, p);
    }
    if let Some(p) = &patch.red {
        apply_brush_patch(&mut brushes.red, p);
    }
    if let Some(p) = &patch.blue {
        apply_brush_patch(&mut brushes.blue, p);
    }
    if let Some(p) = &patch.yellow {
        apply_brush_patch(&mut brushes.yellow, p);
    }
}

pub fn brush_for_shape<'a>(state: &'a HeadlessState, brush_key: &str) -> Option<&'a DrawBrush> {
    match brush_key {
        "g" | "green" => Some(&state.drawable.brushes.green),
        "r" | "red" => Some(&state.drawable.brushes.red),
        "b" | "blue" => Some(&state.drawable.brushes.blue),
        "y" | "yellow" => Some(&state.drawable.brushes.yellow),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::HeadlessState;
    use crate::types::Key;

    #[test]
    fn resolve_shape_style_merges_brush_and_modifiers() {
        let state = HeadlessState::defaults();
        let shape = DrawShape {
            orig: Key::new("e2").unwrap(),
            dest: Some(Key::new("e4").unwrap()),
            brush: Some("g".into()),
            modifiers: Some(DrawModifiers {
                opacity: Some(0.5),
                line_width: Some(14.0),
                ..Default::default()
            }),
            label: None,
            below: false,
        };
        let style = resolve_shape_style(&shape, &state).unwrap();
        assert_eq!(style.color, "#15781B");
        assert_eq!(style.opacity, 0.5);
        assert_eq!(style.line_width, 14.0);
    }

    #[test]
    fn resolve_shape_style_custom_color_without_brush() {
        let state = HeadlessState::defaults();
        let shape = DrawShape {
            orig: Key::new("d5").unwrap(),
            dest: None,
            brush: None,
            modifiers: Some(DrawModifiers {
                color: Some("#ff00ff".into()),
                opacity: Some(0.35),
                line_width: Some(6.0),
                ..Default::default()
            }),
            label: None,
            below: false,
        };
        let style = resolve_shape_style(&shape, &state).unwrap();
        assert_eq!(style.color, "#ff00ff");
        assert_eq!(style.opacity, 0.35);
        assert_eq!(style.line_width, 6.0);
    }
}
