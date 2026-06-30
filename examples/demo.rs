//! Host smoke test with a small control panel.
//!
//! Run: `cargo run --example demo`

use gpui::*;
use gpui_component::button::Button;
use gpui_component::input::{Input, InputState};
use gpui_component::switch::Switch;
use gpui_component::*;
use gpui_component_assets::Assets;

use gpui_chessboard::{
    config::{EvalConfigPatch, MovableConfigPatch},
    ChessboardApi, ChessboardCallbacks, ChessboardView, Chessground, Config, Dests, EvalBarPosition,
    EvalDisplay, INITIAL_FEN, Key, MovableColor,
};

fn starting_white_pawn_dests() -> Dests {
    let mut dests = Dests::new();
    for file in ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'] {
        let orig = Key::new(&format!("{file}2")).unwrap();
        let one = Key::new(&format!("{file}3")).unwrap();
        let two = Key::new(&format!("{file}4")).unwrap();
        let mut squares = vec![one];
        if file == 'a' || file == 'h' {
            dests.insert(orig, squares);
        } else {
            squares.push(two);
            dests.insert(orig, squares);
        }
    }
    dests
}

fn main() {
    let app = gpui_platform::application()
        .with_assets(Assets)
        .with_quit_mode(QuitMode::LastWindowClosed);

    app.run(move |cx| {
        gpui_component::init(cx);

        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::centered(size(px(720.), px(720.)), cx)),
            titlebar: Some(TitleBar::title_bar_options()),
            ..Default::default()
        };

        cx.spawn(async move |cx| {
            cx.open_window(window_options, |window, cx| {
                let fen_input = cx.new(|cx| {
                    InputState::new(window, cx).default_value(INITIAL_FEN)
                });
                let config = Config {
                    movable: Some(MovableConfigPatch {
                        free: Some(false),
                        color: Some(Some(MovableColor::White)),
                        dests: Some(Some(starting_white_pawn_dests())),
                        show_dests: Some(true),
                        ..Default::default()
                    }),
                    ..Default::default()
                };
                let callbacks = ChessboardCallbacks::default();
                let (board, api) = Chessground::new(config, callbacks, window, cx);
                let shell = cx.new(|cx| DemoWindow::new(board, api, fen_input, cx));
                cx.new(|cx| Root::new(shell, window, cx))
            })
            .expect("failed to open window");
        })
        .detach();
    });
}

struct DemoWindow {
    board: Entity<ChessboardView>,
    api: ChessboardApi,
    fen_input: Entity<InputState>,
    view_only: bool,
    eval_enabled: bool,
    eval_position: EvalBarPosition,
    demo_cp: i32,
}

impl DemoWindow {
    fn new(
        board: Entity<ChessboardView>,
        api: ChessboardApi,
        fen_input: Entity<InputState>,
        _: &mut Context<Self>,
    ) -> Self {
        Self {
            board,
            api,
            fen_input,
            view_only: false,
            eval_enabled: false,
            eval_position: EvalBarPosition::Left,
            demo_cp: 0,
        }
    }

    fn apply_fen(&self, cx: &mut Context<Self>) {
        let fen = self.fen_input.read(cx).value();
        self.api.set(
            Config {
                fen: Some(fen.to_string()),
                movable: Some(MovableConfigPatch {
                    free: Some(true),
                    color: Some(Some(MovableColor::Both)),
                    show_dests: Some(true),
                    ..Default::default()
                }),
                ..Default::default()
            },
            cx,
        );
    }
}

impl Render for DemoWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let fen = self.api.get_fen(cx);

        v_flex()
            .size_full()
            .child(
                TitleBar::new().child(
                    div()
                        .text_sm()
                        .child("gpui-chessboard demo"),
                ),
            )
            .child(
                h_flex()
                    .gap_2()
                    .p_2()
                    .items_center()
                    .flex_wrap()
                    .child(div().w(px(220.)).child(Input::new(&self.fen_input)))
                    .child(
                        Button::new("apply-fen")
                            .label("Apply FEN")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.apply_fen(cx);
                            })),
                    )
                    .child(
                        Button::new("flip-board")
                            .label("Flip")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.api.toggle_orientation(cx);
                            })),
                    )
                    .child(
                        Switch::new("eval-bar")
                            .label("Eval bar")
                            .checked(self.eval_enabled)
                            .on_click(cx.listener(|this, checked, _, cx| {
                                this.eval_enabled = *checked;
                                this.api.configure_eval(
                                    EvalConfigPatch {
                                        enabled: Some(this.eval_enabled),
                                        position: Some(this.eval_position),
                                        display: Some(if this.eval_enabled {
                                            Some(EvalDisplay::cp(this.demo_cp))
                                        } else {
                                            None
                                        }),
                                        ..Default::default()
                                    },
                                    cx,
                                );
                            })),
                    )
                    .child(
                        Button::new("eval-side")
                            .label(match self.eval_position {
                                EvalBarPosition::Left => "Eval ← left",
                                EvalBarPosition::Right => "Eval right →",
                            })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.eval_position = match this.eval_position {
                                    EvalBarPosition::Left => EvalBarPosition::Right,
                                    EvalBarPosition::Right => EvalBarPosition::Left,
                                };
                                this.api.set_eval_position(this.eval_position, cx);
                            })),
                    )
                    .child(
                        Button::new("eval-searching")
                            .label("Eval …")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.api.set_eval(None, cx);
                            })),
                    )
                    .child(
                        Button::new("eval-plus")
                            .label("Eval +")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.demo_cp = (this.demo_cp + 80).clamp(-900, 900);
                                this.api.set_eval(Some(EvalDisplay::cp(this.demo_cp)), cx);
                            })),
                    )
                    .child(
                        Button::new("eval-minus")
                            .label("Eval −")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.demo_cp = (this.demo_cp - 80).clamp(-900, 900);
                                this.api.set_eval(Some(EvalDisplay::cp(this.demo_cp)), cx);
                            })),
                    )
                    .child(
                        Switch::new("view-only")
                            .label("View only")
                            .checked(self.view_only)
                            .on_click(cx.listener(|this, checked, _, cx| {
                                this.view_only = *checked;
                                this.api.set(
                                    Config {
                                        view_only: Some(this.view_only),
                                        ..Default::default()
                                    },
                                    cx,
                                );
                            })),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!("FEN: {fen}")),
                    ),
            )
            .child(
                div()
                    .id("demo-board-area")
                    .flex_1()
                    .min_h_0()
                    .min_w_0()
                    .overflow_hidden()
                    .flex()
                    .flex_col()
                    .child(self.board.clone()),
            )
    }
}
