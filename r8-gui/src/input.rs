use crate::emulator::Emulator;
use crate::ui::{FileChooserMode, FileChooserState, UiPanelState};
use crate::ui::{BOTTOM_PANEL_HEIGHT, RIGHT_PANEL_WIDTH, TOP_PANEL_HEIGHT};
use crate::RESOLUTION;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

/// Input plugin is responsible for routing keyboard input into emulator keys
/// and for handling global hotkeys: toggling the debug panel and the file chooser.
pub struct InputPlugin;

impl Plugin for InputPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Startup, setup_camera_system);
    app.add_systems(
      Update,
      (
        input_toggle_system,
        emulator_keys_system,
        camera_update_system,
      ),
    );
  }
}

/// Setup the camera with correct initial position accounting for UI panels
fn setup_camera_system(mut camera_query: Query<&mut Transform, With<Camera2d>>) {
  if let Ok(mut camera) = camera_query.single_mut() {
    // Offset camera to account for top and bottom panels
    // Top panel pushes content down, bottom panel pushes content up
    // Net effect: shift camera up by (bottom - top) / 2
    let vertical_offset = (BOTTOM_PANEL_HEIGHT - TOP_PANEL_HEIGHT) / 2.0;
    camera.translation.y = vertical_offset;
  }
}

/// System to toggle the debug panel and the file chooser via hotkeys.
/// - Escape toggles the debug panel (and resizes the window accordingly)
/// - F1 toggles the file chooser window
fn input_toggle_system(
  keyboard_input: Res<ButtonInput<KeyCode>>,
  mut panel_state: ResMut<UiPanelState>,
  mut file_chooser: ResMut<FileChooserState>,
  mut window_query: Query<&mut Window, With<PrimaryWindow>>,
  mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
  // Toggle debug panel (Escape)
  if keyboard_input.just_pressed(KeyCode::Escape) {
    panel_state.show_debug = !panel_state.show_debug;
    update_window_and_camera(panel_state.show_debug, &mut window_query, &mut camera_query);
  }

  // Toggle file chooser (F1)
  if keyboard_input.just_pressed(KeyCode::F1) {
    file_chooser.show = !file_chooser.show;
    file_chooser.error_message = None;
    file_chooser.selected_file = None;
    // default to ROM mode when toggling via hotkey; the top panel can open the
    // file chooser in a different mode explicitly.
    file_chooser.mode = FileChooserMode::Rom;
  }
}

/// System that updates window size and camera position when debug panel state changes
fn camera_update_system(
  panel_state: Res<UiPanelState>,
  mut window_query: Query<&mut Window, With<PrimaryWindow>>,
  mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
  if panel_state.is_changed() {
    update_window_and_camera(panel_state.show_debug, &mut window_query, &mut camera_query);
  }
}

/// Helper to update window size and camera position based on debug panel visibility
fn update_window_and_camera(
  show_debug: bool,
  window_query: &mut Query<&mut Window, With<PrimaryWindow>>,
  camera_query: &mut Query<&mut Transform, With<Camera2d>>,
) {
  let base_width = RESOLUTION.0 as f32;
  let base_height = RESOLUTION.1 as f32 + TOP_PANEL_HEIGHT + BOTTOM_PANEL_HEIGHT;

  // Vertical offset to center display between top and bottom panels
  let vertical_offset = (BOTTOM_PANEL_HEIGHT - TOP_PANEL_HEIGHT) / 2.0;

  if show_debug {
    // Expand window to include right debug panel
    if let Ok(mut window) = window_query.single_mut() {
      let new_width = (base_width + RIGHT_PANEL_WIDTH) as u32;
      let new_height = base_height as u32;
      window.resolution.set(new_width as f32, new_height as f32);
    }

    // Shift camera left so the display remains centered in the left portion
    // The right panel takes up RIGHT_PANEL_WIDTH, so the display area is shifted
    if let Ok(mut camera) = camera_query.single_mut() {
      camera.translation.x = RIGHT_PANEL_WIDTH / 2.0;
      camera.translation.y = vertical_offset;
    }
  } else {
    // Restore window to base size (no debug panel)
    if let Ok(mut window) = window_query.single_mut() {
      window.resolution.set(base_width, base_height);
    }

    // Center camera horizontally, keep vertical offset for top/bottom panels
    if let Ok(mut camera) = camera_query.single_mut() {
      camera.translation.x = 0.0;
      camera.translation.y = vertical_offset;
    }
  }
}

/// System that maps KeyCode presses/releases to the emulator's virtual keypad.
///
/// Mapping:
/// | 1 | 2 | 3 | C |  ->  1 2 3 4
/// | Q | W | E | R |  ->  Q W E R
/// | A | S | D | F |  ->  A S D F
/// | Z | X | C | V |  ->  Z X C V
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
