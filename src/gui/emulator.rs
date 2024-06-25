use bevy::prelude::*;

#[derive(Resource)]
pub struct Emulator(pub r8::emulator::Emulator);

pub struct EmulatorPlugin;

impl Plugin for EmulatorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_system);
        app.add_systems(Update, tick_system);
    }
}

fn setup_system(mut commands: Commands) {
    commands.insert_resource(Emulator(r8::emulator::Emulator::new()));
}

fn tick_system(mut r8: ResMut<Emulator>) {
    if let Err(err) = r8.0.tick() {
        log::error!("Fatal emulator error: {}", err);
        std::process::exit(1);
    }
}