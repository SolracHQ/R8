use bevy::prelude::*;
use bevy_egui::egui;

use super::state::UiPanelState;
use crate::emulator::ExecutionState;

pub const BOTTOM_PANEL_HEIGHT: f32 = 32.0;

/// Renders the bottom panel with always-visible playback controls
pub fn bottom_panel_system(
  ctx: &egui::Context,
  exec: &mut ResMut<ExecutionState>,
  panel_state: &mut ResMut<UiPanelState>,
) {
  egui::TopBottomPanel::bottom("r8_bottom_panel")
    .exact_height(BOTTOM_PANEL_HEIGHT)
    .show(ctx, |ui| {
      ui.horizontal_centered(|ui| {
        // Pause / Resume toggle
        let pause_label = if exec.paused {
          "▶ Resume"
        } else {
          "⏸ Pause"
        };
        if ui.button(pause_label).clicked() {
          exec.paused = !exec.paused;
        }

        ui.separator();

        // Step — only enabled when paused
        if ui
          .add_enabled(exec.paused, egui::Button::new("⏭ Step"))
          .clicked()
        {
          exec.step_request = true;
        }

        ui.separator();

        // Clock Speed multiplier (1..=10)
        let mut multiplier = exec.clock_multiplier as i32;
        ui.label("Speed:");
        ui.add(egui::Slider::new(&mut multiplier, 1..=10).show_value(true));
        exec.clock_multiplier = multiplier.max(1) as u32;

        ui.separator();

        // Status indicator
        let status_text = if exec.paused {
          "⏸ Paused"
        } else {
          "▶ Running"
        };
        ui.label(status_text);

        ui.separator();

        // Debug panel toggle
        let debug_label = if panel_state.show_debug {
          "🔧 Hide Debug"
        } else {
          "🔧 Show Debug"
        };
        if ui.button(debug_label).clicked() {
          panel_state.show_debug = !panel_state.show_debug;
        }
      });
    });
}
