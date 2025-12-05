use crate::emulator::{Emulator, State};
use r8_core::{Address, EmulatorError, Stack, VRegisters};

/// Impl getters for debugging
impl Emulator {
  /// Returns the current program counter
  pub fn pc(&self) -> Address {
    self.pc
  }

  /// Returns the current index register
  pub fn i(&self) -> Address {
    self.i
  }

  /// Returns the current value of the V registers
  pub fn v_registers(&self) -> &VRegisters {
    &self.registers
  }

  /// Returns the current value of the sound timer
  pub fn sound_timer(&self) -> u8 {
    self.sound_timer.get()
  }

  /// Returns the current value of the delay timer
  pub fn delay_timer(&self) -> u8 {
    self.delay_timer.get()
  }

  /// Returns an immutable reference to the stack
  pub fn stack(&self) -> &Stack<Address> {
    &self.stack
  }

  /// Return the current state of the emulator
  pub fn state(&self) -> &State {
    &self.state
  }

  /// Read memory at the given address into the buffer (for debug/memory inspector)
  pub fn read_memory(&self, address: Address, buffer: &mut [u8]) -> Result<(), EmulatorError> {
    self.memory.write_range(address, buffer)
  }
}
