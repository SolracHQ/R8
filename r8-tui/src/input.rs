//! Input handling for the TUI frontend.
//!
//! This module contains helpers to map terminal keys to emulator keys and to process
//! crossterm key events coming from the terminal. The `map_key` function mirrors
//! the mapping used by the original `main.rs`. The `process_key_event` is a
//! helper to decouple the event handling from the rest of the application.

use crossterm::event::{Event, KeyCode, KeyEvent};
use r8_emulator::{Emulator, Key as EmuKey};

/// Map a char to an emulator Key.
///
/// Chip-8 original layout:
///
/// | 1 | 2 | 3 | C |
/// |---|---|---|---|
/// | 4 | 5 | 6 | D |
/// | 7 | 8 | 9 | E |
/// | A | 0 | B | F |
///
/// TUI mapping:
///
/// | 1 | 2 | 3 | 4 |  ->  1 2 3 4
/// | Q | W | E | R |  ->  Q W E R
/// | A | S | D | F |
/// | Z | X | C | V |
///
/// Returns `Some(Key)` if the char corresponds to a keypad button, otherwise `None`.
pub fn map_key(key: char) -> Option<EmuKey> {
  match key {
    '1' => Some(EmuKey::K1),
    '2' => Some(EmuKey::K2),
    '3' => Some(EmuKey::K3),
    '4' => Some(EmuKey::KC),
    'Q' | 'q' => Some(EmuKey::K4),
    'W' | 'w' => Some(EmuKey::K5),
    'E' | 'e' => Some(EmuKey::K6),
    'R' | 'r' => Some(EmuKey::KD),
    'A' | 'a' => Some(EmuKey::K7),
    'S' | 's' => Some(EmuKey::K8),
    'D' | 'd' => Some(EmuKey::K9),
    'F' | 'f' => Some(EmuKey::KE),
    'Z' | 'z' => Some(EmuKey::KA),
    'X' | 'x' => Some(EmuKey::K0),
    'C' | 'c' => Some(EmuKey::KB),
    'V' | 'v' => Some(EmuKey::KF),
    _ => None,
  }
}

/// Process a `crossterm::event::Event`.
///
/// Returns `true` if the event should cause the TUI to exit (e.g. `Esc` key),
/// otherwise `false`.
///
/// Handles only `Event::Key` events and ignores other event kinds.
pub fn process_event(event: Event, emu: &mut Emulator) -> bool {
  match event {
    Event::Key(KeyEvent { code, .. }) => match code {
      KeyCode::Esc => true,
      KeyCode::Char(ch) => {
        if let Some(k) = map_key(ch) {
          emu.press_key(k);
        }
        false
      }
      _ => false,
    },
    _ => false,
  }
}

/// Release all emulator keys for the current frame.
///
/// The TUI clears all keys on every frame (because TUI limitations only allow
/// key press detection, not releases). This helper simplifies the main loop.
pub fn release_all_keys(emu: &mut Emulator) {
  EmuKey::all().for_each(|k| emu.release_key(*k));
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn map_key_lowercase() {
    assert_eq!(map_key('q'), Some(EmuKey::K4));
    assert_eq!(map_key('x'), Some(EmuKey::K0));
    assert_eq!(map_key('z'), Some(EmuKey::KA));
  }

  #[test]
  fn map_key_uppercase() {
    assert_eq!(map_key('Q'), Some(EmuKey::K4));
    assert_eq!(map_key('X'), Some(EmuKey::K0));
    assert_eq!(map_key('Z'), Some(EmuKey::KA));
  }

  #[test]
  fn map_key_invalid() {
    assert_eq!(map_key('g'), None);
    assert_eq!(map_key('\n'), None);
  }
}
