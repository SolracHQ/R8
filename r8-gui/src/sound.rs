use crate::emulator::Emulator;
use bevy::prelude::*;

#[derive(Component)]
struct Sound;

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Startup, setup_plugin);
    app.add_systems(Update, update_sound);
  }
}

fn setup_plugin(mut commands: Commands, asset_server: Res<AssetServer>) {
  commands.spawn((AudioPlayer::new(asset_server.load("out.ogg")), Sound));
}

fn update_sound(r8: Res<Emulator>, sound: Query<&AudioSink, With<Sound>>) {
  if let Ok(sink) = sound.single() {
    match r8.0.sound_timer() {
      0 => sink.pause(),
      _ => sink.play(),
    }
  }
}
