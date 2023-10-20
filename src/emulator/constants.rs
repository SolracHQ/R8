/// Amount of V registers in the CHIP-8.
/// 
/// http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#2.2
pub const REGISTER_COUNT: usize = 0x10;

/// Width of the display.
pub const WIDTH: usize = 64;

/// Height of the display.
pub const HEIGHT: usize = 32;

/// http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#2.2
/// The chip-8 stack size is traditionally 16 (`0x10`).
pub const STACK_SIZE: usize = 0x10;