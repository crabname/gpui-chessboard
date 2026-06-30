//! Board topology and screen layout (sketch for future non-8×8 / non-chess grids).
//!
//! v1 chessground port keeps using [`crate::types::Key`] and [`crate::util`] directly.
//! This module is optional glue for M2+ when paint/hit-test need a geometry-aware API.

mod chess8x8;
mod layout;
mod rect;

pub use chess8x8::Chess8x8;
pub use layout::RectGridLayout;
pub use rect::RectBoard;

/// Identifies a cell. Standard chess uses [`crate::types::Key`].
pub trait CellId: Clone + PartialEq + Eq + std::hash::Hash + std::fmt::Debug {}

impl CellId for crate::types::Key {}
impl CellId for String {}

/// File/rank address on an orthogonal grid (rank 0 = first rank in storage order).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridCoord {
    pub file: u8,
    pub rank: u8,
}

/// Which axes are flipped when mapping storage coords → screen (orientation / POV).
#[derive(Clone, Copy, Debug, Default)]
pub struct GridView {
    pub flip_files: bool,
    pub flip_ranks: bool,
}

impl GridView {
    pub const WHITE_POV: Self = Self {
        flip_files: false,
        flip_ranks: false,
    };

    pub fn from_orientation(orientation: crate::types::Color) -> Self {
        match orientation {
            crate::types::Color::White => Self::WHITE_POV,
            crate::types::Color::Black => Self {
                flip_files: true,
                flip_ranks: true,
            },
        }
    }
}

/// Board topology: cell set and grid addressing.
///
/// Rendering and input (M2+) should depend on this instead of hard-coded `8`.
pub trait BoardGeometry {
    type Cell: CellId;

    fn files(&self) -> u8;
    fn ranks(&self) -> u8;
    fn cells(&self) -> &[Self::Cell];
    fn contains(&self, cell: &Self::Cell) -> bool;
    fn cell_at(&self, coord: GridCoord) -> Option<&Self::Cell>;
    fn coord_of(&self, cell: &Self::Cell) -> Option<GridCoord>;
    fn label<'a>(&self, cell: &'a Self::Cell) -> &'a str;
}

/// Screen mapping for orthogonal grids (shared by chess 8×8 and generic rectangles).
pub trait GridLayout {
    fn cell_at_point<G: BoardGeometry>(
        &self,
        geometry: &G,
        point: (f32, f32),
        view: GridView,
        bounds: crate::util::BoardBounds,
    ) -> Option<G::Cell>;

    fn cell_center<G: BoardGeometry>(
        &self,
        geometry: &G,
        cell: &G::Cell,
        view: GridView,
        bounds: crate::util::BoardBounds,
    ) -> Option<(f32, f32)>;
}
