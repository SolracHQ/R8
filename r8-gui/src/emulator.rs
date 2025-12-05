use bevy::prelude::*;
use r8_emulator::Emulator as CoreEmulator;

#[derive(Resource)]
pub struct Emulator(pub CoreEmulator);

/// Controls emulation execution: pause/resume, single-step requests, and a
/// clock multiplier that allows running multiple CPU ticks per update.
#[derive(Resource, Debug)]
pub struct ExecutionState {
  /// When true, the main tick loop won't be executed automatically.
  pub paused: bool,
  /// Multiplier of how many CPU ticks to run per Bevy update when not paused.
  pub clock_multiplier: u32,
  /// When true, run a single CPU tick on the next update and then clear this flag.
  pub step_request: bool,
}

impl Default for ExecutionState {
  fn default() -> Self {
    Self {
      paused: false,
      clock_multiplier: 1,
      step_request: false,
    }
  }
}

pub struct EmulatorPlugin;

impl Plugin for EmulatorPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Startup, setup_system);
    app.add_systems(Update, tick_system);
  }
}

fn setup_system(mut commands: Commands) {
  commands.insert_resource(Emulator(CoreEmulator::new()));
  commands.insert_resource(ExecutionState::default());
}

fn tick_system(mut r8: ResMut<Emulator>, mut exec: ResMut<ExecutionState>) {
  // If paused, only perform a single step when requested.
  if exec.paused {
    if exec.step_request {
      if let Err(err) = r8.0.tick() {
        log::error!("Fatal emulator error: {}", err);
        std::process::exit(1);
      }
      exec.step_request = false;
    }
    return;
  }

  // When running, execute `clock_multiplier` ticks per update.
  for _ in 0..exec.clock_multiplier {
    if let Err(err) = r8.0.tick() {
      log::error!("Fatal emulator error: {}", err);
      std::process::exit(1);
    }
  }
}
