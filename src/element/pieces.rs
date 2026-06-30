//! cburnett piece sprites (lichess/chessground), rendered via GPUI [`RenderImage`].

use std::sync::{Arc, OnceLock};

use gpui::*;

use crate::types::{Color, Piece, Role};

const PIECE_COUNT: usize = 12;

static ATLAS: OnceLock<Vec<Arc<RenderImage>>> = OnceLock::new();

fn role_index(role: Role) -> usize {
    match role {
        Role::Pawn => 0,
        Role::Knight => 1,
        Role::Bishop => 2,
        Role::Rook => 3,
        Role::Queen => 4,
        Role::King => 5,
    }
}

fn piece_index(color: Color, role: Role) -> usize {
    let color_ix = match color {
        Color::White => 0,
        Color::Black => 1,
    };
    color_ix * 6 + role_index(role)
}

fn svg_bytes(color: Color, role: Role) -> &'static [u8] {
    match (color, role) {
        (Color::White, Role::Pawn) => include_bytes!("../../assets/pieces/wP.svg"),
        (Color::White, Role::Knight) => include_bytes!("../../assets/pieces/wN.svg"),
        (Color::White, Role::Bishop) => include_bytes!("../../assets/pieces/wB.svg"),
        (Color::White, Role::Rook) => include_bytes!("../../assets/pieces/wR.svg"),
        (Color::White, Role::Queen) => include_bytes!("../../assets/pieces/wQ.svg"),
        (Color::White, Role::King) => include_bytes!("../../assets/pieces/wK.svg"),
        (Color::Black, Role::Pawn) => include_bytes!("../../assets/pieces/bP.svg"),
        (Color::Black, Role::Knight) => include_bytes!("../../assets/pieces/bN.svg"),
        (Color::Black, Role::Bishop) => include_bytes!("../../assets/pieces/bB.svg"),
        (Color::Black, Role::Rook) => include_bytes!("../../assets/pieces/bR.svg"),
        (Color::Black, Role::Queen) => include_bytes!("../../assets/pieces/bQ.svg"),
        (Color::Black, Role::King) => include_bytes!("../../assets/pieces/bK.svg"),
    }
}

fn atlas(cx: &App) -> &[Arc<RenderImage>] {
    ATLAS.get_or_init(|| {
        let renderer = cx.svg_renderer();
        (0..PIECE_COUNT)
            .map(|index| {
                let color = if index < 6 {
                    Color::White
                } else {
                    Color::Black
                };
                let role = match index % 6 {
                    0 => Role::Pawn,
                    1 => Role::Knight,
                    2 => Role::Bishop,
                    3 => Role::Rook,
                    4 => Role::Queen,
                    _ => Role::King,
                };
                renderer
                    .render_single_frame(svg_bytes(color, role), 1.0)
                    .expect("cburnett piece svg")
            })
            .collect()
    })
}

pub fn paint_piece(
    piece: &Piece,
    square: Bounds<Pixels>,
    window: &mut Window,
    cx: &mut App,
) {
    let image = &atlas(cx)[piece_index(piece.color, piece.role)];
    let side = square.size.width.min(square.size.height);
    let pad = side * 0.06;
    let bounds = Bounds::new(
        point(square.origin.x + pad, square.origin.y + pad),
        size(side - pad * 2., side - pad * 2.),
    );
    let _ = window.paint_image(bounds, Corners::default(), image.clone(), 0, false);
}
