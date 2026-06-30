//! Orthogonal grid ↔ pixel mapping.

use super::{BoardGeometry, GridCoord, GridLayout, GridView};

/// Uniform rectangular grid layout (letterboxed square uses caller-provided bounds).
#[derive(Clone, Copy, Debug, Default)]
pub struct RectGridLayout;

impl GridLayout for RectGridLayout {
    fn cell_at_point<G: BoardGeometry>(
        &self,
        geometry: &G,
        point: (f32, f32),
        view: GridView,
        bounds: crate::util::BoardBounds,
    ) -> Option<G::Cell> {
        let files = geometry.files() as i32;
        let ranks = geometry.ranks() as i32;
        if files == 0 || ranks == 0 {
            return None;
        }

        let mut file =
            ((files as f32 * (point.0 - bounds.left)) / bounds.width).floor() as i32;
        let mut rank =
            ranks - 1 - ((ranks as f32 * (point.1 - bounds.top)) / bounds.height).floor() as i32;

        if view.flip_files {
            file = files - 1 - file;
        }
        if view.flip_ranks {
            rank = ranks - 1 - rank;
        }

        if !(0..files).contains(&file) || !(0..ranks).contains(&rank) {
            return None;
        }

        geometry
            .cell_at(GridCoord {
                file: file as u8,
                rank: rank as u8,
            })
            .cloned()
    }

    fn cell_center<G: BoardGeometry>(
        &self,
        geometry: &G,
        cell: &G::Cell,
        view: GridView,
        bounds: crate::util::BoardBounds,
    ) -> Option<(f32, f32)> {
        let coord = geometry.coord_of(cell)?;
        let files = geometry.files() as f32;
        let ranks = geometry.ranks() as f32;
        let sq_w = bounds.width / files;
        let sq_h = bounds.height / ranks;

        let mut file = coord.file as f32;
        let mut rank = coord.rank as f32;
        if view.flip_files {
            file = files - 1.0 - file;
        }
        if view.flip_ranks {
            rank = ranks - 1.0 - rank;
        }

        let screen_rank = ranks - 1.0 - rank;
        Some((
            bounds.left + (file + 0.5) * sq_w,
            bounds.top + (screen_rank + 0.5) * sq_h,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Chess8x8;
    use crate::types::Key;
    use crate::util::BoardBounds;

    #[test]
    fn rect_layout_matches_util_hit_test_for_chess8x8() {
        let geometry = Chess8x8::new();
        let layout = RectGridLayout;
        let bounds = BoardBounds {
            left: 0.0,
            top: 0.0,
            width: 800.0,
            height: 800.0,
        };
        let point = (50.0, 750.0);
        let via_layout = layout
            .cell_at_point(&geometry, point, GridView::WHITE_POV, bounds)
            .unwrap();
        let via_util = crate::util::get_key_at_pos(point, true, bounds).unwrap();
        assert_eq!(via_layout, via_util);
        assert_eq!(via_layout, Key::new("a1").unwrap());
    }

    #[test]
    fn cell_at_point_distinguishes_e2_and_e3_centers() {
        let geometry = Chess8x8::new();
        let layout = RectGridLayout;
        let bounds = BoardBounds {
            left: 0.0,
            top: 0.0,
            width: 800.0,
            height: 800.0,
        };
        let sq = bounds.width / 8.0;
        // file e = 4, rank 2 = storage rank 1, rank 3 = storage rank 2
        let e2_center = (bounds.left + (4.0 + 0.5) * sq, bounds.top + (6.0 + 0.5) * sq);
        let e3_center = (bounds.left + (4.0 + 0.5) * sq, bounds.top + (5.0 + 0.5) * sq);
        assert_eq!(
            layout
                .cell_at_point(&geometry, e2_center, GridView::WHITE_POV, bounds)
                .unwrap(),
            Key::new("e2").unwrap()
        );
        assert_eq!(
            layout
                .cell_at_point(&geometry, e3_center, GridView::WHITE_POV, bounds)
                .unwrap(),
            Key::new("e3").unwrap()
        );
    }
}
