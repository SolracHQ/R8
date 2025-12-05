mod bottom_panel;
mod file_chooser;
mod right_panel;
mod state;
mod top_panel;

pub use bottom_panel::BOTTOM_PANEL_HEIGHT;
pub use file_chooser::{FileChooserMode, FileChooserState};
pub use right_panel::RIGHT_PANEL_WIDTH;
pub use state::UiPanelState;
pub use top_panel::{TopPanelState, TOP_PANEL_HEIGHT};

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
use std::io::Cursor;

use crate::emulator::{Emulator, ExecutionState};

/// Message event for loading a ROM into the emulator from the UI
#[derive(Message)]
pub struct UiLoadRomMessage {
  pub contents: Vec<u8>,
  pub name: String,
}

/// Top panel and file chooser plugin
pub struct UiPlugin;

impl Plugin for UiPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_message::<UiLoadRomMessage>()
      .init_resource::<FileChooserState>()
      .init_resource::<TopPanelState>()
      .init_resource::<UiPanelState>()
      .add_systems(EguiPrimaryContextPass, (ui_system, rom_loaded_system));
  }
}

/// Main UI system that renders all panels
fn ui_system(
  mut contexts: EguiContexts,
  mut file_state: ResMut<FileChooserState>,
  top_state: Res<TopPanelState>,
  mut panel_state: ResMut<UiPanelState>,
  mut exec: ResMut<ExecutionState>,
  emulator: Res<Emulator>,
  mut rom_writer: MessageWriter<UiLoadRomMessage>,
) {
  let Ok(ctx) = contexts.ctx_mut() else {
    return;
  };

  // Always render top panel
  top_panel::top_panel_system(ctx, &mut file_state, &top_state);

  // Always render bottom panel with playback controls
  bottom_panel::bottom_panel_system(ctx, &mut exec, &mut panel_state);

  // Render right debug panel if enabled
  if panel_state.show_debug {
    right_panel::right_panel_system(ctx, &emulator, &mut panel_state.memory_inspector);
  }

  // Render file chooser as a floating window if open
  if file_state.show {
    render_file_chooser_window(ctx, &mut file_state, &mut rom_writer);
  }
}

/// Renders the file chooser as a centered window
fn render_file_chooser_window(
  ctx: &egui::Context,
  state: &mut FileChooserState,
  rom_writer: &mut MessageWriter<UiLoadRomMessage>,
) {
  egui::Window::new("File Browser")
    .collapsible(false)
    .resizable(false)
    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
    .show(ctx, |ui| {
      if let Some((contents, name)) = file_chooser::file_chooser_ui(ui, state) {
        rom_writer.write(UiLoadRomMessage { contents, name });
      }
    });
}

fn rom_loaded_system(
  mut rom_reader: MessageReader<UiLoadRomMessage>,
  mut emulator: ResMut<Emulator>,
  mut top_state: ResMut<TopPanelState>,
) {
  for msg in rom_reader.read() {
    match emulator.0.load_rom(Cursor::new(&msg.contents)) {
      Ok(_) => {
        log::info!("Loaded ROM (UI): {}", msg.name);
        top_state.latest_loaded = Some(msg.name.clone());
      }
      Err(e) => {
        log::error!("Failed to load ROM {}: {}", msg.name, e);
        top_state.latest_loaded = None;
      }
    }
  }
}
