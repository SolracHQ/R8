//! Terminal display helpers for the TUI frontend.
//!
//! This module implements an efficient renderer for the chip-8 framebuffer using a
//! virtual buffer. It avoids clearing and redrawing the entire screen by
//! calculating the difference between the previous and the current framebuffer
//! and only updating already changed pixels. It also checks the terminal size
//! and provides a friendly message when the terminal is too small to render
//! the chip-8 framebuffer.

use std::io::{self, Stdout, Write};

use crossterm::{
  cursor::{self, MoveTo},
  style::{self, Stylize},
  terminal::{self, Clear, ClearType},
  ExecutableCommand,
};
use r8_core::constants;

/// TUI renderer for the CHIP-8 framebuffer.
///
/// - Maintains a virtual buffer of the last rendered frame to avoid full redraws.
/// - Renders using 2 characters per CHIP-8 pixel horizontally (so each chip pixel
///   maps to a 2-column cell).
pub struct TerminalDisplay {
  stdout: Stdout,
  prev_vram: Vec<bool>, // flattened [x + y * WIDTH]
  pub min_cols: u16,    // minimum required terminal columns (WIDTH * 2)
  pub min_rows: u16,    // minimum required terminal rows (HEIGHT)
  small_warning_shown: bool,
}

impl TerminalDisplay {
  /// Create a new `TerminalDisplay` instance.
  ///
  /// This clears the terminal and hides the cursor (if possible).
  pub fn new() -> io::Result<Self> {
    let mut stdout = std::io::stdout();

    // Hide cursor to avoid annoying flicker
    stdout.execute(cursor::Hide)?;
    // Start with a clean screen
    stdout.execute(Clear(ClearType::All))?;

    let prev_vram = vec![false; constants::WIDTH * constants::HEIGHT];

    Ok(Self {
      stdout,
      prev_vram,
      min_cols: (constants::WIDTH * 2) as u16,
      min_rows: constants::HEIGHT as u16,
      small_warning_shown: false,
    })
  }

  /// Ensure the terminal size is large enough for rendering and shows an
  /// informative message if it is not.
  ///
  /// Returns `Ok(true)` if size is OK, `Ok(false)` if too small.
  fn ensure_size_ok(&mut self) -> std::io::Result<bool> {
    let (cols, rows) = terminal::size()?;
    if cols < self.min_cols || rows < self.min_rows {
      // Only draw the warning if it has not been shown or the size changed
      if !self.small_warning_shown {
        let msg = format!(
          "Terminal too small. Minimum columns: {} rows: {}. Current: {} x {}",
          self.min_cols, self.min_rows, cols, rows
        );
        // Clear and print the message centered.
        self.stdout.execute(Clear(ClearType::All))?;
        let x = cols.saturating_sub(msg.len() as u16) / 2;
        let y = rows / 2;
        self.stdout.execute(MoveTo(x, y))?;
        self.stdout.execute(style::Print(msg))?;
        // Move cursor to bottom-right to avoid jumping into the middle of the screen.
        self
          .stdout
          .execute(MoveTo(cols.saturating_sub(1), rows.saturating_sub(1)))?;
        self.small_warning_shown = true;
        let _ = self.stdout.flush();
      }
      Ok(false)
    } else {
      // When the terminal becomes big enough, clear the message and reset.
      if self.small_warning_shown {
        self.stdout.execute(Clear(ClearType::All))?;
        self.small_warning_shown = false;
        let _ = self.stdout.flush();
      }
      Ok(true)
    }
  }

  /// Public wrapper to check terminal size and show a message if needed.
  ///
  /// This is provided so upstream code (e.g. main) can check the terminal size
  /// independently of rendering behavior.
  pub fn check_size(&mut self) -> io::Result<bool> {
    self.ensure_size_ok()
  }

  /// Render the provided emulator framebuffer.
  ///
  /// The framebuffer is the emulator's 64x32 boolean array where `true` means
  /// a lit pixel.
  ///
  /// This method computes a list of changed pixels compared to the previous
  /// frame and updates only those cells in the terminal to minimize flicker.
  pub fn render(&mut self, vram: &[[bool; constants::HEIGHT]; constants::WIDTH]) -> io::Result<()> {
    // If the terminal is too small, show a message and skip rendering.
    let size_ok = self.ensure_size_ok()?;
    if !size_ok {
      return Ok(());
    }

    // We'll collect changed pixel coordinates first to avoid interleaved cursor positions
    // while generating the change list.
    let mut changes: Vec<(u16, u16, bool)> = Vec::with_capacity(128);

    for y in 0..constants::HEIGHT {
      for x in 0..constants::WIDTH {
        let idx = x + y * constants::WIDTH;
        let new_pixel = vram[x][y];
        let old_pixel = self.prev_vram[idx];
        if new_pixel != old_pixel {
          changes.push(((x as u16) * 2, y as u16, new_pixel));
        }
      }
    }

    // If nothing changed, just return.
    if changes.is_empty() {
      return Ok(());
    }

    // Now perform the minimum number of terminal writes to update the changed pixels.
    for (tx, ty, new_state) in changes.iter() {
      // Move cursor to that pixel
      self.stdout.execute(MoveTo(*tx, *ty))?;
      // Print the content for new or off content for false.
      if *new_state {
        // Filled pixel: print with blue foreground such that it looks like a block.
        // We prefer printing full block characters; the original main used "██".blue()
        self.stdout.execute(style::Print("██".blue()))?;
      } else {
        // Empty pixel: print two spaces which effectively clears the two-character cell.
        // Style the off pixel as black to keep visual consistency with the on pixel's
        // styled `blue()` content and avoid artifacting on some terminals.
        self.stdout.execute(style::Print("  ".black()))?;
      }
      // Update the internal state
      let idx = ((*tx as usize) / 2) + (*ty as usize) * constants::WIDTH;
      self.prev_vram[idx] = *new_state;
    }

    // Move cursor to the bottom-right corner to avoid disrupting user's input flow.
    let (cols, rows) = terminal::size()?;
    self
      .stdout
      .execute(MoveTo(cols.saturating_sub(1), rows.saturating_sub(1)))?;
    let _ = self.stdout.flush();
    Ok(())
  }
}

impl Drop for TerminalDisplay {
  fn drop(&mut self) {
    // Try to restore cursor visibility. This is best-effort because destructors should
    // not panic. Ignore any errors.
    let _ = self.stdout.execute(cursor::Show);
    let _ = self.stdout.execute(Clear(ClearType::All));
    let _ = self.stdout.flush();
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Simple integration smoke test to verify we can create the display
  /// and call `render` without panicking. It uses a fake emulator with a few
  /// pixels set. This does not actually assert on terminal output — it's just
  /// a functional test.
  #[test]
  fn display_render_smoke() {
    let mut td = TerminalDisplay::new().expect("Failed to create TerminalDisplay");

    // Prepare a small test pattern (one pixel at (0,0) and one at (1,1))
    let mut test_vram = [[false; constants::HEIGHT]; constants::WIDTH];
    test_vram[0][0] = true;
    test_vram[1][1] = true;

    // Even if terminal is small (e.g. in CI), render should return without panicking.
    let result = td.render(&test_vram);
    assert!(result.is_ok());
  }
}
