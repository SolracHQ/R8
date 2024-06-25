use bevy::prelude::*;
use bevy_file_dialog::prelude::*;
use crate::emulator::Emulator;

pub struct InputPlugin;

struct Rom;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FileDialogPlugin::new().with_load_file::<Rom>());
        app.add_systems(Startup, init_system);
        app.add_systems(Update, (emulator_keys_system, rom_loaded_system));
    }
}

fn init_system(mut commands: Commands) {
    commands.dialog().load_file::<Rom>();
}

fn rom_loaded_system(mut ev_loaded: EventReader<DialogFileLoaded<Rom>>, mut emulator: ResMut<Emulator>) {
    for ev in ev_loaded.read() {
        emulator.0.load_rom(std::io::Cursor::new(&ev.contents)).unwrap()
    }
}

fn emulator_keys_system(mut r8: ResMut<Emulator>, keyboard_input: Res<ButtonInput<KeyCode>>) {
    use r8::keyboard::Key;

    /// Map Real KeyCodes to the corresponding Chip8 Virtual Keys
    fn map_key(key: &KeyCode) -> Option<Key> {
        return Some(match key {
            KeyCode::Digit1 => Key::K1,
            KeyCode::Digit2 => Key::K2,
            KeyCode::Digit3 => Key::K3,
            KeyCode::Digit4 => Key::KC,
            KeyCode::KeyQ => Key::K4,
            KeyCode::KeyW => Key::K5,
            KeyCode::KeyE => Key::K6,
            KeyCode::KeyR => Key::KD,
            KeyCode::KeyA => Key::K7,
            KeyCode::KeyS => Key::K8,
            KeyCode::KeyD => Key::K9,
            KeyCode::KeyF => Key::KE,
            KeyCode::KeyZ => Key::KA,
            KeyCode::KeyX => Key::K0,
            KeyCode::KeyC => Key::KB,
            KeyCode::KeyV => Key::KF,
            _ => return None,
        });
    }

    keyboard_input
        .get_just_pressed()
        .filter_map(map_key)
        .for_each(|key| r8.0.press_key(key));

    keyboard_input
        .get_just_released()
        .filter_map(map_key)
        .for_each(|key| r8.0.release_key(key));
}