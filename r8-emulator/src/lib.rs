/*!
r8-emulator

This crate implements the CHIP-8 emulator runtime and device modules (display, keyboard, etc).
It focuses on the runtime and exposes the `Emulator` type as the primary entrypoint for other
frontends (GUI, TUI, tests, etc).

Core types and low-level utilities (Address, Memory, Opcode, Register types, Stack, Timers, and
errors) are placed in the separate `r8-core` crate. This crate depends on and uses those types
via `r8_core`.
*/

// Public modules that belong to this crate. Keep these modules focused on the runtime and devices.
pub mod debug;
pub mod display;
pub mod emulator;
pub mod keyboard;

/// Re-export the main emulator type so downstream crates can import it directly:
///
/// use r8_emulator::Emulator;
pub use emulator::Emulator;

/// Re-export the common keyboard types so frontends can map or forward inputs easily.
pub use keyboard::{Key, KeyBoard};

/// Optionally re-export the public display type to be used by frontends that need direct access.
pub use display::Display;

#[cfg(test)]
mod tests;
