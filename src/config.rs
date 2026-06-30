//! Board configuration. Port of chessground `config.ts`.

use crate::board::set_check;
use crate::board::set_selected;
use crate::draw::{apply_brushes_patch, DrawBrushesPatch, DrawShape};
use crate::eval::{EvalBarPosition, EvalDisplay};
use crate::fen;
use crate::state::HeadlessState;
use crate::types::{Color, Dests, Fen, Key, MovableColor, RanksPosition};

#[derive(Clone, Debug, Default)]
pub struct Config {
    pub fen: Option<Fen>,
    pub orientation: Option<Color>,
    pub turn_color: Option<Color>,
    pub check: Option<CheckConfig>,
    pub last_move: Option<Option<Vec<Key>>>,
    pub selected: Option<Key>,
    pub coordinates: Option<bool>,
    pub coordinates_on_squares: Option<bool>,
    pub ranks_position: Option<RanksPosition>,
    pub auto_castle: Option<bool>,
    pub view_only: Option<bool>,
    pub disable_context_menu: Option<bool>,
    pub add_piece_z_index: Option<bool>,
    pub block_touch_scroll: Option<bool>,
    pub touch_ignore_radius: Option<f32>,
    pub trust_all_events: Option<bool>,
    pub js_hover: Option<bool>,
    pub highlight: Option<HighlightConfigPatch>,
    pub animation: Option<AnimationConfigPatch>,
    pub movable: Option<MovableConfigPatch>,
    pub premovable: Option<PremovableConfigPatch>,
    pub predroppable: Option<PredroppableConfigPatch>,
    pub draggable: Option<DraggableConfigPatch>,
    pub selectable: Option<SelectableConfigPatch>,
    pub drawable: Option<DrawableConfigPatch>,
    pub eval: Option<EvalConfigPatch>,
}

/// Patch for the optional evaluation bar (scores come from the host).
#[derive(Clone, Debug, Default)]
pub struct EvalConfigPatch {
    pub enabled: Option<bool>,
    /// `Left` (default) or `Right` of the board square.
    pub position: Option<EvalBarPosition>,
    /// `Some(None)` = bar visible, score pending (engine searching).
    /// `Some(Some(v))` = show this evaluation.
    /// Field omitted on the patch = leave unchanged.
    pub display: Option<Option<EvalDisplay>>,
}

#[derive(Clone, Copy, Debug)]
pub enum CheckConfig {
    Unset,
    CurrentTurn,
    Color(Color),
}

#[derive(Clone, Debug, Default)]
pub struct HighlightConfigPatch {
    pub last_move: Option<bool>,
    pub check: Option<bool>,
}

#[derive(Clone, Debug, Default)]
pub struct AnimationConfigPatch {
    pub enabled: Option<bool>,
    pub duration: Option<u32>,
}

#[derive(Clone, Debug, Default)]
pub struct MovableConfigPatch {
    pub free: Option<bool>,
    pub color: Option<Option<MovableColor>>,
    pub dests: Option<Option<Dests>>,
    pub show_dests: Option<bool>,
    pub rook_castle: Option<bool>,
}

#[derive(Clone, Debug, Default)]
pub struct PremovableConfigPatch {
    pub enabled: Option<bool>,
    pub show_dests: Option<bool>,
    pub castle: Option<bool>,
    pub dests: Option<Option<Vec<Key>>>,
    pub custom_dests: Option<Option<Dests>>,
}

#[derive(Clone, Debug, Default)]
pub struct PredroppableConfigPatch {
    pub enabled: Option<bool>,
}

#[derive(Clone, Debug, Default)]
pub struct DraggableConfigPatch {
    pub enabled: Option<bool>,
    pub distance: Option<f32>,
    pub auto_distance: Option<bool>,
    pub show_ghost: Option<bool>,
    pub delete_on_drop_off: Option<bool>,
}

#[derive(Clone, Debug, Default)]
pub struct SelectableConfigPatch {
    pub enabled: Option<bool>,
}

#[derive(Clone, Debug, Default)]
pub struct DrawableConfigPatch {
    pub enabled: Option<bool>,
    pub visible: Option<bool>,
    pub default_snap_to_valid_move: Option<bool>,
    pub erase_on_movable_piece_click: Option<bool>,
    pub brushes: Option<DrawBrushesPatch>,
    pub shapes: Option<Vec<DrawShape>>,
    pub auto_shapes: Option<Vec<DrawShape>>,
}

pub fn apply_animation(state: &mut HeadlessState, patch: &AnimationConfigPatch) {
    if let Some(enabled) = patch.enabled {
        state.animation.enabled = enabled;
    }
    if let Some(duration) = patch.duration {
        state.animation.duration = duration;
    }
    if state.animation.duration < 70 {
        state.animation.enabled = false;
    }
}

pub fn configure(state: &mut HeadlessState, config: &Config) {
    if let Some(Some(_)) = config.movable.as_ref().map(|m| m.dests.as_ref()) {
        state.movable.dests = None;
    }
    if config.drawable.as_ref().and_then(|d| d.auto_shapes.as_ref()).is_some() {
        state.drawable.auto_shapes.clear();
    }

    if let Some(fen) = &config.fen {
        state.pieces = fen::read(fen);
        state.drawable.shapes = config
            .drawable
            .as_ref()
            .and_then(|d| d.shapes.clone())
            .unwrap_or_default();
    }

    if let Some(orientation) = config.orientation {
        state.orientation = orientation;
    }
    if let Some(turn_color) = config.turn_color {
        state.turn_color = turn_color;
    }
    if let Some(selected) = &config.selected {
        state.selected = Some(selected.clone());
    }
    if let Some(coordinates) = config.coordinates {
        state.coordinates = coordinates;
    }
    if let Some(coordinates_on_squares) = config.coordinates_on_squares {
        state.coordinates_on_squares = coordinates_on_squares;
    }
    if let Some(ranks_position) = config.ranks_position {
        state.ranks_position = ranks_position;
    }
    if let Some(auto_castle) = config.auto_castle {
        state.auto_castle = auto_castle;
    }
    if let Some(view_only) = config.view_only {
        state.view_only = view_only;
    }
    if let Some(disable_context_menu) = config.disable_context_menu {
        state.disable_context_menu = disable_context_menu;
    }
    if let Some(add_piece_z_index) = config.add_piece_z_index {
        state.add_piece_z_index = add_piece_z_index;
    }
    if let Some(block_touch_scroll) = config.block_touch_scroll {
        state.block_touch_scroll = block_touch_scroll;
    }
    if let Some(touch_ignore_radius) = config.touch_ignore_radius {
        state.touch_ignore_radius = touch_ignore_radius;
    }
    if let Some(trust_all_events) = config.trust_all_events {
        state.trust_all_events = trust_all_events;
    }
    if let Some(js_hover) = config.js_hover {
        state.js_hover = js_hover;
    }

    if let Some(highlight) = &config.highlight {
        if let Some(last_move) = highlight.last_move {
            state.highlight.last_move = last_move;
        }
        if let Some(check) = highlight.check {
            state.highlight.check = check;
        }
    }

    if let Some(movable) = &config.movable {
        if let Some(free) = movable.free {
            state.movable.free = free;
        }
        if let Some(color) = movable.color {
            state.movable.color = color;
        }
        if let Some(dests) = movable.dests.clone() {
            state.movable.dests = dests;
        }
        if let Some(show_dests) = movable.show_dests {
            state.movable.show_dests = show_dests;
        }
        if let Some(rook_castle) = movable.rook_castle {
            state.movable.rook_castle = rook_castle;
        }
    }

    if let Some(premovable) = &config.premovable {
        if let Some(enabled) = premovable.enabled {
            state.premovable.enabled = enabled;
        }
        if let Some(show_dests) = premovable.show_dests {
            state.premovable.show_dests = show_dests;
        }
        if let Some(castle) = premovable.castle {
            state.premovable.castle = castle;
        }
        if let Some(dests) = premovable.dests.clone() {
            state.premovable.dests = dests;
        }
        if let Some(custom_dests) = premovable.custom_dests.clone() {
            state.premovable.custom_dests = custom_dests;
        }
    }

    if let Some(predroppable) = &config.predroppable
        && let Some(enabled) = predroppable.enabled
    {
        state.predroppable.enabled = enabled;
    }

    if let Some(draggable) = &config.draggable {
        if let Some(enabled) = draggable.enabled {
            state.draggable.enabled = enabled;
        }
        if let Some(distance) = draggable.distance {
            state.draggable.distance = distance;
        }
        if let Some(auto_distance) = draggable.auto_distance {
            state.draggable.auto_distance = auto_distance;
        }
        if let Some(show_ghost) = draggable.show_ghost {
            state.draggable.show_ghost = show_ghost;
        }
        if let Some(delete_on_drop_off) = draggable.delete_on_drop_off {
            state.draggable.delete_on_drop_off = delete_on_drop_off;
        }
    }

    if let Some(selectable) = &config.selectable
        && let Some(enabled) = selectable.enabled
    {
        state.selectable.enabled = enabled;
    }

    if let Some(drawable) = &config.drawable {
        if let Some(enabled) = drawable.enabled {
            state.drawable.enabled = enabled;
        }
        if let Some(visible) = drawable.visible {
            state.drawable.visible = visible;
        }
        if let Some(default_snap) = drawable.default_snap_to_valid_move {
            state.drawable.default_snap_to_valid_move = default_snap;
        }
        if let Some(erase) = drawable.erase_on_movable_piece_click {
            state.drawable.erase_on_movable_piece_click = erase;
        }
        if let Some(brushes) = &drawable.brushes {
            apply_brushes_patch(&mut state.drawable.brushes, brushes);
        }
        if let Some(shapes) = &drawable.shapes {
            state.drawable.shapes = shapes.clone();
        }
        if let Some(auto_shapes) = &drawable.auto_shapes {
            state.drawable.auto_shapes = auto_shapes.clone();
        }
    }

    if let Some(check) = config.check {
        match check {
            CheckConfig::Unset => set_check(state, None),
            CheckConfig::CurrentTurn => set_check(state, Some(state.turn_color)),
            CheckConfig::Color(color) => set_check(state, Some(color)),
        }
    }

    if let Some(last_move) = &config.last_move {
        state.last_move = last_move.clone();
    }

    if let Some(selected) = &state.selected.clone() {
        set_selected(state, selected.clone());
    }

    if let Some(animation) = &config.animation {
        apply_animation(state, animation);
    }

    if let Some(eval) = &config.eval {
        if let Some(enabled) = eval.enabled {
            state.eval.enabled = enabled;
        }
        if let Some(position) = eval.position {
            state.eval.position = position;
        }
        if let Some(display) = &eval.display {
            state.eval.display = *display;
        }
    }

    filter_rook_castle_dests(state);
}

fn filter_rook_castle_dests(state: &mut HeadlessState) {
    if state.movable.rook_castle {
        return;
    }
    let Some(MovableColor::White | MovableColor::Black) = state.movable.color else {
        return;
    };
    let rank = match state.movable.color {
        Some(MovableColor::White) => '1',
        Some(MovableColor::Black) => '8',
        Some(MovableColor::Both) => return,
        None => return,
    };
    let king_start = Key::new(&format!("e{rank}")).unwrap();
    let Some(dests) = state.movable.dests.as_mut() else {
        return;
    };
    let Some(king_dests) = dests.get(&king_start).cloned() else {
        return;
    };
    let king = state.pieces.get(&king_start);
    if king.is_none_or(|p| p.role != crate::types::Role::King) {
        return;
    }
    let c_rank = format!("c{rank}");
    let g_rank = format!("g{rank}");
    let a_rank = format!("a{rank}");
    let h_rank = format!("h{rank}");
    let has_c = king_dests.iter().any(|d| d.as_str() == c_rank);
    let has_g = king_dests.iter().any(|d| d.as_str() == g_rank);
    let filtered: Vec<Key> = king_dests
        .into_iter()
        .filter(|d| {
            !(d.as_str() == a_rank && has_c || d.as_str() == h_rank && has_g)
        })
        .collect();
    dests.insert(king_start, filtered);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::{EvalBarPosition, EvalDisplay};
    use crate::fen::INITIAL_FEN;
    use crate::types::{Key, Role};

    #[test]
    fn configure_replaces_pieces_from_fen() {
        let mut state = HeadlessState::defaults();
        configure(
            &mut state,
            &Config {
                fen: Some("8/8/8/8/8/8/8/k7".into()),
                ..Default::default()
            },
        );
        assert_eq!(state.pieces.len(), 1);
        let king = state.pieces.get(&Key::new("a1").unwrap()).unwrap();
        assert_eq!(king.role, Role::King);
    }

    #[test]
    fn configure_clears_dests_before_merge_when_new_dests_sent() {
        let mut state = HeadlessState::defaults();
        let mut dests = Dests::new();
        dests.insert(Key::new("e2").unwrap(), vec![Key::new("e4").unwrap()]);
        configure(
            &mut state,
            &Config {
                movable: Some(MovableConfigPatch {
                    dests: Some(Some(dests)),
                    ..Default::default()
                }),
                ..Default::default()
            },
        );
        assert!(state.movable.dests.is_some());
        configure(
            &mut state,
            &Config {
                movable: Some(MovableConfigPatch {
                    dests: Some(Some(Dests::new())),
                    ..Default::default()
                }),
                ..Default::default()
            },
        );
        assert!(state.movable.dests.as_ref().unwrap().is_empty());
    }

    #[test]
    fn configure_eval_position() {
        let mut state = HeadlessState::defaults();
        assert_eq!(state.eval.position, EvalBarPosition::Left);

        configure(
            &mut state,
            &Config {
                eval: Some(EvalConfigPatch {
                    position: Some(EvalBarPosition::Right),
                    ..Default::default()
                }),
                ..Default::default()
            },
        );
        assert_eq!(state.eval.position, EvalBarPosition::Right);
    }

    #[test]
    fn configure_eval_display() {
        let mut state = HeadlessState::defaults();
        configure(
            &mut state,
            &Config {
                eval: Some(EvalConfigPatch {
                    enabled: Some(true),
                    display: Some(Some(EvalDisplay::cp(120))),
                    ..Default::default()
                }),
                ..Default::default()
            },
        );
        assert!(state.eval.enabled);
        assert_eq!(state.eval.display, Some(EvalDisplay::cp(120)));

        configure(
            &mut state,
            &Config {
                eval: Some(EvalConfigPatch {
                    display: Some(None),
                    ..Default::default()
                }),
                ..Default::default()
            },
        );
        assert!(state.eval.enabled);
        assert_eq!(state.eval.display, None);
    }

    #[test]
    fn configure_starting_position() {
        let mut state = HeadlessState::defaults();
        configure(
            &mut state,
            &Config {
                fen: Some(INITIAL_FEN.into()),
                ..Default::default()
            },
        );
        assert_eq!(state.pieces.len(), 32);
    }
}
