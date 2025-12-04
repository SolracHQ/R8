use bevy::prelude::*;
use bevy::window::WindowResolution;
use r8_core::constants;

mod debug;
mod display;
mod emulator;
mod input;
mod sound;

pub const SCALE: usize = 16;
pub const RESOLUTION: (u32, u32) = (
  (constants::WIDTH * SCALE) as u32,
  (constants::HEIGHT * SCALE) as u32,
);

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
    .add_plugins(debug::DebugPlugin)
    .run();
}
