//! Standard chess 8×8 geometry (chessground-compatible).

use crate::types::{Key, Pos, FILES, RANKS};
use crate::util::all_keys;

use super::{BoardGeometry, GridCoord};

/// Fixed 8×8 chess board; [`Key`] cells (`"a1"` … `"h8"`).
#[derive(Clone, Debug, Default)]
pub struct Chess8x8 {
    cells: Vec<Key>,
}

impl Chess8x8 {
    pub const FILES: u8 = 8;
    pub const RANKS: u8 = 8;

    pub fn new() -> Self {
        Self {
            cells: all_keys(),
        }
    }
}

impl BoardGeometry for Chess8x8 {
    type Cell = Key;

    fn files(&self) -> u8 {
        Self::FILES
    }

    fn ranks(&self) -> u8 {
        Self::RANKS
    }

    fn cells(&self) -> &[Key] {
        &self.cells
    }

    fn contains(&self, cell: &Key) -> bool {
        self.cells.iter().any(|c| c == cell)
    }

    fn cell_at(&self, coord: GridCoord) -> Option<&Key> {
        if coord.file >= Self::FILES || coord.rank >= Self::RANKS {
            return None;
        }
        let idx = coord.file as usize * Self::RANKS as usize + coord.rank as usize;
        self.cells.get(idx)
    }

    fn coord_of(&self, cell: &Key) -> Option<GridCoord> {
        let pos: Pos = crate::types::key_to_pos(cell)?;
        Some(GridCoord {
            file: pos.file,
            rank: pos.rank,
        })
    }

    fn label<'a>(&self, cell: &'a Key) -> &'a str {
        cell.as_str()
    }
}

/// File/rank label tables for coordinates overlay (chessground-style).
#[allow(dead_code)]
pub fn file_labels() -> &'static [char] {
    &FILES
}

#[allow(dead_code)]
pub fn rank_labels() -> &'static [char] {
    &RANKS
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Key;

    #[test]
    fn chess8x8_has_64_cells() {
        let g = Chess8x8::new();
        assert_eq!(g.files(), 8);
        assert_eq!(g.ranks(), 8);
        assert_eq!(g.cells().len(), 64);
    }

    #[test]
    fn chess8x8_coord_round_trip() {
        let g = Chess8x8::new();
        let e4 = Key::new("e4").unwrap();
        let coord = g.coord_of(&e4).unwrap();
        assert_eq!(coord.file, 4);
        assert_eq!(coord.rank, 3);
        assert_eq!(g.cell_at(coord).unwrap(), &e4);
    }
}
