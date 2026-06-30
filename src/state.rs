//! Headless board state. Port of chessground `state.ts`.

use std::time::Instant;

use crate::draw::Drawable;
use crate::drag::DragCurrent;
use crate::element::anim::AnimCurrent;
use crate::eval::EvalBarConfig;
use crate::fen;
use crate::types::{Color, Dests, Key, MovableColor, Piece, Pieces, RanksPosition};

#[derive(Clone, Debug, Default)]
pub struct HighlightConfig {
    pub last_move: bool,
    pub check: bool,
    pub custom: Option<crate::types::SquareClasses>,
}

#[derive(Clone, Debug)]
pub struct AnimationConfig {
    pub enabled: bool,
    pub duration: u32,
    pub current: Option<AnimCurrent>,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            duration: 200,
            current: None,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct MovableEvents {
    // Callbacks are wired in the GPUI view layer (M3+).
}

#[derive(Clone, Debug)]
pub struct MovableConfig {
    pub free: bool,
    pub color: Option<MovableColor>,
    pub dests: Option<Dests>,
    pub show_dests: bool,
    pub events: MovableEvents,
    pub rook_castle: bool,
}

impl Default for MovableConfig {
    fn default() -> Self {
        Self {
            free: true,
            color: Some(MovableColor::Both),
            dests: None,
            show_dests: true,
            events: MovableEvents::default(),
            rook_castle: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PremovableConfig {
    pub enabled: bool,
    pub show_dests: bool,
    pub castle: bool,
    pub dests: Option<Vec<Key>>,
    pub custom_dests: Option<Dests>,
    pub current: Option<(Key, Key)>,
    pub events: PremovableEvents,
}

#[derive(Clone, Debug, Default)]
pub struct PremovableEvents {}

impl Default for PremovableConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            show_dests: true,
            castle: true,
            dests: None,
            custom_dests: None,
            current: None,
            events: PremovableEvents::default(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct PredroppableConfig {
    pub enabled: bool,
    pub current: Option<Predrop>,
    pub events: PredroppableEvents,
}

#[derive(Clone, Debug)]
pub struct Predrop {
    pub role: crate::types::Role,
    pub key: Key,
}

#[derive(Clone, Debug, Default)]
pub struct PredroppableEvents {}

#[derive(Clone, Debug)]
pub struct DraggableConfig {
    pub enabled: bool,
    pub distance: f32,
    pub auto_distance: bool,
    pub show_ghost: bool,
    pub delete_on_drop_off: bool,
    pub current: Option<DragCurrent>,
}

impl Default for DraggableConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            distance: 3.0,
            auto_distance: true,
            show_ghost: true,
            delete_on_drop_off: false,
            current: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SelectableConfig {
    pub enabled: bool,
}

impl Default for SelectableConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Stats {
    pub dragged: bool,
    pub ctrl_key: bool,
}

#[derive(Clone, Debug, Default)]
pub struct BoardEvents {}

#[derive(Clone, Debug, Default)]
pub struct Dropmode {
    pub active: bool,
    pub piece: Option<Piece>,
}

#[derive(Clone, Debug, Default)]
pub struct Timer {
    started: Option<Instant>,
}

impl Timer {
    pub fn start(&mut self) {
        self.started = Some(Instant::now());
    }

    pub fn cancel(&mut self) {
        self.started = None;
    }

    pub fn stop(&mut self) -> u64 {
        let Some(started) = self.started.take() else {
            return 0;
        };
        started.elapsed().as_millis() as u64
    }
}

/// Headless chessground state (no GPUI / DOM).
#[derive(Clone, Debug)]
pub struct HeadlessState {
    pub pieces: Pieces,
    pub orientation: Color,
    pub turn_color: Color,
    pub check: Option<Key>,
    pub last_move: Option<Vec<Key>>,
    pub selected: Option<Key>,
    pub coordinates: bool,
    pub coordinates_on_squares: bool,
    pub ranks_position: RanksPosition,
    pub auto_castle: bool,
    pub view_only: bool,
    pub disable_context_menu: bool,
    pub add_piece_z_index: bool,
    pub block_touch_scroll: bool,
    pub touch_ignore_radius: f32,
    pub piece_key: bool,
    pub trust_all_events: bool,
    pub js_hover: bool,
    pub highlight: HighlightConfig,
    pub animation: AnimationConfig,
    pub movable: MovableConfig,
    pub premovable: PremovableConfig,
    pub predroppable: PredroppableConfig,
    pub draggable: DraggableConfig,
    pub dropmode: Dropmode,
    pub selectable: SelectableConfig,
    pub stats: Stats,
    pub events: BoardEvents,
    pub drawable: Drawable,
    pub hold: Timer,
    pub eval: EvalBarConfig,
}

impl Default for HeadlessState {
    fn default() -> Self {
        Self::defaults()
    }
}

impl HeadlessState {
    pub fn defaults() -> Self {
        Self {
            pieces: fen::read(fen::INITIAL_FEN),
            orientation: Color::White,
            turn_color: Color::White,
            check: None,
            last_move: None,
            selected: None,
            coordinates: true,
            coordinates_on_squares: false,
            ranks_position: RanksPosition::Right,
            auto_castle: true,
            view_only: false,
            disable_context_menu: false,
            add_piece_z_index: false,
            block_touch_scroll: false,
            touch_ignore_radius: 1.0,
            piece_key: false,
            trust_all_events: false,
            js_hover: false,
            highlight: HighlightConfig {
                last_move: true,
                check: true,
                custom: None,
            },
            animation: AnimationConfig::default(),
            movable: MovableConfig::default(),
            premovable: PremovableConfig::default(),
            predroppable: PredroppableConfig::default(),
            draggable: DraggableConfig::default(),
            dropmode: Dropmode::default(),
            selectable: SelectableConfig::default(),
            stats: Stats {
                dragged: true,
                ctrl_key: false,
            },
            events: BoardEvents::default(),
            drawable: Drawable::default(),
            hold: Timer::default(),
            eval: EvalBarConfig::default(),
        }
    }
}
