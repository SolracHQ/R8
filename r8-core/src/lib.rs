//! r8-core
//!
//! Core types and utilities for the R8 project.
//!
//! This crate contains the fundamental types (addresses, memory, opcodes, registers,
//! timers, stack), errors and small utilities that are shared between the
//! various components of this project (emulator, assembler, GUI, TUI).

// Public modules
pub mod constants;
pub mod error;
pub mod memory;
pub mod opcode;
pub mod rand;
pub mod register;
pub mod stack;
pub mod timer;

// Re-export commonly used types for ergonomic imports by downstream crates.
pub use error::EmulatorError;
pub use memory::{Address, Memory};
pub use opcode::Opcode;
pub use rand::RandGen;
pub use register::{RegisterIndex, VRegisters};
pub use stack::Stack;
pub use timer::Timer;
