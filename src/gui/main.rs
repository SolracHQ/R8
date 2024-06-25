use bevy::prelude::*;
use bevy::window::WindowResolution;

mod display;
mod emulator;
mod input;
mod sound;

const SCALE: usize = 12;
const RESOLUTION: (f32, f32) = ((r8::constants::WIDTH * SCALE)as f32, (r8::constants::HEIGHT * SCALE) as f32);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "R8 - Chip8 Emulator".to_string(),
                resolution: WindowResolution::new(RESOLUTION.0, RESOLUTION.1),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(emulator::EmulatorPlugin)
        .add_plugins(display::DisplayPlugin)
        .add_plugins(input::InputPlugin)
        .add_plugins(sound::SoundPlugin)
        .run();
}
