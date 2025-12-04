use crate::emulator::Emulator;
use crate::RESOLUTION;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use r8_core::constants;

const DEBUG_PANEL_HEIGHT: f32 = 150.0;

#[derive(Resource)]
pub struct DebugState {
  pub enable: bool,
}

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugins(EguiPlugin::default())
      .insert_resource(DebugState { enable: false })
      .add_systems(EguiPrimaryContextPass, (toggle_system, debug_ui_system));
  }
}

fn toggle_system(
  mut debug_state: ResMut<DebugState>,
  keyboard_input: Res<ButtonInput<KeyCode>>,
  mut window_query: Query<&mut Window, With<PrimaryWindow>>,
  mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
  if keyboard_input.just_pressed(KeyCode::Escape) {
    debug_state.enable = !debug_state.enable;

    if let Ok(mut window) = window_query.single_mut() {
      let base_height = RESOLUTION.1 as f32;
      if debug_state.enable {
        window
          .resolution
          .set(RESOLUTION.0 as f32, base_height + DEBUG_PANEL_HEIGHT);
        let Ok(mut camera) = camera_query.single_mut() else {
          return;
        };
        camera.translation.y = -DEBUG_PANEL_HEIGHT / 2.0;
      } else {
        window.resolution.set(RESOLUTION.0 as f32, base_height);
        let Ok(mut camera) = camera_query.single_mut() else {
          return;
        };
        camera.translation.y = 0.0;
      }
    }
  }
}

fn debug_ui_system(
  mut contexts: EguiContexts,
  emulator: Res<Emulator>,
  debug_state: Res<DebugState>,
) {
  if !debug_state.enable {
    return;
  }

  let Ok(ctx) = contexts.ctx_mut() else {
    return;
  };

  egui::TopBottomPanel::bottom("debug_panel")
    .exact_height(DEBUG_PANEL_HEIGHT)
    .show(ctx, |ui| {
      ui.heading("Debug Panel");
      ui.separator();

      ui.horizontal(|ui| {
        ui.label(format!("PC: 0x{:03X}", emulator.0.pc().inner()));
        ui.separator();
        ui.label(format!("I: 0x{:03X}", emulator.0.i().inner()));
        ui.separator();
        ui.label(format!("SP: 0x{:03X}", emulator.0.stack().len()));
        ui.separator();
        ui.label(format!("DT: 0x{:02X}", emulator.0.delay_timer()));
        ui.separator();
        ui.label(format!("ST: 0x{:02X}", emulator.0.sound_timer()));
      });

      ui.horizontal(|ui| {
        ui.label(format!("State: {:?}", emulator.0.state()));
        ui.separator();
        if let Ok(opcode) = emulator.0.fetch_opcode() {
          ui.label(format!("Opcode: {}", opcode));
        }
      });

      ui.separator();
      ui.label("Registers:");

      // Render register values 8 per row (V0..V15)
      for i in (0..constants::REGISTER_COUNT).step_by(8) {
        ui.horizontal(|ui| {
          for j in i..usize::min(i + 8, constants::REGISTER_COUNT) {
            let idx = j as u8;
            let value = *emulator.0.v_registers().try_index(idx).unwrap();
            ui.label(format!("V{:X}: {:02X}", idx, value));
            if j < i + 7 && j < constants::REGISTER_COUNT - 1 {
              ui.separator();
            }
          }
        });
      }
    });
}
