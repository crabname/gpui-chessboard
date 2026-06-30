//! Embeddable chessboard UI for [GPUI](https://gpui.rs) applications.
//!
//! Licensed under GPL-3.0-or-later (see LICENSE). Derivative of
//! [lichess-org/chessground](https://github.com/lichess-org/chessground).
//!
//! This crate is a **library widget**, not a standalone chess app. It displays
//! chess positions, animates moves, and captures user input — like
//! [chessground](https://github.com/lichess-org/chessground) in the browser.
//!
//! The **host application** owns game logic, windows, menus, and persistence.
//! Pass positions and legal moves via [`Config`]; receive user moves via callbacks.
//!
//! # Quick start
//!
//! ```no_run
//! use gpui::*;
//! use gpui_chessboard::{ChessboardCallbacks, Chessground, Config};
//!
//! # fn embed(window: &mut Window, cx: &mut App) {
//! let (board, api) = Chessground::new(Config::default(), ChessboardCallbacks::default(), window, cx);
//! // Embed `board` in your layout — see **Layout** below.
//! api.set(Config::default(), cx);
//! # }
//! ```
//!
//! # Layout and embedding
//!
//! [`ChessboardView`] fills the **flex space its host allocates** and paints a
//! **centered square** using `min(available_width, available_height)`. It does
//! not pick a fixed pixel size.
//!
//! If the host chain never assigns height (or width in row layouts), the board
//! can collapse to a thin strip or zero height. That is normal flex behaviour —
//! wrap the view correctly in your GPUI layout.
//!
//! ## Internal structure
//!
//! ```text
//! ChessboardView root     flex_1, min_h_0, min_w_0
//!   └── input layer       size_full, min_h_0, min_w_0
//!         └── element     100% × 100%, flex_shrink: 1
//!               └── square side = min(parent_w, parent_h)
//! ```
//!
//! The view root already uses [`flex_1`](gpui::Styled::flex_1). It only grows
//! when its **parent is a flex container** with spare space along the main axis.
//!
//! ## Minimal wrapper (recommended)
//!
//! Matches `examples/demo.rs`:
//!
//! ```no_run
//! # use gpui::*;
//! # use gpui_chessboard::ChessboardView;
//! # fn layout(board: Entity<ChessboardView>) -> impl IntoElement {
//! div()
//!     .id("board-area")
//!     .flex_1()
//!     .min_h_0()
//!     .min_w_0()
//!     .overflow_hidden()
//!     .flex()
//!     .flex_col()
//!     .child(board.clone())
//! # }
//! ```
//!
//! ## Full-height window (toolbar + board)
//!
//! Fixed-height chrome (title bar, toolbar) should not use `flex_1`. Give the
//! board row the remaining space:
//!
//! ```no_run
//! # use gpui::*;
//! # use gpui_chessboard::ChessboardView;
//! # fn layout(board: Entity<ChessboardView>) -> impl IntoElement {
//! v_flex()
//!     .size_full()
//!     .overflow_hidden()
//!     .child(/* TitleBar, toolbar: flex_shrink_0 or natural height */)
//!     .child(
//!         div()
//!             .flex_1()
//!             .min_h_0()
//!             .min_w_0()
//!             .overflow_hidden()
//!             .flex()
//!             .flex_col()
//!             .child(board.clone()),
//!     )
//! # }
//! ```
//!
//! ## Sidebar + main content
//!
//! Both columns must share height from the same flex row:
//!
//! ```no_run
//! # use gpui::*;
//! # use gpui_chessboard::ChessboardView;
//! # fn layout(sidebar: impl IntoElement, board: Entity<ChessboardView>) -> impl IntoElement {
//! h_flex()
//!     .flex_1()
//!     .min_h_0()
//!     .min_w_0()
//!     .overflow_hidden()
//!     .items_stretch()
//!     .child(sidebar)
//!     .child(
//!         v_flex()
//!             .flex_1()
//!             .min_w_0()
//!             .min_h_0()
//!             .overflow_hidden()
//!             .child(
//!                 div()
//!                     .flex_1()
//!                     .min_h_0()
//!                     .flex()
//!                     .flex_col()
//!                     .child(board.clone()),
//!             ),
//!     )
//! # }
//! ```
//!
//! ## Tabs and nested panels
//!
//! **Common mistake:** tab content has `flex_1()` but is **not**
//! `flex().flex_col()`. The board child then has no height to expand into.
//!
//! ```no_run
//! # use gpui::*;
//! # use gpui_chessboard::ChessboardView;
//! # fn tab_content(board: Entity<ChessboardView>) -> impl IntoElement {
//! div()
//!     .id("tab-content")
//!     .flex_1()
//!     .min_h_0()
//!     .min_w_0()
//!     .overflow_hidden()
//!     .flex()
//!     .flex_col()
//!     .child(
//!         div()
//!             .size_full()
//!             .min_h_0()
//!             .flex()
//!             .flex_col()
//!             .child(board.clone()),
//!     )
//! # }
//! ```
//!
//! Every ancestor from the window root to the board wrapper should either pass
//! flex space through (`flex_1`, `min_h_0`, `overflow_hidden`) or be explicitly
//! sized (`size_full()` on the root column).
//!
//! ## Checklist when the board is missing or flat
//!
//! 1. Root column uses `.size_full()` (or equivalent window-filling bounds).
//! 2. The growing row/column uses `.flex_1().min_h_0()` (and `.min_w_0()` in
//!    horizontal layouts).
//! 3. The **direct wrapper** around `board.clone()` uses `.flex().flex_col()`.
//! 4. No broken link in the chain (e.g. tab content without flex display).
//! 5. Toolbars / tab bars use `.flex_shrink_0()` so they keep natural height.
//!
//! ## Resizing
//!
//! - The board scales down when the window shrinks; the square follows
//!   `min(width, height)`.
//! - Aspect ratio is preserved by layout geometry, not by a fixed outer
//!   `aspect_square` on the host unless that host div also receives a defined
//!   size from flex.
//!
//! Longer prose and diagrams: `docs/LAYOUT.md` in the repository.
//!
//! # Examples
//!
//! | Example | Layout |
//! |---------|--------|
//! | `cargo run --example demo` | Toolbar + board area |

pub mod api;
pub mod board;
pub mod callbacks;
pub mod config;
pub mod draw;
pub mod drag;
pub mod element;
pub mod eval;
pub mod fen;
pub mod geometry;
pub mod move_format;
pub mod premove;
pub mod state;
pub mod types;
pub mod util;
pub mod view;

pub use api::{ChessboardApi, Chessground};
pub use callbacks::ChessboardCallbacks;
pub use config::{Config, EvalConfigPatch};
pub use element::ChessboardElement;
pub use fen::{read as read_fen, write as write_fen, INITIAL_FEN};
pub use move_format::{format_move, format_user_move, parse_move};
pub use geometry::{BoardGeometry, Chess8x8, GridCoord, GridLayout, GridView, RectBoard, RectGridLayout};
pub use draw::{DrawBrush, DrawBrushPatch, DrawBrushes, DrawBrushesPatch, DrawModifiers, DrawShape, Drawable, ResolvedShapeStyle};
pub use eval::{EvalBarConfig, EvalBarPosition, EvalDisplay};
pub use state::HeadlessState;
pub use types::{Color, Dests, Fen, Key, MovableColor, Piece, Role, UserMove};
pub use view::ChessboardView;
