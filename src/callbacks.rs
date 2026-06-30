//! Host callbacks (not stored in headless [`HeadlessState`]).
//!
//! Passed to [`crate::Chessground::new`] and invoked after user moves.

use std::sync::Arc;

use crate::draw::DrawShape;
use crate::types::{Key, MoveMetadata, Piece};

pub type MoveFn = Arc<dyn Fn(Key, Key, Option<Piece>) + Send + Sync>;
pub type AfterMoveFn = Arc<dyn Fn(Key, Key, MoveMetadata) + Send + Sync>;
pub type ChangeFn = Arc<dyn Fn() + Send + Sync>;
pub type ShapesChangeFn = Arc<dyn Fn(Vec<DrawShape>) + Send + Sync>;

/// Callbacks wired at construction time (chessground `events` + `movable.events`).
#[derive(Clone, Default)]
pub struct ChessboardCallbacks {
    /// `events.move(orig, dest, captured)`
    pub on_move: Option<MoveFn>,
    /// `events.change()`
    pub on_change: Option<ChangeFn>,
    /// `movable.events.after(orig, dest, metadata)`
    pub after_move: Option<AfterMoveFn>,
    /// `drawable.onChange(shapes)`
    pub on_shapes_change: Option<ShapesChangeFn>,
}

impl ChessboardCallbacks {
    pub fn after_move(f: impl Fn(Key, Key, MoveMetadata) + Send + Sync + 'static) -> Self {
        Self {
            after_move: Some(Arc::new(f)),
            ..Default::default()
        }
    }
}
