use self::error::EmulatorError;

mod display;
pub mod error;
mod keyboard;
mod memory;
mod opcode;
mod rand;
mod stack;
mod register;
mod timer;
#[cfg(test)]
mod tests;
pub mod emulator;
pub mod debug;

pub mod assembler;

pub use display::{HEIGHT, WIDTH};

/// Amount of V registers in the CHIP-8.
/// 
/// http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#2.2
pub const REGISTER_COUNT: usize = 0x10;

/// Translate a number to BCD.
/// 
/// # Arguments
/// 
/// * `value` - The value to translate.
/// 
/// # Returns
/// 
/// * `[u8; 3]` - The BCD representation of the value.
fn bcd(value: u8) -> [u8; 3] {
    let hundreds = value / 100;
    let tens = (value % 100) / 10;
    let ones = value % 10;
    [hundreds, tens, ones]
}