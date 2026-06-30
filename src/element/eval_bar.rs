//! Evaluation gauge painted beside the board square.

use gpui::*;

use crate::eval::black_height_fraction;
use crate::state::HeadlessState;

use super::layout::BoardPaintLayout;

const EVAL_WHITE: u32 = 0xf5f5f5;
const EVAL_BLACK: u32 = 0x262421;
const EVAL_BORDER: u32 = 0x1a1a1a;
const EVAL_SEARCHING_WHITE: u32 = 0xd8d8d8;
const EVAL_SEARCHING_BLACK: u32 = 0x4a4845;

pub fn paint_eval_bar(
    layout: &BoardPaintLayout,
    state: &HeadlessState,
    window: &mut Window,
    _cx: &mut App,
) {
    let Some(bounds) = layout.eval_bar else {
        return;
    };

    let searching = state.eval.display.is_none();
    let black_fraction = black_height_fraction(state.eval.display, state.orientation);
    let black_height = bounds.size.height * black_fraction as f32;

    let white_color = if searching {
        rgb(EVAL_SEARCHING_WHITE)
    } else {
        rgb(EVAL_WHITE)
    };
    let black_color = if searching {
        rgb(EVAL_SEARCHING_BLACK)
    } else {
        rgb(EVAL_BLACK)
    };

    window.paint_quad(fill(bounds, white_color));
    window.paint_quad(fill(
        Bounds::new(
            point(bounds.origin.x, bounds.origin.y + bounds.size.height - black_height),
            size(bounds.size.width, black_height),
        ),
        black_color,
    ));

    let border = px(1.);
    window.paint_quad(fill(
        Bounds::new(bounds.origin, size(bounds.size.width, border)),
        rgb(EVAL_BORDER),
    ));
    window.paint_quad(fill(
        Bounds::new(
            point(bounds.origin.x, bounds.bottom() - border),
            size(bounds.size.width, border),
        ),
        rgb(EVAL_BORDER),
    ));
    window.paint_quad(fill(
        Bounds::new(bounds.origin, size(border, bounds.size.height)),
        rgb(EVAL_BORDER),
    ));
    window.paint_quad(fill(
        Bounds::new(
            point(bounds.right() - border, bounds.origin.y),
            size(border, bounds.size.height),
        ),
        rgb(EVAL_BORDER),
    ));
}
