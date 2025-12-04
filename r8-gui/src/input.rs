use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
use std::fs;
use std::path::PathBuf;

use crate::emulator::Emulator;

/// Message event for loading a ROM into the emulator
#[derive(Message)]
pub struct LoadRomMessage {
  pub contents: Vec<u8>,
  pub name: String,
}

/// Resource to track file chooser state
#[derive(Resource)]
struct FileChooserState {
  show: bool,
  current_path: PathBuf,
  selected_file: Option<PathBuf>,
  error_message: Option<String>,
}

impl Default for FileChooserState {
  fn default() -> Self {
    // Start in the roms directory if it exists, otherwise current directory
    let current_path = std::env::current_dir()
      .unwrap_or_else(|_| PathBuf::from("."))
      .join("roms");

    let current_path = if current_path.exists() {
      current_path
    } else {
      std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    };

    Self {
      show: true, // Show at startup
      current_path,
      selected_file: None,
      error_message: None,
    }
  }
}

/// Input plugin that handles loading ROMs via egui dialog and mapping keyboard input to the emulator.
pub struct InputPlugin;

impl Plugin for InputPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_message::<LoadRomMessage>()
      .init_resource::<FileChooserState>()
      .add_systems(
        EguiPrimaryContextPass,
        (
          file_chooser_ui_system,
          rom_loaded_system,
          emulator_keys_system,
        ),
      );
  }
}

fn file_chooser_ui_system(
  mut contexts: EguiContexts,
  mut state: ResMut<FileChooserState>,
  mut rom_writer: MessageWriter<LoadRomMessage>,
  keyboard_input: Res<ButtonInput<KeyCode>>,
) {
  // Toggle file chooser with F1
  if keyboard_input.just_pressed(KeyCode::F1) {
    state.show = !state.show;
    state.error_message = None;
  }

  if !state.show {
    return;
  }

  let Ok(ctx) = contexts.ctx_mut() else {
    return;
  };

  egui::TopBottomPanel::top("file chooser").show(ctx, |ui| {
    // Current path display
    ui.horizontal(|ui| {
      ui.label("Path:");
      ui.monospace(state.current_path.display().to_string());
    });

    ui.separator();

    // Navigation buttons
    ui.horizontal(|ui| {
      if ui.button("⬆ Parent").clicked() {
        if let Some(parent) = state.current_path.parent() {
          state.current_path = parent.to_path_buf();
          state.selected_file = None;
          state.error_message = None;
        }
      }

      if ui.button("🏠 Home").clicked() {
        if let Some(home) = dirs::home_dir() {
          state.current_path = home;
          state.selected_file = None;
          state.error_message = None;
        }
      }

      if ui.button("🔄 Refresh").clicked() {
        state.error_message = None;
      }
    });

    ui.separator();

    // File list with scroll area
    egui::ScrollArea::vertical()
      .max_height(300.0)
      .show(ui, |ui| {
        match fs::read_dir(&state.current_path) {
          Ok(entries) => {
            let mut items: Vec<_> = entries.filter_map(|e| e.ok()).collect();
            items.sort_by(|a, b| {
              let a_is_dir = a.path().is_dir();
              let b_is_dir = b.path().is_dir();
              match (a_is_dir, b_is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.file_name().cmp(&b.file_name()),
              }
            });

            for entry in items {
              let path = entry.path();
              let is_dir = path.is_dir();
              let file_name = entry.file_name().to_string_lossy().to_string();

              // Only show directories and .ch8 files
              if !is_dir
                && !file_name.to_lowercase().ends_with(".ch8")
                && !file_name.to_lowercase().ends_with(".rom")
              {
                continue;
              }

              let display_name = if is_dir {
                format!("📁 {}", file_name)
              } else {
                format!("🎮 {}", file_name)
              };

              let is_selected = state.selected_file.as_ref() == Some(&path);

              if ui.selectable_label(is_selected, &display_name).clicked() {
                if is_dir {
                  state.current_path = path;
                  state.selected_file = None;
                  state.error_message = None;
                } else {
                  state.selected_file = Some(path);
                  state.error_message = None;
                }
              }
            }
          }
          Err(e) => {
            ui.colored_label(
              egui::Color32::RED,
              format!("Error reading directory: {}", e),
            );
          }
        }
      });

    ui.separator();

    // Selected file display
    if let Some(selected) = &state.selected_file {
      ui.horizontal(|ui| {
        ui.label("Selected:");
        ui.monospace(selected.file_name().unwrap_or_default().to_string_lossy());
      });
    }

    // Error message display
    if let Some(error) = &state.error_message {
      ui.colored_label(egui::Color32::RED, error);
    }

    ui.separator();

    // Action buttons
    ui.horizontal(|ui| {
      let load_enabled = state.selected_file.is_some();

      if ui
        .add_enabled(load_enabled, egui::Button::new("Load ROM"))
        .clicked()
      {
        if let Some(path) = &state.selected_file {
          match fs::read(path) {
            Ok(contents) => {
              let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "Unknown".to_string());

              rom_writer.write(LoadRomMessage { contents, name });
              state.show = false;
              state.error_message = None;
            }
            Err(e) => {
              state.error_message = Some(format!("Failed to read file: {}", e));
            }
          }
        }
      }

      if ui.button("Cancel").clicked() {
        state.show = false;
      }
    });

    ui.separator();
    ui.label("Press F1 to toggle this window");
  });
}

fn rom_loaded_system(
  mut rom_reader: MessageReader<LoadRomMessage>,
  mut emulator: ResMut<Emulator>,
) {
  for msg in rom_reader.read() {
    match emulator.0.load_rom(std::io::Cursor::new(&msg.contents)) {
      Ok(_) => {
        log::info!("Loaded ROM: {}", msg.name);
      }
      Err(e) => {
        log::error!("Failed to load ROM {}: {}", msg.name, e);
      }
    }
  }
}

fn emulator_keys_system(mut r8: ResMut<Emulator>, keyboard_input: Res<ButtonInput<KeyCode>>) {
  use r8_emulator::Key;

  /// Map Real KeyCodes to the corresponding Chip8 Virtual Keys
  fn map_key(key: &KeyCode) -> Option<Key> {
    Some(match key {
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
    })
  }

  // When a mapped key is pressed, notify the emulator
  keyboard_input
    .get_just_pressed()
    .filter_map(map_key)
    .for_each(|key| r8.0.press_key(key));

  // When a mapped key is released, notify the emulator
  keyboard_input
    .get_just_released()
    .filter_map(map_key)
    .for_each(|key| r8.0.release_key(key));
}
