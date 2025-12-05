use bevy::prelude::*;

use super::right_panel::MemoryInspectorState;

/// Shared state for UI panels visibility and configuration
#[derive(Resource)]
pub struct UiPanelState {
  /// Whether to show the debug/right panel
  pub show_debug: bool,
  /// Memory inspector state (persisted even when panel is hidden)
  pub memory_inspector: MemoryInspectorState,
}

impl Default for UiPanelState {
  fn default() -> Self {
    Self {
      show_debug: false,
      memory_inspector: MemoryInspectorState::new(),
    }
  }
}
