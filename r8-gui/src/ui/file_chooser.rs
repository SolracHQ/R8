use bevy::prelude::*;
use bevy_egui::egui;
use std::fs;
use std::path::PathBuf;

/// Mode for the file chooser: load raw ROM or assemble a source file first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileChooserMode {
  Rom,
  Asm,
}

/// Resource to track file chooser state
#[derive(Resource)]
pub struct FileChooserState {
  pub show: bool,
  pub current_path: PathBuf,
  pub selected_file: Option<PathBuf>,
  pub error_message: Option<String>,
  pub mode: FileChooserMode,
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
      show: false,
      current_path,
      selected_file: None,
      error_message: None,
      mode: FileChooserMode::Rom,
    }
  }
}

/// Helper to determine if the file should be displayed based on the current mode
fn allowed_by_mode(path: &PathBuf, mode: FileChooserMode) -> bool {
  if path.is_dir() {
    return true;
  }

  match mode {
    FileChooserMode::Rom => {
      let fname = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .to_lowercase();
      fname.ends_with(".ch8") || fname.ends_with(".rom")
    }
    FileChooserMode::Asm => {
      let fname = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .to_lowercase();
      fname.ends_with(".asm") || fname.ends_with(".s") || fname.ends_with(".8s")
    }
  }
}

/// Renders the file chooser UI and returns Some((contents, name)) if a file was loaded
pub fn file_chooser_ui(
  ui: &mut egui::Ui,
  state: &mut FileChooserState,
) -> Option<(Vec<u8>, String)> {
  let mut result = None;

  // Show which mode we're using
  ui.horizontal(|ui| {
    ui.label("Mode:");
    ui.monospace(match state.mode {
      FileChooserMode::Rom => "ROM",
      FileChooserMode::Asm => "ASM (assemble)",
    });
  });

  ui.separator();

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
    .show(ui, |ui| match fs::read_dir(&state.current_path) {
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
          if !allowed_by_mode(&path, state.mode) {
            continue;
          }

          let is_dir = path.is_dir();
          let file_name = entry.file_name().to_string_lossy().to_string();

          let display_name = if is_dir {
            format!("📁 {}", file_name)
          } else {
            match state.mode {
              FileChooserMode::Asm => format!("📝 {}", file_name),
              FileChooserMode::Rom => format!("🎮 {}", file_name),
            }
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

    let load_label = match state.mode {
      FileChooserMode::Rom => "Load ROM",
      FileChooserMode::Asm => "Assemble & Load",
    };

    if ui
      .add_enabled(load_enabled, egui::Button::new(load_label))
      .clicked()
    {
      if let Some(path) = &state.selected_file {
        match state.mode {
          FileChooserMode::Rom => match fs::read(path) {
            Ok(contents) => {
              let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "Unknown".to_string());

              result = Some((contents, name));
              state.show = false;
              state.error_message = None;
            }
            Err(e) => {
              state.error_message = Some(format!("Failed to read file: {}", e));
            }
          },

          FileChooserMode::Asm => match fs::File::open(path) {
            Ok(mut file) => {
              let mut rom = vec![];
              match r8_assembly::assemble(&mut file, &mut rom) {
                Ok(_) => {
                  let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Unknown".to_string());

                  result = Some((rom, name));
                  state.show = false;
                  state.error_message = None;
                }
                Err(e) => {
                  state.error_message = Some(format!("Failed to assemble: {}", e));
                }
              }
            }
            Err(e) => {
              state.error_message = Some(format!("Failed to open source: {}", e));
            }
          },
        }
      }
    }

    if ui.button("Cancel").clicked() {
      state.show = false;
    }
  });

  ui.separator();
  ui.label("Press F1 to toggle this window");

  result
}
