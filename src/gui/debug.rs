use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use r8::constants::REGISTER_COUNT;
use crate::emulator::Emulator;

#[derive(Resource)]
struct DebugState {
    enable: bool
}

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .insert_resource(DebugState { enable: false })
            .add_systems(Update, debug_ui_system)
            .add_systems(Update, toggle_system);
    }
}

fn toggle_system(mut debug_state: ResMut<DebugState>, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.get_pressed().any(|k| matches!(k, KeyCode::Escape)) {
        debug_state.enable = !debug_state.enable;
    }
}

fn debug_ui_system(mut contexts: EguiContexts, emulator: Res<Emulator>, debug_state: Res<DebugState>) {

    if !debug_state.enable {
        return;
    }

    egui::Window::new("Debug Window").show(contexts.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            ui.label(format!("PC: 0x{:03X}", emulator.0.pc().inner()));
            ui.label(format!("I: 0x{:03X}", emulator.0.i().inner()));
            ui.label(format!("SP: 0x{:03X}", emulator.0.stack().len()));
            ui.label(format!("DT: 0x{:02X}", emulator.0.delay_timer()));
        });

        ui.horizontal(|ui| {
            ui.label(format!("ST: 0x{:02X}", emulator.0.sound_timer()));
            ui.label(format!("State: {:?}", emulator.0.state()));
            ui.label(format!("{}", emulator.0.fetch_opcode().unwrap()));
        });

        for i in (0..REGISTER_COUNT as u8).step_by(4) {
            ui.horizontal(|ui| {
                for j in i..i + 4 {
                    ui.label(format!(
                        "V{:X}: 0x{:02X}",
                        j,
                        emulator.0.v_registers().try_index(j).unwrap()
                    ));
                }
            });
        }
    });
}