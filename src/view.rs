//! GPUI view: [`ChessboardView`] + pointer input wiring.
//!
//! See the crate-level **Layout and embedding** section for flex wrappers so
//! the board receives height and does not collapse.

use std::rc::Rc;
use std::time::Duration;

use gpui::*;

use crate::board::{self, SelectResult};
use crate::callbacks::ChessboardCallbacks;
use crate::config::{configure, Config};
use crate::drag;
use crate::draw;
use crate::element::anim;
use crate::element::{BoardPaintLayout, ChessboardElement, DragPaintHook};
use crate::state::HeadlessState;
use crate::types::MoveMetadata;

/// Root chessboard view embedded in host layouts.
///
/// Render this entity as `.child(board.clone())` inside a flex wrapper — see
/// [crate-level layout docs](crate#layout-and-embedding).
///
/// The view root uses `flex_1`, `min_h_0`, and `min_w_0`. Its parent must be a
/// flex container with available space (typically
/// `.flex_1().min_h_0().flex().flex_col()` on the wrapper `div`).
pub struct ChessboardView {
    pub(crate) state: HeadlessState,
    pub(crate) layout: Option<BoardPaintLayout>,
    pub(crate) element_bounds: Option<Bounds<Pixels>>,
    callbacks: ChessboardCallbacks,
    animation_loop: bool,
    needs_layout: bool,
    destroyed: bool,
}

impl ChessboardView {
    pub fn new(state: HeadlessState, callbacks: ChessboardCallbacks) -> Self {
        Self {
            state,
            layout: None,
            element_bounds: None,
            callbacks,
            animation_loop: false,
            needs_layout: true,
            destroyed: false,
        }
    }

    pub fn is_destroyed(&self) -> bool {
        self.destroyed
    }

    pub(crate) fn destroy(&mut self) {
        self.destroyed = true;
        board::stop(&mut self.state);
        drag::cancel(&mut self.state);
        draw::cancel_draw(&mut self.state);
    }

    pub fn open(window: &mut Window, cx: &mut App) -> Entity<Self> {
        Self::open_with_config(Config::default(), ChessboardCallbacks::default(), window, cx)
    }

    pub fn open_with_config(
        config: Config,
        callbacks: ChessboardCallbacks,
        window: &mut Window,
        cx: &mut App,
    ) -> Entity<Self> {
        let _ = window;
        cx.new(|_| {
            let mut state = HeadlessState::defaults();
            configure(&mut state, &config);
            Self::new(state, callbacks)
        })
    }

    pub fn state(&self) -> &HeadlessState {
        &self.state
    }

    pub fn callbacks(&self) -> &ChessboardCallbacks {
        &self.callbacks
    }

    pub fn key_at_window_point(&self, pos: Point<Pixels>) -> Option<crate::types::Key> {
        if let Some(layout) = &self.layout {
            return layout.key_at_window_point(pos);
        }
        self.element_bounds
            .and_then(|bounds| {
                BoardPaintLayout::from_state(bounds, &self.state).key_at_window_point(pos)
            })
    }

    fn pointer_xy(position: Point<Pixels>) -> (f32, f32) {
        (position.x.into(), position.y.into())
    }

    pub(crate) fn ensure_animating(&mut self, cx: &mut Context<Self>) {
        if self.state.animation.current.is_none() || self.animation_loop {
            return;
        }
        self.animation_loop = true;
        cx.spawn(async move |this, cx| {
            loop {
                let still = this
                    .update(cx, |view, cx| {
                        let still = anim::step(&mut view.state);
                        if still {
                            cx.notify();
                        } else {
                            view.animation_loop = false;
                        }
                        still
                    })
                    .ok()
                    .unwrap_or(false);
                if !still {
                    break;
                }
                cx.background_executor()
                    .timer(Duration::from_millis(16))
                    .await;
            }
        })
        .detach();
    }

    fn paint_layout(&self) -> Option<BoardPaintLayout> {
        if let Some(layout) = &self.layout {
            return Some(layout.clone());
        }
        self.element_bounds
            .map(|bounds| BoardPaintLayout::from_state(bounds, &self.state))
    }

    fn on_left_mouse_down(&mut self, event: &MouseDownEvent, cx: &mut Context<Self>) {
        if self.destroyed || self.state.view_only {
            return;
        }
        if self.state.drawable.current.is_some() {
            draw::cancel_draw(&mut self.state);
        }

        let Some(layout) = self.paint_layout() else {
            self.needs_layout = true;
            cx.notify();
            return;
        };
        let Some(key) = layout.key_at_window_point(event.position) else {
            return;
        };

        self.state.stats.ctrl_key = event.modifiers.control;
        let previously_selected = self.state.selected.clone();

        if previously_selected.is_none() && self.state.drawable.enabled {
            let piece = self.state.pieces.get(&key);
            if (self.state.drawable.erase_on_movable_piece_click
                || piece.is_none()
                || piece.is_some_and(|p| p.color != self.state.turn_color))
                && draw::clear_shapes(&mut self.state)
            {
                self.fire_shapes_change(cx);
            }
        }

        let result = if self
            .state
            .selected
            .as_ref()
            .is_some_and(|selected| board::can_move(&self.state, selected, &key))
        {
            anim::anim(
                |state| board::select_square(state, key.clone(), false),
                &mut self.state,
            )
        } else {
            board::select_square(&mut self.state, key.clone(), false)
        };

        if self.state.selected.as_ref() == Some(&key) {
            drag::start(&mut self.state, key, event.position, previously_selected);
        }

        self.dispatch_result(result, cx);
        self.ensure_animating(cx);
        cx.notify();
    }

    fn on_right_mouse_down(&mut self, event: &MouseDownEvent, cx: &mut Context<Self>) {
        if self.destroyed || self.state.view_only || !self.state.drawable.enabled {
            return;
        }
        let Some(layout) = self.paint_layout() else {
            self.needs_layout = true;
            cx.notify();
            return;
        };
        let brush = draw::event_brush(
            event.modifiers.shift,
            event.modifiers.control,
            true,
            event.modifiers.alt,
            event.modifiers.platform,
        );
        draw::start(
            &mut self.state,
            Self::pointer_xy(event.position),
            &layout,
            brush,
            event.modifiers.control,
        );
        cx.notify();
    }

    fn on_right_mouse_up(&mut self, _event: &MouseUpEvent, cx: &mut Context<Self>) {
        if draw::end_draw(&mut self.state) {
            self.fire_shapes_change(cx);
            self.fire_change_callback(cx);
        } else {
            draw::cancel_draw(&mut self.state);
        }
        cx.notify();
    }

    fn finish_pointer(
        &mut self,
        position: Point<Pixels>,
        layout: &BoardPaintLayout,
        ctrl_key: bool,
        cx: &mut Context<Self>,
    ) {
        if let Some(result) = drag::end(&mut self.state, position, layout, ctrl_key) {
            self.dispatch_result(result, cx);
        }
        self.ensure_animating(cx);
        cx.notify();
    }

    fn on_mouse_up(&mut self, event: &MouseUpEvent, cx: &mut Context<Self>) {
        let Some(layout) = self.paint_layout() else {
            drag::cancel(&mut self.state);
            return;
        };
        self.finish_pointer(event.position, &layout, event.modifiers.control, cx);
    }

    fn dispatch_result(&mut self, result: SelectResult, cx: &mut Context<Self>) {
        match result {
            SelectResult::Moved {
                orig,
                dest,
                metadata,
            } => {
                self.fire_move_callbacks(orig, dest, metadata, cx);
            }
            SelectResult::Selected(_) | SelectResult::Unselected | SelectResult::Premoved { .. } => {
                self.fire_change_callback(cx);
            }
            SelectResult::None => {}
        }
    }

    fn fire_move_callbacks(
        &self,
        orig: crate::types::Key,
        dest: crate::types::Key,
        metadata: MoveMetadata,
        cx: &mut Context<Self>,
    ) {
        let captured = metadata.captured;
        let callbacks = self.callbacks.clone();
        cx.spawn(async move |_this, _cx| {
            if let Some(f) = callbacks.on_move {
                f(orig.clone(), dest.clone(), captured);
            }
            if let Some(f) = callbacks.after_move {
                f(orig, dest, metadata);
            }
            if let Some(f) = callbacks.on_change {
                f();
            }
        })
        .detach();
    }

    fn fire_change_callback(&self, cx: &mut Context<Self>) {
        let Some(f) = self.callbacks.on_change.clone() else {
            return;
        };
        cx.spawn(async move |_this, _cx| f()).detach();
    }

    fn fire_shapes_change(&self, cx: &mut Context<Self>) {
        let Some(f) = self.callbacks.on_shapes_change.clone() else {
            return;
        };
        let shapes = self.state.drawable.shapes.clone();
        cx.spawn(async move |_this, _cx| f(shapes)).detach();
    }
}

impl Render for ChessboardView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.destroyed {
            return div().id("chessboard-destroyed");
        }

        if self.needs_layout && self.layout.is_none() {
            let entity = cx.entity();
            cx.defer(move |cx| {
                entity.update(cx, |_, cx| cx.notify());
            });
        }

        let view = cx.entity();
        let layout_sink: crate::element::LayoutSink =
            Rc::new(move |layout, bounds, cx| {
                view.update(cx, |view, _| {
                    view.element_bounds = Some(bounds);
                    view.layout = Some(layout);
                    view.needs_layout = false;
                });
            });

        let drag_view = cx.entity();
        let drag_paint_hook: DragPaintHook = Rc::new(move |layout, window, _cx| {
            let layout = layout.clone();
            let move_view = drag_view.clone();
            let move_layout = layout.clone();
            window.on_mouse_event(move |event: &MouseMoveEvent, phase, _, cx| {
                if phase == DispatchPhase::Bubble {
                    let view = move_view.clone();
                    let layout = move_layout.clone();
                    view.update(cx, |view, cx| {
                        if view.state.draggable.current.is_some() {
                            drag::update(&mut view.state, event.position, &layout);
                            cx.notify();
                        }
                    });
                }
            });
            let up_view = drag_view.clone();
            let up_layout = layout;
            window.on_mouse_event(move |event: &MouseUpEvent, phase, _, cx| {
                if phase == DispatchPhase::Bubble && event.button == MouseButton::Left {
                    up_view.update(cx, |view, cx| {
                        view.finish_pointer(event.position, &up_layout, event.modifiers.control, cx);
                    });
                }
            });
        });

        div()
            .id("chessboard-root")
            .flex_1()
            .min_h_0()
            .min_w_0()
            .overflow_hidden()
            .bg(rgb(0x302e2b))
            .p_2()
            .child(
                div()
                    .id("chessboard-input")
                    .size_full()
                    .min_h_0()
                    .min_w_0()
                    .overflow_hidden()
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, event, _, cx| {
                        this.on_left_mouse_down(event, cx);
                    }))
                    .on_mouse_down(MouseButton::Right, cx.listener(|this, event, _, cx| {
                        this.on_right_mouse_down(event, cx);
                    }))
                    .on_mouse_up(MouseButton::Left, cx.listener(|this, event, _, cx| {
                        this.on_mouse_up(event, cx);
                    }))
                    .on_mouse_up(MouseButton::Right, cx.listener(|this, event, _, cx| {
                        this.on_right_mouse_up(event, cx);
                    }))
                    .on_mouse_up_out(MouseButton::Left, cx.listener(|this, event, _, cx| {
                        this.on_mouse_up(event, cx);
                    }))
                    .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                        if let Some(layout) = this.paint_layout() {
                            if this.state.drawable.current.is_some() {
                                draw::move_draw(
                                    &mut this.state,
                                    ChessboardView::pointer_xy(event.position),
                                    &layout,
                                );
                                cx.notify();
                            } else if this.state.draggable.current.is_some() {
                                drag::update(&mut this.state, event.position, &layout);
                                cx.notify();
                            }
                        }
                    }))
                    .child(
                        ChessboardElement::new(self.state.clone())
                            .with_layout_sink(layout_sink)
                            .with_drag_paint_hook(drag_paint_hook),
                    ),
            )
    }
}
