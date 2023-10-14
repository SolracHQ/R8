use crate::{
    emulator::{Emulator, State},
    memory::Address,
    stack::Stack,
    REGISTER_COUNT,
};

// Impl getters for debugging
impl Emulator {
    /// Returns the current program counter
    pub fn pc(&self) -> Address {
        self.pc
    }

    /// Returns the current index register
    pub fn i(&self) -> Address {
        self.i
    }
    /// Returns the current value of the register
    pub fn v_registers(&self) -> &[u8; REGISTER_COUNT] {
        &self.v_registers
    }

    /// Returns the current value of the sound register
    pub fn sound_timer(&self) -> u8 {
        self.sound_timer
    }

    /// Returns the current value of the delay register
    pub fn delay_timer(&self) -> u8 {
        self.delay_timer
    }
    /// Returns an inmutable reference to the stack
    pub fn stack(&self) -> &Stack<Address> {
        &self.stack
    }
    /// Return the current state of the emulator
    pub fn state(&self) -> &State {
        &self.state
    }
}
