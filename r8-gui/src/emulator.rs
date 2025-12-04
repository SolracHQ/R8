use bevy::prelude::*;
use r8_emulator::Emulator as CoreEmulator;

#[derive(Resource)]
pub struct Emulator(pub CoreEmulator);

pub struct EmulatorPlugin;

impl Plugin for EmulatorPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Startup, setup_system);
    app.add_systems(Update, tick_system);
  }
}

fn setup_system(mut commands: Commands) {
  commands.insert_resource(Emulator(CoreEmulator::new()));
}

fn tick_system(mut r8: ResMut<Emulator>) {
  if let Err(err) = r8.0.tick() {
    log::error!("Fatal emulator error: {}", err);
    std::process::exit(1);
  }
}
