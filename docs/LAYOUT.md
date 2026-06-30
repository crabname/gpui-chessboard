# Embedding the board (flex layout)

> The same content appears in **`cargo doc`** on the crate root (`gpui_chessboard`) under **Layout and embedding**.

`ChessboardView` is designed to **fill the space its host gives it** and draw a **centered square** inside that area (`min(width, height)`). It does **not** force its own pixel size.

If the host layout does not allocate height (or width), the board can collapse to a thin strip or disappear. This is normal flexbox behaviour — not a bug in the widget.

## How the widget lays out internally

```
ChessboardView (root div)
  flex_1, min_h_0, min_w_0
  └── chessboard-input
        size_full, min_h_0, min_w_0
        └── ChessboardElement
              size: 100% × 100%, flex_shrink: 1
              └── BoardPaintLayout: square = min(parent_w, parent_h)
```

The host must therefore provide a **bounded flex region** with non-zero height. Width alone is not enough when the parent is a column.

## Minimal working wrapper

This pattern matches `examples/demo.rs`:

```rust
div()
    .id("board-area")
    .flex_1()          // take remaining space in a column/row
    .min_h_0()         // allow shrinking below content size (critical)
    .min_w_0()         // same for horizontal flex rows
    .overflow_hidden()
    .flex()            // become a flex container
    .flex_col()        // column: child flex_1 gets height
    .child(board.clone())
```

`ChessboardView` already uses `flex_1()` on its root. It only expands when **its parent is a flex container** with available space.

## Full-height window (toolbar + board)

Put fixed-height UI (title bar, toolbar) as `flex_shrink_0` or natural height. Give the board row `flex_1().min_h_0()`:

```rust
v_flex()
    .size_full()
    .overflow_hidden()
    .child(TitleBar::new().child(/* … */))
    .child(
        h_flex()
            .flex_shrink_0()
            .p_2()
            .child(/* toolbar buttons */),
    )
    .child(
        div()
            .flex_1()
            .min_h_0()
            .min_w_0()
            .overflow_hidden()
            .flex()
            .flex_col()
            .child(board.clone()),
    )
```

## Sidebar + main content

When the board sits beside a sidebar, both columns need height from the same flex row:

```rust
h_flex()
    .flex_1()
    .min_h_0()
    .min_w_0()
    .overflow_hidden()
    .items_stretch()   // stretch sidebar and main to full row height
    .child(sidebar)    // fixed width, e.g. .w(px(280.))
    .child(
        v_flex()
            .flex_1()
            .min_w_0()
            .min_h_0()
            .overflow_hidden()
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .child(board.clone()),
            ),
    )
```

## Tabs, panels, or nested views

**Common mistake:** a tab content `div` has `flex_1()` but is **not** `flex().flex_col()`. The board child then has nothing to expand into and collapses.

```rust
// Tab bar (fixed height)
TabBar::new("tabs").flex_shrink_0() /* … */

// Tab content — must be a flex column container
div()
    .id("tab-content")
    .flex_1()
    .min_h_0()
    .min_w_0()
    .overflow_hidden()
    .flex()
    .flex_col()
    .child(
        div()
            .size_full()   // or flex_1 + min_h_0 inside the column above
            .min_h_0()
            .flex()
            .flex_col()
            .child(board.clone()),
    )
```

Every ancestor from the window root down to the board wrapper should either:

- pass through flex space (`flex_1`, `min_h_0`, `overflow_hidden`), or
- be explicitly sized (`size_full()` on the window root column).

## Checklist when the board is missing or flat

1. Root column uses `.size_full()` (or equivalent window-filling bounds).
2. The row/column that should grow uses `.flex_1().min_h_0()` (and `.min_w_0()` in horizontal layouts).
3. The **direct wrapper** around `board.clone()` uses `.flex().flex_col()`.
4. No missing link in the chain (e.g. tab content without flex display).
5. Toolbars / tab bars use `.flex_shrink_0()` so they do not steal all space incorrectly.

## Resizing behaviour

- The board **scales down** when the window shrinks; the painted square follows `min(available_width, available_height)`.
- Aspect ratio is preserved by layout (square), not by forcing a fixed `aspect_ratio` on the view.
- Do not rely on `aspect_square` on the outer host div unless you also give that div a defined size from flex; prefer the wrapper pattern above.

## Creating the view

```rust
use gpui_chessboard::{ChessboardCallbacks, Chessground, Config};

let (board, api) = Chessground::new(
    Config::default(),
    ChessboardCallbacks::default(),
    window,
    cx,
);

// In Render:
div()
    .flex_1()
    .min_h_0()
    .flex()
    .flex_col()
    .child(board.clone())
```

Update position and arrows via `api.set(config, cx)` whenever your host state changes.

## Examples in this repo

| Example | Layout pattern |
|---------|----------------|
| `examples/demo.rs` | Toolbar + single board area |

Run: `cargo run --example demo`.
