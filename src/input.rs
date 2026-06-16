//! Keyboard input, mapped to high-level viewer intents.

use egui::{Context, Key};

/// One frame's worth of keyboard intents.
#[derive(Default)]
pub struct InputState {
    pub next: bool,
    pub prev: bool,
    pub first: bool,
    pub last: bool,
    pub zoom_in: bool,
    pub zoom_out: bool,
    pub reset_view: bool,
    pub toggle_fullscreen: bool,
    pub toggle_help: bool,
}

impl InputState {
    pub fn read(ctx: &Context) -> Self {
        ctx.input(|i| Self {
            next: i.key_pressed(Key::ArrowRight) || i.key_pressed(Key::Space),
            prev: i.key_pressed(Key::ArrowLeft) || i.key_pressed(Key::Backspace),
            first: i.key_pressed(Key::Home),
            last: i.key_pressed(Key::End),
            zoom_in: i.key_pressed(Key::Plus) || i.key_pressed(Key::Equals),
            zoom_out: i.key_pressed(Key::Minus),
            reset_view: i.key_pressed(Key::Num0) || i.key_pressed(Key::R),
            toggle_fullscreen: i.key_pressed(Key::F11) || i.key_pressed(Key::F),
            toggle_help: i.key_pressed(Key::H) || i.key_pressed(Key::F1),
        })
    }
}
