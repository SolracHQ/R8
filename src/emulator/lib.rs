use self::error::EmulatorError;

mod display;
pub mod error;
mod keyboard;
mod memory;
mod opcode;
mod rand;
mod stack;
#[cfg(test)]
mod tests;
pub mod emulator;
pub mod debug;

pub use display::{HEIGHT, WIDTH};

/// http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#2.2
/// Amount of V registers in the CHIP-8.
pub const REGISTER_COUNT: usize = 0x10;

// Helper Functions
fn bcd(value: u8) -> [u8; 3] {
    let hundreds = value / 100;
    let tens = (value % 100) / 10;
    let ones = value % 10;
    [hundreds, tens, ones]
}