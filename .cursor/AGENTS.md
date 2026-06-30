# gpui-chessboard

Embeddable chessboard **UI library** for GPUI — displays positions and moves, like [lichess-org/chessground](https://github.com/lichess-org/chessground) for desktop apps. **Not a standalone chess application.**

**Target architecture** — [docs/ARCHITECTURE.md](../docs/ARCHITECTURE.md).

## What this crate is

| In scope | Out of scope |
|----------|--------------|
| Render board, pieces, highlights | Chess rules / legality |
| User click/drag input | Game state, clocks, PGN |
| `Config` in, `UserMove` callbacks out | Own window / menus / persistence |
| Embed in host `Render` layout | Engine protocols |

The **host app** (e.g. analysis tool, game client) depends on this crate, embeds `ChessboardView`, and passes `movable.dests` + position updates via `ChessboardApi::set`.

## References

- Board UI and API — `../chessground`
- GPUI embed patterns — [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui) (`Entity`, `Render`, `cx.spawn`)
- Type naming — [shakmaty](https://docs.rs/shakmaty) (guide only, no dependency)

## Stack

- **Rust** edition 2024
- **gpui** — required library dependency
- **gpui_platform**, **gpui-component** — `[dev-dependencies]` for `examples/` only

Do **not** add chess rule crates to this project.

## License

**GPL-3.0-or-later** — same as [chessground](../chessground). Derivative work; source must be provided to users of distributed binaries.

## Module architecture

```
src/lib.rs       — public API: Chessground, ChessboardView, ChessboardApi, Config
src/types.rs     — Key, Square, Piece, Dests, UserMove
src/fen.rs       — piece-placement FEN
src/board.rs …   — headless interaction (no rules engine)
src/element/     — rendering
src/view.rs      — embeddable GPUI view
src/api.rs       — imperative API for host

examples/demo.rs — minimal host example
```

## Host integration (primary usage)

```rust
// Host creates and embeds the widget:
let (board, api) = Chessground::new(config, window, cx);

// In host Render:
div().flex_1().child(board.clone())

// Host updates after position changes:
api.set(host.to_board_config(), cx);
```

`movable.dests` and move callbacks always come from the host.

## Build

```bash
cargo check                 # library
cargo run --example demo    # smoke-test example
```

## Agent constraints

- **Library first** — every feature serves embeddable `lib` API
- Usage examples go in `examples/`, never `src/main.rs`
- No app state, game logic, or menus in this crate
- Minimal diff; formats in `types.rs`, rendering in `element/`
- Tests and docs — only when requested
- Commits — only on explicit user request
