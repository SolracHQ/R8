use self::error::EmulatorError;

/// Module to centralize all the errors that can occur in the emulator.
pub mod error;

pub mod constants;

mod display;
mod keyboard;
mod memory;
mod opcode;
mod rand;
mod stack;
mod register;
mod timer;
pub mod emulator;

pub mod debug;

pub mod assembler;

#[cfg(test)]
mod tests;