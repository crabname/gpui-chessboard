//! Imperative API. Port of chessground/api.ts.

use gpui::{App, Entity, Point, Pixels, Window};

use crate::board::{self, toggle_orientation};
use crate::callbacks::ChessboardCallbacks;
use crate::config::{configure, Config, EvalConfigPatch};
use crate::draw::DrawShape;
use crate::element::anim;
use crate::eval::{EvalBarPosition, EvalDisplay};
use crate::fen;
use crate::state::HeadlessState;
use crate::types::{Drop, Fen, Key, Piece, PiecesDiff};
use crate::view::ChessboardView;

/// Entry point mirroring chessground `Chessground(element, config)`.
pub struct Chessground;

impl Chessground {
    /// Create a board view and API handle.
    ///
    /// Returns `(board, api)`. Embed `board` in your GPUI tree using a flex
    /// wrapper so it gets non-zero height — see
    /// [layout and embedding](crate#layout-and-embedding).
    ///
    /// ```no_run
    /// # use gpui::*;
    /// # use gpui_chessboard::{ChessboardCallbacks, Chessground, Config};
    /// # fn example(window: &mut Window, cx: &mut App) -> impl IntoElement {
    /// # let (board, api) = Chessground::new(Config::default(), ChessboardCallbacks::default(), window, cx);
    /// div()
    ///     .flex_1()
    ///     .min_h_0()
    ///     .flex()
    ///     .flex_col()
    ///     .child(board.clone())
    /// # }
    /// ```
    #[allow(clippy::new_ret_no_self)] // mirrors chessground factory: returns (view, api)
    pub fn new(
        config: Config,
        callbacks: ChessboardCallbacks,
        window: &mut Window,
        cx: &mut App,
    ) -> (Entity<ChessboardView>, ChessboardApi) {
        let view = ChessboardView::open_with_config(config, callbacks, window, cx);
        let api = ChessboardApi { view: view.clone() };
        (view, api)
    }
}

/// Handle for programmatic board updates.
#[derive(Clone)]
pub struct ChessboardApi {
    view: Entity<ChessboardView>,
}

impl ChessboardApi {
    pub fn set(&self, config: Config, cx: &mut App) {
        self.view.update(cx, |view, cx| {
            if config.fen.is_some() && view.state.animation.enabled {
                anim::anim(|state| configure(state, &config), &mut view.state);
            } else {
                configure(&mut view.state, &config);
            }
            view.ensure_animating(cx);
            cx.notify();
        });
    }

    /// Update the evaluation bar without touching board position or animation.
    ///
    /// Pass `None` for `display` while the engine is searching; pass `Some(score)`
    /// when a new evaluation arrives.
    pub fn set_eval(&self, display: Option<EvalDisplay>, cx: &mut App) {
        self.view.update(cx, |view, cx| {
            view.state.eval.enabled = true;
            view.state.eval.display = display;
            cx.notify();
        });
    }

    /// Enable or hide the evaluation bar.
    pub fn set_eval_enabled(&self, enabled: bool, cx: &mut App) {
        self.view.update(cx, |view, cx| {
            view.state.eval.enabled = enabled;
            cx.notify();
        });
    }

    /// Place the evaluation bar to the left or right of the board square.
    pub fn set_eval_position(&self, position: EvalBarPosition, cx: &mut App) {
        self.view.update(cx, |view, cx| {
            view.state.eval.position = position;
            cx.notify();
        });
    }

    /// Full eval bar patch (enable, side, score) via [`Config`].
    pub fn configure_eval(&self, patch: EvalConfigPatch, cx: &mut App) {
        self.set(
            Config {
                eval: Some(patch),
                ..Default::default()
            },
            cx,
        );
    }

    pub fn move_(&self, orig: Key, dest: Key, cx: &mut App) {
        self.view.update(cx, |view, cx| {
            let orig = orig.clone();
            let dest = dest.clone();
            anim::anim(
                |state| {
                    let _ = board::base_move(state, orig, dest);
                },
                &mut view.state,
            );
            view.ensure_animating(cx);
            cx.notify();
        });
    }

    pub fn get_fen(&self, cx: &App) -> Fen {
        fen::write(&self.view.read(cx).state.pieces)
    }

    pub fn state(&self, cx: &App) -> HeadlessState {
        self.view.read(cx).state.clone()
    }

    pub fn toggle_orientation(&self, cx: &mut App) {
        self.view.update(cx, |view, cx| {
            toggle_orientation(&mut view.state);
            cx.notify();
        });
    }

    pub fn cancel_move(&self, cx: &mut App) {
        self.view.update(cx, |view, cx| {
            board::cancel_move(&mut view.state);
            cx.notify();
        });
    }

    pub fn stop(&self, cx: &mut App) {
        self.view.update(cx, |view, cx| {
            board::stop(&mut view.state);
            cx.notify();
        });
    }

    pub fn play_premove(&self, cx: &mut App) -> bool {
        self.view.update(cx, |view, cx| {
            let ok = board::play_premove(&mut view.state);
            if ok {
                cx.notify();
            }
            ok
        })
    }

    pub fn set_shapes(&self, shapes: Vec<DrawShape>, cx: &mut App) {
        self.view.update(cx, |view, cx| {
            view.state.drawable.shapes = shapes;
            cx.notify();
        });
    }

    pub fn set_auto_shapes(&self, shapes: Vec<DrawShape>, cx: &mut App) {
        self.view.update(cx, |view, cx| {
            view.state.drawable.auto_shapes = shapes;
            cx.notify();
        });
    }

    pub fn get_key_at_pos(&self, pos: Point<Pixels>, cx: &App) -> Option<Key> {
        self.view.read(cx).key_at_window_point(pos)
    }

    pub fn set_pieces(&self, diff: PiecesDiff, cx: &mut App) {
        self.view.update(cx, |view, cx| {
            board::set_pieces(&mut view.state, diff);
            cx.notify();
        });
    }

    pub fn new_piece(&self, piece: Piece, key: Key, cx: &mut App) -> bool {
        self.view.update(cx, |view, cx| {
            let ok = board::new_piece(&mut view.state, piece, key).is_ok();
            if ok {
                cx.notify();
            }
            ok
        })
    }

    pub fn select_square(&self, key: Option<Key>, force: bool, cx: &mut App) {
        self.view.update(cx, |view, cx| {
            if view.is_destroyed() {
                return;
            }
            if let Some(key) = key {
                anim::anim(
                    |state| {
                        let _ = board::select_square(state, key, force);
                    },
                    &mut view.state,
                );
            } else {
                board::unselect(&mut view.state);
            }
            view.ensure_animating(cx);
            cx.notify();
        });
    }

    pub fn play_predrop<F>(&self, cx: &mut App, validate: F) -> bool
    where
        F: FnOnce(&Drop) -> bool,
    {
        self.view.update(cx, |view, cx| {
            if view.is_destroyed() {
                return false;
            }
            let ok = board::play_predrop(&mut view.state, validate);
            if ok {
                cx.notify();
            }
            ok
        })
    }

    pub fn is_destroyed(&self, cx: &App) -> bool {
        self.view.read(cx).is_destroyed()
    }

    pub fn destroy(&self, cx: &mut App) {
        self.view.update(cx, |view, cx| {
            view.destroy();
            cx.notify();
        });
    }
}
