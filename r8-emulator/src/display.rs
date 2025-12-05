//! Display device for the R8 emulator crate.
//!
//! This module depends on the core `constants` in the `r8_core` crate.

use r8_core::constants;

/// Represents the display of the Chip8 system.
/// The display is a 64x32 monochrome display.
///
/// # Fields
///
/// * `vram` - A 2D array of booleans representing the video RAM of the display.
/// * `updated` - Indicates whether the display has been updated (to avoid redrawing when there are no changes).
pub struct Display {
  /// The video RAM of the display.
  vram: [[bool; constants::HEIGHT]; constants::WIDTH],
  /// Indicates whether the display has been updated.
  pub updated: bool,
}

impl Display {
  /// Creates a new display with all pixels set to false and `updated` set to false.
  pub(super) fn new() -> Self {
    Self {
      vram: [[false; constants::HEIGHT]; constants::WIDTH],
      updated: false,
    }
  }

  /// Clears the display by setting all pixels to false and marking it as updated.
  pub(super) fn clear(&mut self) {
    self.updated = true;
    self.vram = [[false; constants::HEIGHT]; constants::WIDTH];
  }

  /// Sets 8 pixels on the display encoded as a single byte.
  ///
  /// # Arguments
  ///
  /// * `x` - The x-coordinate of the pixel (leftmost bit).
  /// * `y` - The y-coordinate of the pixel (top).
  /// * `value` - 8-bit encoded pixels, MSB is the left-most.
  ///
  /// # Returns
  ///
  /// `u8` - 1 if a pixel was erased (collision), otherwise 0.
  pub fn set(&mut self, x: u8, mut y: u8, value: u8) -> u8 {
    self.updated = true;
    let mut result = 0;
    y %= constants::HEIGHT as u8;
    let y_usize = y as usize;

    for bit_index in 0..u8::BITS as u8 {
      let x_usize = ((x + bit_index) as usize) % constants::WIDTH;
      let pixel = (value & (0x80 >> bit_index)) != 0;
      if !(self.vram[x_usize][y_usize] ^ pixel) && !pixel {
        result = 1;
      }
      self.vram[x_usize][y_usize] ^= pixel;
    }

    result
  }

  /// Returns the value of a pixel at the specified coordinates.
  pub fn get(&self, x: usize, y: usize) -> bool {
    self.vram[x][y]
  }

  /// Returns a reference to the video RAM of the display.
  ///
  /// Useful for front-ends that want to render the vram directly.
  pub fn get_vram(&self) -> &[[bool; constants::HEIGHT]; constants::WIDTH] {
    &self.vram
  }
}

impl std::ops::Index<(usize, usize)> for Display {
  type Output = bool;

  /// Index the display to obtain a pixel value by (x, y).
  fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
    &self.vram[x][y]
  }
}
