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
        let mut f = file as f32;
        let mut r = rank as f32;
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
        let mut rank = storage_rank;
        if self.view.flip_ranks {
            rank = 7 - rank;
        }
        RANKS[rank as usize]
    }

    /// Rank digit for coordinate margin at screen row (0 = top, always 8..=1).
    pub fn screen_rank_label(screen_rank: u8) -> char {
        RANKS[(7 - screen_rank) as usize]
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
    fn rank_margin_labels_run_8_to_1_top_down() {
        assert_eq!(Layout::screen_rank_label(0), '8');
        assert_eq!(Layout::screen_rank_label(7), '1');
    }
}
