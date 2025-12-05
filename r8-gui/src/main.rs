use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_egui::EguiPlugin;
use r8_core::constants;

mod display;
mod emulator;
mod input;
mod sound;
mod ui;

pub const SCALE: usize = 16;
pub const RESOLUTION: (u32, u32) = (
  (constants::WIDTH * SCALE) as u32,
  (constants::HEIGHT * SCALE) as u32,
);

fn main() {
  // Calculate initial window size accounting for UI panels
  let window_width = RESOLUTION.0;
  let window_height = RESOLUTION.1 + ui::TOP_PANEL_HEIGHT as u32 + ui::BOTTOM_PANEL_HEIGHT as u32;

  App::new()
    .add_plugins(DefaultPlugins.set(WindowPlugin {
      primary_window: Some(Window {
        title: "R8 - Chip8 Emulator".to_string(),
        resolution: WindowResolution::new(window_width, window_height),
        resizable: false,
        ..default()
      }),
      ..default()
    }))
    .add_plugins(EguiPlugin::default())
    .add_plugins(emulator::EmulatorPlugin)
    .add_plugins(display::DisplayPlugin)
    .add_plugins(ui::UiPlugin)
    .add_plugins(input::InputPlugin)
    .add_plugins(sound::SoundPlugin)
    .run();
}
