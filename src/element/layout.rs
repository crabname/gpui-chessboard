//! Square layout, orientation, coordinate margins.

use gpui::*;

use crate::eval::EvalBarPosition;
use crate::geometry::{Chess8x8, GridLayout, RectGridLayout};
use crate::geometry::GridView;
use crate::state::HeadlessState;
use crate::types::{Key, RanksPosition, FILES, RANKS};
use crate::util::BoardBounds;

pub const LIGHT_SQUARE: u32 = 0xf0d9b5;
pub const DARK_SQUARE: u32 = 0xb58863;

pub const LABEL_INSET: Pixels = px(18.);

pub const EVAL_BAR_WIDTH: Pixels = px(14.);
pub const EVAL_BAR_GAP: Pixels = px(4.);

/// Soft floor used only when painting (layout uses parent bounds).
pub const MIN_BOARD_SIDE: Pixels = px(1.);

/// Layout style for [`super::ChessboardElement`]: fill parent, shrink with flex.
pub fn board_element_style() -> Style {
    Style {
        size: Size::full(),
        max_size: Size::full(),
        flex_shrink: 1.,
        ..Default::default()
    }
}

#[derive(Clone, Debug)]
pub struct BoardPaintLayout {
    pub board: Bounds<Pixels>,
    pub square: Pixels,
    pub view: GridView,
    pub show_coords: bool,
    pub ranks_on_left: bool,
    pub label_color: Hsla,
    pub eval_bar: Option<Bounds<Pixels>>,
}

impl BoardPaintLayout {
    pub fn from_state(bounds: Bounds<Pixels>, state: &HeadlessState) -> Self {
        let show_coords = state.coordinates && !state.coordinates_on_squares;
        let ranks_on_left = state.ranks_position == RanksPosition::Left;
        let label_inset = if show_coords { LABEL_INSET } else { px(0.) };
        let eval_reserve = if state.eval.enabled {
            EVAL_BAR_WIDTH + EVAL_BAR_GAP
        } else {
            px(0.)
        };

        let mut area = bounds;
        if show_coords {
            area.size.height -= label_inset;
        }

        let mut left_margin = px(0.);
        let mut right_margin = px(0.);
        if show_coords {
            if ranks_on_left {
                left_margin += label_inset;
            } else {
                right_margin += label_inset;
            }
        }
        if state.eval.enabled {
            match state.eval.position {
                EvalBarPosition::Left => left_margin += eval_reserve,
                EvalBarPosition::Right => right_margin += eval_reserve,
            }
        }

        area.origin.x += left_margin;
        area.size.width -= left_margin + right_margin;

        let side = area.size.width.min(area.size.height).max(MIN_BOARD_SIDE);
        let board = Bounds::new(
            point(
                area.origin.x + (area.size.width - side) / 2.,
                area.origin.y + (area.size.height - side) / 2.,
            ),
            size(side, side),
        );

        let eval_bar = state.eval.enabled.then(|| {
            eval_bar_bounds(board, state.eval.position, ranks_on_left, show_coords)
        });

        Self {
            board,
            square: side / 8.,
            view: GridView::from_orientation(state.orientation),
            show_coords,
            ranks_on_left,
            label_color: rgb(0xc0c0c0).into(),
            eval_bar,
        }
    }

    pub fn square_bounds(&self, file: u8, rank: u8) -> Bounds<Pixels> {
        self.square_bounds_f32(file as f32, rank as f32)
    }

    /// Square bounds from storage file/rank, including fractional animation offsets.
    pub fn square_bounds_f32(&self, file: f32, rank: f32) -> Bounds<Pixels> {
        let mut f = file;
        let mut r = rank;
        if self.view.flip_files {
            f = 7.0 - f;
        }
        if self.view.flip_ranks {
            r = 7.0 - r;
        }
        let screen_rank = 7.0 - r;
        Bounds::new(
            point(
                self.board.origin.x + f * self.square,
                self.board.origin.y + screen_rank * self.square,
            ),
            size(self.square, self.square),
        )
    }

    pub fn is_light_square(file: u8, rank: u8) -> bool {
        (file + rank) % 2 == 1
    }

    pub fn file_label(&self, screen_file: u8) -> char {
        let mut file = screen_file;
        if self.view.flip_files {
            file = 7 - file;
        }
        FILES[file as usize]
    }

    pub fn rank_label(&self, storage_rank: u8) -> char {
        RANKS[storage_rank as usize]
    }

    /// Map screen column (0 = left) and row (0 = top) to storage file/rank.
    pub fn storage_coord_at_screen(&self, screen_file: u8, screen_rank: u8) -> (u8, u8) {
        self.view.storage_coord_at_screen(screen_file, screen_rank)
    }

    /// Map a window-space pointer position to a board [`Key`].
    pub fn key_at_window_point(&self, position: Point<Pixels>) -> Option<Key> {
        let board = self.board;
        if position.x < board.origin.x
            || position.y < board.origin.y
            || position.x >= board.origin.x + board.size.width
            || position.y >= board.origin.y + board.size.height
        {
            return None;
        }

        let bounds = BoardBounds {
            left: board.origin.x.into(),
            top: board.origin.y.into(),
            width: board.size.width.into(),
            height: board.size.height.into(),
        };
        RectGridLayout.cell_at_point(
            &Chess8x8::new(),
            (position.x.into(), position.y.into()),
            self.view,
            bounds,
        )
    }
}

fn eval_bar_bounds(
    board: Bounds<Pixels>,
    position: EvalBarPosition,
    ranks_on_left: bool,
    show_coords: bool,
) -> Bounds<Pixels> {
    let x = match position {
        EvalBarPosition::Left => {
            let mut x = board.origin.x - EVAL_BAR_GAP - EVAL_BAR_WIDTH;
            if show_coords && ranks_on_left {
                x -= LABEL_INSET;
            }
            x
        }
        EvalBarPosition::Right => {
            let mut x = board.origin.x + board.size.width + EVAL_BAR_GAP;
            if show_coords && !ranks_on_left {
                x += LABEL_INSET;
            }
            x
        }
    };
    Bounds::new(point(x, board.origin.y), size(EVAL_BAR_WIDTH, board.size.height))
}

#[cfg(test)]
mod tests {
    use super::BoardPaintLayout as Layout;
    use crate::geometry::GridView;
    use crate::types::Color;

    #[test]
    fn a1_is_dark_h1_is_light() {
        assert!(!Layout::is_light_square(0, 0));
        assert!(Layout::is_light_square(7, 0));
    }

    #[test]
    fn white_pov_bottom_left_is_a1() {
        let view = GridView::from_orientation(Color::White);
        assert_eq!(view.storage_coord_at_screen(0, 7), (0, 0));
    }

    #[test]
    fn black_pov_bottom_left_is_h1() {
        let view = GridView::from_orientation(Color::Black);
        assert_eq!(view.storage_coord_at_screen(0, 7), (7, 7));
    }

    #[test]
    fn black_pov_top_left_is_h8() {
        let view = GridView::from_orientation(Color::Black);
        assert_eq!(view.storage_coord_at_screen(0, 0), (7, 0));
    }

    #[test]
    fn rank_labels_match_square_ranks() {
        use crate::types::RANKS;

        let white = GridView::from_orientation(Color::White);
        let (_, top) = white.storage_coord_at_screen(0, 0);
        let (_, bottom) = white.storage_coord_at_screen(0, 7);
        assert_eq!(RANKS[top as usize], '8');
        assert_eq!(RANKS[bottom as usize], '1');

        let black = GridView::from_orientation(Color::Black);
        let (_, top) = black.storage_coord_at_screen(0, 0);
        let (_, bottom) = black.storage_coord_at_screen(0, 7);
        assert_eq!(RANKS[top as usize], '1');
        assert_eq!(RANKS[bottom as usize], '8');
    }

    #[test]
    fn animation_offset_uses_storage_coordinates() {
        use gpui::*;

        fn layout_for(orientation: Color) -> Layout {
            Layout {
                board: Bounds::new(point(px(0.), px(0.)), size(px(400.), px(400.))),
                square: px(50.),
                view: GridView::from_orientation(orientation),
                show_coords: false,
                ranks_on_left: true,
                label_color: rgb(0xc0c0c0).into(),
                eval_bar: None,
            }
        }

        let white = layout_for(Color::White);
        let black = layout_for(Color::Black);
        let e4 = (4_f32, 3_f32);
        let e2 = (4_f32, 1_f32);

        let white_dest = white.square_bounds_f32(e4.0, e4.1);
        let white_from = white.square_bounds_f32(e2.0, e2.1);
        assert!(white_from.origin.y > white_dest.origin.y);

        let black_dest = black.square_bounds_f32(e4.0, e4.1);
        let black_from = black.square_bounds_f32(e2.0, e2.1);
        assert!(black_from.origin.y < black_dest.origin.y);
    }
}
