use bevy::prelude::*;
use bevy_egui::egui;

use super::file_chooser::{FileChooserMode, FileChooserState};

pub const TOP_PANEL_HEIGHT: f32 = 28.0;

/// UI state for the top panel (e.g. show the last loaded file)
#[derive(Resource, Default)]
pub struct TopPanelState {
  pub latest_loaded: Option<String>,
}

pub fn top_panel_system(
  ctx: &egui::Context,
  file_state: &mut FileChooserState,
  top_state: &TopPanelState,
) {
  egui::TopBottomPanel::top("r8_top_panel")
    .exact_height(TOP_PANEL_HEIGHT)
    .show(ctx, |ui| {
      ui.horizontal_centered(|ui| {
        ui.heading("R8");

        ui.separator();

        if ui.button("📂 Load ROM").clicked() {
          file_state.show = true;
          file_state.mode = FileChooserMode::Rom;
          file_state.selected_file = None;
          file_state.error_message = None;
        }

        if ui.button("📝 Load ASM").clicked() {
          file_state.show = true;
          file_state.mode = FileChooserMode::Asm;
          file_state.selected_file = None;
          file_state.error_message = None;
        }

        ui.separator();

        if let Some(name) = &top_state.latest_loaded {
          ui.label(format!("Loaded: {}", name));
        } else {
          ui.label("No ROM loaded");
        }
      });
    });
}
