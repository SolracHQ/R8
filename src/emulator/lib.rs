use self::error::EmulatorError;

/// Module to centralize all the errors that can occur in the emulator.
pub mod error;

pub mod constants;

mod display;
pub mod emulator;
pub mod keyboard;
mod memory;
mod opcode;
mod rand;
mod register;
mod stack;
mod timer;

pub mod debug;

pub mod assembler;

#[cfg(test)]
mod tests;
