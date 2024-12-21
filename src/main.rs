use std::cell::RefCell;

use eframe::egui::{self, Align, Color32, CursorIcon, Frame, IconData, InputState, Key, Layout, Margin, RichText, Rounding, Sense, Ui, Widget};

/// The height, in pixels, of buttons on the calculator.
static BUTTON_HEIGHT: f32 = 40.;

/// The gap legnth, in pixels between buttons on the calculator.
static BUTTON_SPACING: f32 = 3.;

/// The height, in pixels, of the "screen" part of the calculator, which is the part
/// that displays the expression to evaluate.
static SCREEN_HEIGHT: f32 = 140.;

fn main() -> eframe::Result {
    let icon = image::load_from_memory(include_bytes!("../assets/images/icon.png")).unwrap().to_rgba8();
    let (icon_width, icon_height) = icon.dimensions();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([250., SCREEN_HEIGHT + BUTTON_HEIGHT * 5. + BUTTON_SPACING * 7.])
            .with_icon(IconData {
                rgba: icon.into_raw(),
                width: icon_width,
                height: icon_height,
            }),
        ..Default::default()
    };

    eframe::run_native("Silico Calculator", options, Box::new(|_cc| Ok(Box::<AppState>::default())))
}

/// Assigns the value on the left to the value on the right. This avoids borrow errors
/// with `RefCell` in this particular situation:
///
/// ```rust
/// let cell = RefCell::new(some_data);
/// *cell.borrow_mut() = cell.borrow().do_something();
/// ```
///
/// Normally, this causes an "already borrowed" error, even if `do_something()` returns
/// a value that doesn't borrow the data. This macro avoids this error, under the condition
/// that the value on the right doesn't borrow the `RefCell`.
///
/// This can be safely used with values other than `RefCell` as well, but it's relatively
/// pointless.
macro_rules! assign {
    ($left:expr => $right:expr) => {{
        let result = $right;
        $left = result;
    }};
}

#[derive(Default)]
struct AppState {
    expression: RefCell<String>,
}

impl AppState {
    pub fn clear(&self) {
        *self.expression.borrow_mut() = String::new();
    }

    pub fn evaluate(&self) {
        assign!(
            *self.expression.borrow_mut() =>
            meval::eval_str(&self.expression.borrow().replace("×", "*").replace("÷", "/")).map(|result| {
                if result.fract() == 0. {
                    format!("{result}")
                } else {
                    format!("{result:.8}").trim_end_matches('0').to_owned()
                }
            })
            .unwrap_or_else(|_error| "Error".to_owned())
        );
    }

    pub fn backspace(&self) {
        assign!(*self.expression.borrow_mut() => self.expression.borrow().trim_end().to_owned().replace("×", "*").replace("÷", "/"));

        assign!(*self.expression.borrow_mut() => if self.expression.borrow().len() == 0 {
            String::new()
        } else {
            self.expression
                .borrow()
                .get(0 .. self.expression.borrow().len() - 1)
                .unwrap()
                .trim_end()
                .to_owned()
        });

        if self.expression.borrow().ends_with(|char| matches!(char, '+' | '-' | '/' | '*')) {
            *self.expression.borrow_mut() += " ";
        }

        assign!(*self.expression.borrow_mut() => self.expression.borrow().replace("*", "×").replace("/", "÷"));
    }
}

macro_rules! button {
    ($key:expr, $display:expr, $app:expr) => {
        PressableKey::new($key, $display, || {
            *$app.expression.borrow_mut() += $display;
        })
    };

    ($key:expr, $display:tt, $app:expr,spaced) => {
        PressableKey::new($key, $display, || {
            *$app.expression.borrow_mut() += concat!(" ", $display, " ");
        })
    };
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(Frame {
                fill: Color32::from_hex("#1A1F32").unwrap(),
                ..Default::default()
            })
            .show(ctx, |ui| {
                Frame {
                    inner_margin: Margin::same(BUTTON_SPACING),
                    ..Default::default()
                }
                .show(ui, |ui| {
                    ui.add_sized([ui.available_width(), SCREEN_HEIGHT], |ui: &mut Ui| {
                        Frame {
                            inner_margin: Margin::same(10.),
                            ..Default::default()
                        }
                        .show(ui, |ui| {
                            ui.with_layout(Layout::bottom_up(Align::RIGHT), |ui| {
                                ui.label(
                                    RichText::new(self.expression.borrow().clone())
                                        .size(36.)
                                        .color(Color32::from_hex("#FFFFFF").unwrap()),
                                );
                            });
                        })
                        .response
                    });

                    ui.style_mut().spacing.item_spacing = [BUTTON_SPACING, BUTTON_SPACING].into();
                    let width = (ui.available_width() - 3. * BUTTON_SPACING) / 4.;

                    // Backspace
                    if ui.ctx().input(|input| input.key_pressed(Key::Backspace)) {
                        self.backspace();
                    }

                    macro_rules! add_button {
                        ($ui:expr, $button:expr) => {
                            $ui.add_sized([width, BUTTON_HEIGHT], $button);
                        };
                    }

                    ui.horizontal(|ui| {
                        add_button!(ui, button!(Key::C, "C", self).action(|| self.clear()));
                        add_button!(ui, button!(Key::Period, ".", self));
                        add_button!(ui, button!(Key::Num6, "^", self, spaced).hold_shift());
                        add_button!(ui, button!(Key::Slash, "÷", self, spaced));
                    });

                    ui.horizontal(|ui| {
                        add_button!(ui, button!(Key::Num7, "7", self));
                        add_button!(ui, button!(Key::Num8, "8", self));
                        add_button!(ui, button!(Key::Num9, "9", self));
                        add_button!(ui, button!(Key::Num8, "×", self, spaced).hold_shift());
                    });

                    ui.horizontal(|ui| {
                        add_button!(ui, button!(Key::Num4, "4", self));
                        add_button!(ui, button!(Key::Num5, "5", self));
                        add_button!(ui, button!(Key::Num6, "6", self));
                        add_button!(ui, button!(Key::Minus, "-", self));
                    });

                    ui.horizontal(|ui| {
                        add_button!(ui, button!(Key::Num1, "1", self));
                        add_button!(ui, button!(Key::Num2, "2", self));
                        add_button!(ui, button!(Key::Num3, "3", self));
                        add_button!(ui, button!(Key::Plus, "+", self, spaced).maybe_hold_shift());
                    });

                    ui.horizontal(|ui| {
                        add_button!(ui, button!(Key::Num9, "(", self).hold_shift());
                        add_button!(ui, button!(Key::Num0, "0", self));
                        add_button!(ui, button!(Key::Num0, ")", self).hold_shift());
                        add_button!(
                            ui,
                            button!(Key::Enter, "=", self)
                                .background("#4CC2FF")
                                .foreground("#000000")
                                .action(|| self.evaluate())
                        );
                    });
                });
            });
    }
}

#[derive(PartialEq, Eq, Hash)]
struct PressableKey<F: Fn()> {
    key: Key,
    requires_shift: Option<bool>,
    display_text: &'static str,
    action: F,
    background: Color32,
    foreground: Color32,
}

impl<F: Fn()> PressableKey<F> {
    fn new(key: Key, display_text: &'static str, action: F) -> Self {
        Self {
            key,
            display_text,
            action,
            requires_shift: Some(false),
            background: if display_text.starts_with(|character: char| character.is_ascii_digit()) {
                Color32::from_hex("#353A4E").unwrap()
            } else {
                Color32::from_hex("#2C2C40").unwrap()
            },
            foreground: Color32::WHITE,
        }
    }

    fn is_pressed(&self, input: &InputState) -> bool {
        if !input.key_pressed(self.key) {
            return false;
        }

        if let Some(requires_shift) = self.requires_shift {
            return !requires_shift ^ input.modifiers.shift;
        }

        true
    }

    fn action<G: Fn()>(self, action: G) -> PressableKey<G> {
        PressableKey {
            action,
            key: self.key,
            display_text: self.display_text,
            requires_shift: self.requires_shift,
            background: self.background,
            foreground: self.foreground,
        }
    }

    fn hold_shift(mut self) -> Self {
        self.requires_shift = Some(true);
        self
    }

    fn maybe_hold_shift(mut self) -> Self {
        self.requires_shift = None;
        self
    }

    fn background(mut self, color: &str) -> Self {
        self.background = Color32::from_hex(color).unwrap();
        self
    }

    fn foreground(mut self, color: &str) -> Self {
        self.foreground = Color32::from_hex(color).unwrap();
        self
    }
}

impl<F: Fn()> Widget for PressableKey<F> {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let frame = Frame {
            fill: self.background,
            rounding: Rounding::same(3.),
            ..Default::default()
        };

        //
        // HACK: Below is just the code from `Frame::show()` expanded. This is required because
        // `Frames` are hardcoded to use `Sense::hover()`, meaning they can't listen for clicks.
        // So, we inline all of the code from `Frame::show()` and replace `Sense::hover()` with
        // `Sense::clicked()`.
        //
        // See https://github.com/emilk/egui/blob/36a70e12c3a8a70308a4faa15799d557a5c0a064/crates/egui/src/containers/frame.rs#L323
        //
        let add_contents = |ui: &mut Ui| ui.label(RichText::new(self.display_text).strong().color(self.foreground).size(15.));
        let mut prepared = frame.begin(ui);
        add_contents(&mut prepared.content_ui);
        prepared.paint(ui);
        let content_with_margin = prepared.content_ui.min_rect() + prepared.frame.inner_margin + prepared.frame.outer_margin;
        let response = ui.allocate_rect(content_with_margin, Sense::click());

        if response.hovered() {
            ui.ctx().output_mut(|output| output.cursor_icon = CursorIcon::PointingHand);
        }

        if response.clicked() || ui.ctx().input(|input| self.is_pressed(input)) {
            (self.action)();
        }

        response
    }
}
