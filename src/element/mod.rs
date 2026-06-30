//! Custom GPUI `Element` for the chessboard.

pub mod anim;
mod eval_bar;
mod highlights;
pub mod layout;
mod pieces;
mod shapes;

use std::rc::Rc;

use gpui::*;

use crate::geometry::{BoardGeometry, Chess8x8};
use crate::state::HeadlessState;

pub use layout::BoardPaintLayout;

use highlights::paint_highlights;
use eval_bar::paint_eval_bar;
use layout::{
    board_element_style, BoardPaintLayout as Layout, DARK_SQUARE, LIGHT_SQUARE,
};
use pieces::paint_piece;
use shapes::paint_shapes;

use crate::drag;

pub type LayoutSink = Rc<dyn Fn(BoardPaintLayout, Bounds<Pixels>, &mut App)>;

/// Called each paint frame while a drag is active — register window-level pointer listeners.
pub type DragPaintHook = Rc<dyn Fn(&BoardPaintLayout, &mut Window, &mut App)>;

/// Renders the board grid and pieces from [`HeadlessState`].
pub struct ChessboardElement {
    id: ElementId,
    state: HeadlessState,
    layout_sink: Option<LayoutSink>,
    drag_paint_hook: Option<DragPaintHook>,
}

impl ChessboardElement {
    pub fn new(state: HeadlessState) -> Self {
        Self {
            id: ElementId::from("gpui-chessboard"),
            state,
            layout_sink: None,
            drag_paint_hook: None,
        }
    }

    pub fn with_layout_sink(mut self, sink: LayoutSink) -> Self {
        self.layout_sink = Some(sink);
        self
    }

    pub fn with_drag_paint_hook(mut self, hook: DragPaintHook) -> Self {
        self.drag_paint_hook = Some(hook);
        self
    }
}

impl IntoElement for ChessboardElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for ChessboardElement {
    type RequestLayoutState = ();
    type PrepaintState = BoardPaintLayout;

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let layout_id = window.request_layout(board_element_style(), [], cx);
        (layout_id, ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let layout = Layout::from_state(bounds, &self.state);
        if let Some(sink) = &self.layout_sink {
            sink(layout.clone(), bounds, cx);
        }
        layout
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        paint_board(prepaint, &self.state, window, cx);
        if drag::active_drag(&self.state).is_some()
            && let Some(hook) = &self.drag_paint_hook
        {
            hook(prepaint, window, cx);
        }
    }
}

fn paint_board(
    layout: &Layout,
    state: &HeadlessState,
    window: &mut Window,
    cx: &mut App,
) {
    let geometry = Chess8x8::new();

    if layout.eval_bar.is_some() {
        paint_eval_bar(layout, state, window, cx);
    }

    for file in 0..8u8 {
        for rank in 0..8u8 {
            let sq = layout.square_bounds(file, rank);
            let color = if Layout::is_light_square(file, rank) {
                rgb(LIGHT_SQUARE)
            } else {
                rgb(DARK_SQUARE)
            };
            window.paint_quad(fill(sq, color));
        }
    }

    paint_highlights(layout, state, window, cx);
    paint_shapes(layout, state, true, window, cx);

    for (key, piece) in &state.pieces {
        if should_hide_piece(state, key) {
            continue;
        }
        if anim::is_fading(state, key) {
            continue;
        }
        if let Some(coord) = geometry.coord_of(key) {
            let sq = piece_square_bounds(layout, coord.file, coord.rank, state, key);
            paint_piece(piece, sq, window, cx);
        }
    }

    paint_ghost(layout, state, window, cx);
    paint_shapes(layout, state, false, window, cx);

    if layout.show_coords {
        paint_coordinates(layout, window, cx);
    }
}

fn paint_coordinates(layout: &Layout, window: &mut Window, cx: &mut App) {
    use layout::LABEL_INSET;

    let font_size = px(11.);
    let mut label_style = window.text_style();
    label_style.font_size = AbsoluteLength::Pixels(font_size);
    label_style.color = layout.label_color;

    for screen_file in 0..8u8 {
        let sq = layout.square_bounds(screen_file, 0);
        let label = layout.file_label(screen_file).to_string();
        let mut run = label_style.to_run(label.len());
        run.color = layout.label_color;
        let line = window
            .text_system()
            .shape_line(label.into(), font_size, &[run], None);
        let x = sq.origin.x + (sq.size.width - line.width()) / 2.;
        let y = layout.board.bottom() + px(2.);
        let _ = line.paint(point(x, y), font_size, TextAlign::Left, None, window, cx);
    }

    for rank in 0..8u8 {
        let sq = layout.square_bounds(0, rank);
        let label = layout.rank_label(rank).to_string();
        let mut run = label_style.to_run(label.len());
        run.color = layout.label_color;
        let line = window
            .text_system()
            .shape_line(label.into(), font_size, &[run], None);
        let y = sq.origin.y + (sq.size.height - font_size) / 2.;
        let x = if layout.ranks_on_left {
            layout.board.left() - LABEL_INSET + px(2.)
        } else {
            layout.board.right() + px(4.)
        };
        let _ = line.paint(point(x, y), font_size, TextAlign::Left, None, window, cx);
    }
}

fn should_hide_piece(state: &HeadlessState, key: &crate::types::Key) -> bool {
    if let Some(drag) = drag::active_drag(state)
        && drag.started && &drag.orig == key
    {
        return true;
    }
    false
}

fn piece_square_bounds(
    layout: &Layout,
    file: u8,
    rank: u8,
    state: &HeadlessState,
    key: &crate::types::Key,
) -> Bounds<Pixels> {
    let mut sq = layout.square_bounds(file, rank);
    if let Some((df, dr)) = anim::offset_for(state, key) {
        sq.origin.x += df * layout.square;
        sq.origin.y -= dr * layout.square;
    }
    sq
}

fn paint_ghost(
    layout: &Layout,
    state: &HeadlessState,
    window: &mut Window,
    cx: &mut App,
) {
    let Some(drag) = drag::active_drag(state) else {
        return;
    };
    if !drag.started || !state.draggable.show_ghost {
        return;
    }
    let side = layout.square * 0.88;
    let bounds = Bounds::new(
        point(drag.pos.x - side / 2., drag.pos.y - side / 2.),
        size(side, side),
    );
    paint_piece(&drag.piece, bounds, window, cx);
}
