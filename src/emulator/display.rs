/// Width of the display.
pub const WIDTH: usize = 64;
/// Height of the display.
pub const HEIGHT: usize = 32;

/// Represents the display of the Chip8 system.
pub struct Display {
    /// The video RAM of the display.
    vram: [[bool; HEIGHT]; WIDTH],
    /// Indicates whether the display has been updated.
    pub updated: bool,
}

impl Display {
    /// Creates a new display.
    pub(super) fn new() -> Self {
        Self {
            vram: [[false; HEIGHT]; WIDTH],
            updated: false,
        }
    }

    /// Clears the display.
    pub(super) fn clear(&mut self) {
        self.updated = true;
        self.vram = [[false; HEIGHT]; WIDTH];
    }

    /// Sets 8 pixels on the display.
    ///
    /// # Arguments
    ///
    /// * `x` - The x-coordinate of the pixel.
    /// * `y` - The y-coordinate of the pixel.
    /// * `value` - The value to set the pixels to, represented as 8 bit-encoded pixels.
    ///
    /// # Returns
    ///
    /// * `u8` - Returns 1 if a pixel was erased, otherwise returns 0.
    pub fn set(&mut self, x: u8, mut y: u8, value: u8) -> u8 {
        self.updated = true;
        let mut result = 0;
        y = y % HEIGHT as u8;
        let y_usize = y as usize;
        for bit_index in 0..u8::BITS as u8 {
            let x_usize = (x + bit_index) as usize % WIDTH;
            let pixel = (value & (0x80 >> bit_index)) != 0;
            if !(self.vram[x_usize][y_usize] ^ pixel) && !pixel {
                result = 1
            }
            self.vram[x_usize][y_usize] ^= pixel;
        }
        result
    }

    pub fn get(&self, x: usize, y: usize) -> bool {
        self.vram[x][y]
    }
}
