use std::{collections::HashSet, sync::LazyLock};

use eframe::egui::{self, Align, Color32, Frame, IconData, InputState, Key, Layout, Margin, RichText, Rounding, Sense, Ui, Widget};
use unicode_segmentation::UnicodeSegmentation;

/// The height, in pixels, of buttons on the calculator.
static BUTTON_HEIGHT: f32 = 40.;

/// The gap legnth, in pixels between buttons on the calculator.
static BUTTON_SPACING: f32 = 3.;

/// The height, in pixels, of the "screen" part of the calculator, which is the part
/// that displays the expression to evaluate.
static SCREEN_HEIGHT: f32 = 120.;

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

    eframe::run_native("Calculator", options, Box::new(|_cc| Ok(Box::<AppState>::default())))
}

#[derive(Default)]
struct AppState {
    expression: String,
}

static KEYS: LazyLock<HashSet<PressableKey>> = LazyLock::new(|| {
    HashSet::from([
        // Numbers
        PressableKey::no_shift(Key::Num1, "1"),
        PressableKey::no_shift(Key::Num2, "2"),
        PressableKey::no_shift(Key::Num3, "3"),
        PressableKey::no_shift(Key::Num4, "4"),
        PressableKey::no_shift(Key::Num5, "5"),
        PressableKey::no_shift(Key::Num6, "6"),
        PressableKey::no_shift(Key::Num7, "7"),
        PressableKey::no_shift(Key::Num8, "8"),
        PressableKey::no_shift(Key::Num9, "9"),
        PressableKey::no_shift(Key::Num0, "0"),
        // Operators
        PressableKey::shift(Key::Num9, "("),
        PressableKey::shift(Key::Num0, ")"),
        PressableKey::shift(Key::Num6, " ^ "),
        PressableKey::maybe_shift(Key::Plus, " + "),
        PressableKey::no_shift(Key::Minus, " - "),
        PressableKey::shift(Key::Num8, " × "),
        PressableKey::no_shift(Key::Slash, " / "),
    ])
});

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
                                ui.label(RichText::new(&self.expression).size(32.).color(Color32::from_hex("#FFFFFF").unwrap()));
                            });
                        })
                        .response
                    });

                    ui.style_mut().spacing.item_spacing = [BUTTON_SPACING, BUTTON_SPACING].into();
                    let width = (ui.available_width() - 3. * BUTTON_SPACING) / 4.;

                    for key in KEYS.iter() {
                        if ui.ctx().input(|input| key.is_pressed(input)) {
                            if &self.expression == "Error" {
                                self.expression = String::new();
                            }
                            self.expression += key.expression;
                        }
                    }

                    // Enter
                    if ui.ctx().input(|input| input.key_pressed(Key::Enter)) {
                        self.expression = meval::eval_str(&self.expression.replace("×", "*"))
                            .map(|result| result.to_string())
                            .unwrap_or_else(|_error| "Error".to_owned());
                    }

                    // Clear
                    if ui.ctx().input(|input| input.key_pressed(Key::C)) {
                        self.expression = String::new();
                    }

                    // Backspace
                    if ui.ctx().input(|input| input.key_pressed(Key::Backspace)) {
                        self.expression = self.expression.trim_end().to_owned();
                        self.expression = self
                            .expression
                            .grapheme_indices(true)
                            .filter_map(|(index, character)| (index != self.expression.len()).then_some(character))
                            .collect();
                    }

                    ui.horizontal(|ui| {
                        ui.add_sized([width, BUTTON_HEIGHT], Button::operator("C"));
                        ui.add_sized([width, BUTTON_HEIGHT], Button::operator("B"));
                        ui.add_sized([width, BUTTON_HEIGHT], Button::operator("^"));
                        ui.add_sized([width, BUTTON_HEIGHT], Button::operator("+"));
                    });

                    ui.horizontal(|ui| {
                        ui.add_sized([width, BUTTON_HEIGHT], Button::number("7"));
                        ui.add_sized([width, BUTTON_HEIGHT], Button::number("8"));
                        ui.add_sized([width, BUTTON_HEIGHT], Button::number("9"));
                        ui.add_sized([width, BUTTON_HEIGHT], Button::operator("÷"));
                    });

                    ui.horizontal(|ui| {
                        ui.add_sized([width, BUTTON_HEIGHT], Button::number("4"));
                        ui.add_sized([width, BUTTON_HEIGHT], Button::number("5"));
                        ui.add_sized([width, BUTTON_HEIGHT], Button::number("6"));
                        ui.add_sized([width, BUTTON_HEIGHT], Button::operator("×"));
                    });

                    ui.horizontal(|ui| {
                        ui.add_sized([width, BUTTON_HEIGHT], Button::number("1"));
                        ui.add_sized([width, BUTTON_HEIGHT], Button::number("2"));
                        ui.add_sized([width, BUTTON_HEIGHT], Button::number("3"));
                        ui.add_sized([width, BUTTON_HEIGHT], Button::operator("-"));
                    });

                    ui.horizontal(|ui| {
                        ui.add_sized([width, BUTTON_HEIGHT], Button::operator("("));
                        ui.add_sized([width, BUTTON_HEIGHT], Button::number("0"));
                        ui.add_sized([width, BUTTON_HEIGHT], Button::operator(")"));
                        ui.add_sized([width, BUTTON_HEIGHT], Button::operator("=").background("#4CC2FF"));
                    });
                });
            });
    }
}

struct Button {
    text: &'static str,
    background: Color32,
    key: Option<&'static PressableKey>,
}

impl Button {
    pub fn number(text: &'static str) -> Button {
        Self {
            text: text.into(),
            background: Color32::from_hex("#353A4E").unwrap(),
            key: KEYS.iter().find(|key| key.expression.trim() == text),
        }
    }

    pub fn operator(text: &'static str) -> Button {
        Self {
            text: text.into(),
            background: Color32::from_hex("#2C2C40").unwrap(),
            key: KEYS.iter().find(|key| key.expression.trim() == text),
        }
    }

    pub fn background(mut self, color: &str) -> Button {
        self.background = Color32::from_hex(color).unwrap();
        self
    }
}

impl Widget for Button {
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
        let add_contents = |ui: &mut Ui| ui.label(RichText::new(self.text).strong().color(Color32::from_hex("#FFFFFF").unwrap()).size(15.));
        let mut prepared = frame.begin(ui);
        add_contents(&mut prepared.content_ui);
        prepared.paint(ui);
        let content_with_margin = prepared.content_ui.min_rect() + prepared.frame.inner_margin + prepared.frame.outer_margin;
        let response = ui.allocate_rect(content_with_margin, Sense::click());

        if let Some(key) = self.key {
            if response.clicked() {}
        };

        response
    }
}

#[derive(PartialEq, Eq, Hash)]
struct PressableKey {
    key: Key,
    requires_shift: Option<bool>,
    expression: &'static str,
}

impl PressableKey {
    /// Creates a new pressable key, which will only claim to be pressed when shift is *not*
    /// being held down, but the given key *is* pressed.
    ///
    /// # Parameters
    ///
    /// - `key` - The key to listen for.
    /// - `expression` - the tokens to append to the calculator's screen.
    ///
    /// # Returns
    ///
    /// The `PressableKey` object.
    fn no_shift(key: Key, expression: &'static str) -> PressableKey {
        PressableKey {
            key,
            expression,
            requires_shift: Some(false),
        }
    }

    /// Creates a new pressable key, which will only claim to be pressed when shift is being held down
    /// while the given key is pressed.
    ///
    /// # Parameters
    ///
    /// - `key` - The key to listen for.
    /// - `expression` - the tokens to append to the calculator's screen.
    ///
    /// # Returns
    ///
    /// The `PressableKey` object.
    fn shift(key: Key, expression: &'static str) -> PressableKey {
        PressableKey {
            key,
            expression,
            requires_shift: Some(true),
        }
    }

    /// Creates a new pressable key, which will claim to be pressed when the given key is
    /// pressed, regardless of whether or not shift is being held.
    ///
    /// # Parameters
    ///
    /// - `key` - The key to listen for.
    /// - `expression` - the tokens to append to the calculator's screen.
    ///
    /// # Returns
    ///
    /// The `PressableKey` object.
    fn maybe_shift(key: Key, expression: &'static str) -> PressableKey {
        PressableKey {
            key,
            expression,
            requires_shift: None,
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
}
