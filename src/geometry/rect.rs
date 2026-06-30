//! Generic rectangular board (sketch — not wired into FEN/premove).

use super::{BoardGeometry, GridCoord};

/// N×M grid with arbitrary single-char file/rank labels.
///
/// Cell ids are `"{file}{rank}"` strings, e.g. `"a1"` on 8×8 or `"j10"` if labels allow.
/// Host supplies pieces/dests; no chess-specific logic here.
#[derive(Clone, Debug)]
pub struct RectBoard {
    file_labels: Vec<char>,
    rank_labels: Vec<char>,
    cells: Vec<String>,
}

impl RectBoard {
    pub fn new(file_labels: impl Into<Vec<char>>, rank_labels: impl Into<Vec<char>>) -> Self {
        let file_labels = file_labels.into();
        let rank_labels = rank_labels.into();
        let mut cells = Vec::with_capacity(file_labels.len() * rank_labels.len());
        for &rank in &rank_labels {
            for &file in &file_labels {
                cells.push(format!("{file}{rank}"));
            }
        }
        Self {
            file_labels,
            rank_labels,
            cells,
        }
    }

    pub fn chess8x8() -> Self {
        Self::new(
            ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'],
            ['1', '2', '3', '4', '5', '6', '7', '8'],
        )
    }
}

impl BoardGeometry for RectBoard {
    type Cell = String;

    fn files(&self) -> u8 {
        self.file_labels.len() as u8
    }

    fn ranks(&self) -> u8 {
        self.rank_labels.len() as u8
    }

    fn cells(&self) -> &[String] {
        &self.cells
    }

    fn contains(&self, cell: &String) -> bool {
        self.cells.iter().any(|c| c == cell)
    }

    fn cell_at(&self, coord: GridCoord) -> Option<&String> {
        if coord.file as usize >= self.file_labels.len()
            || coord.rank as usize >= self.rank_labels.len()
        {
            return None;
        }
        let idx = coord.file as usize * self.file_labels.len() + coord.rank as usize;
        self.cells.get(idx)
    }

    fn coord_of(&self, cell: &String) -> Option<GridCoord> {
        let rank_char = cell.chars().last()?;
        let file_char = cell.chars().next()?;
        let file = self.file_labels.iter().position(|&c| c == file_char)? as u8;
        let rank = self.rank_labels.iter().position(|&c| c == rank_char)? as u8;
        Some(GridCoord { file, rank })
    }

    fn label<'a>(&self, cell: &'a String) -> &'a str {
        cell
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_board_custom_size() {
        let board = RectBoard::new(['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j'], ['1', '2']);
        assert_eq!(board.files(), 10);
        assert_eq!(board.ranks(), 2);
        assert_eq!(board.cells().len(), 20);
        let j2 = "j2".to_string();
        assert!(board.contains(&j2));
        assert_eq!(board.coord_of(&j2).unwrap(), GridCoord { file: 9, rank: 1 });
    }
}
