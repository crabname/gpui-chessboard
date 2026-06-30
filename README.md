# gpui-chessboard

Embeddable chessboard **UI library** for [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui) desktop applications. It displays positions, animates moves, and captures user input — the same role as [lichess-org/chessground](https://github.com/lichess-org/chessground) in the browser.

This crate is a **widget**, not a chess application. The host app owns game rules, engines, databases, windows, and persistence. You pass positions and legal destinations via [`Config`](src/config.rs); user moves come back through callbacks.

## Features

- Board and piece rendering from FEN (piece placement)
- Click-to-move and drag-and-drop
- Move highlights, check highlight, destination dots
- Animated moves and programmatic `api.move_`
- Orientation flip, last-move highlight
- Drawable arrows and circles (manual and auto shapes)
- Premove highlights and editor-style free moves
- Optional evaluation bar (host-provided scores)
- Move string helpers (`e2e4`, `e7e8q`) — not a UCI client

## Out of scope

- Chess rules and legality checking
- Game state, clocks, PGN import/export
- Engine protocols (UCI, etc.)
- Standalone window or application shell

The host supplies legal moves through `movable.dests` (or `movable.free` in editor mode).

## Requirements

- Rust 2024 edition
- GPUI (pinned git dependency — see `Cargo.toml`)
- macOS for the included demo (`gpui_platform`); other platforms depend on GPUI support in your host app

## Adding to your project

```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed", rev = "1d217ee39d381ac101b7cf49d3d22451ac1093fe" }
gpui-chessboard = { path = "../gpui-chessboard" }
```

Pin the same GPUI revision as this crate unless you know the APIs you rely on are unchanged.

## Quick start

```rust
use gpui::*;
use gpui_chessboard::{ChessboardCallbacks, Chessground, Config};

let (board, api) = Chessground::new(
    Config::default(),
    ChessboardCallbacks::default(),
    window,
    cx,
);

// In your Render tree — the wrapper must be a flex column with flex_1 + min_h_0:
div()
    .flex_1()
    .min_h_0()
    .min_w_0()
    .overflow_hidden()
    .flex()
    .flex_col()
    .child(board.clone());

// Update position, highlights, legal moves:
api.set(Config {
    fen: Some("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3".into()),
    ..Default::default()
}, cx);
```

Handle user moves in `ChessboardCallbacks` (or read `api.state(cx)` after interaction), validate in your app, then call `api.set` with the new FEN and `dests`.

## Layout

[`ChessboardView`](src/view.rs) fills the flex space its parent allocates and paints a **centered square** using `min(width, height)`. If wrappers omit `flex_1`, `min_h_0`, and `flex().flex_col()`, the board can collapse to zero height.

See **[docs/LAYOUT.md](docs/LAYOUT.md)** for copy-paste patterns (toolbar + board, sidebar layouts, tabs).

## API overview

| Type | Role |
|------|------|
| `Chessground::new` | Factory → `(Entity<ChessboardView>, ChessboardApi)` |
| `ChessboardApi` | Imperative updates: `set`, `move_`, `toggle_orientation`, `set_shapes`, `set_eval`, … |
| `Config` | Partial patch merged into board state (FEN, dests, highlights, drawable, eval bar) |
| `ChessboardCallbacks` | `on_move`, `on_change`, drawable hooks |
| `UserMove` | `{ orig, dest, promotion }` from user input |
| `format_move` / `parse_move` | Optional LAN-style strings for host exchange |

Crate docs (`cargo doc --open`) and [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) describe modules and data formats in more detail.

## Demo

A minimal host shell with a control panel lives in `examples/demo.rs` (dev-dependencies only — not required for library consumers):

```bash
cargo run --example demo
```

## License

**GPL-3.0-or-later** — aligned with [chessground](https://github.com/lichess-org/chessground). See [LICENSE](LICENSE).

Porting chessground logic or using GPL-licensed board assets (e.g. cburnett pieces under `assets/`) is permitted under the same license. If you distribute binaries or combined works, you must provide corresponding source to recipients.

## Related

- [lichess-org/chessground](https://github.com/lichess-org/chessground) — behavioral reference
- [shakmaty](https://docs.rs/shakmaty) — naming guide only (not a dependency)
