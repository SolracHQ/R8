#![allow(non_snake_case)]
use std::io::Read;

use self::{error::EmulatorError, memory::Address};
use log::{debug, error};

mod display;
pub mod error;
mod keyboard;
mod memory;
mod opcode;
mod rand;
mod stack;
#[cfg(test)]
mod tests;

pub use display::{HEIGHT, WIDTH};

/// http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#2.2
/// Amount of V registers in the CHIP-8.
pub const REGISTER_COUNT: usize = 0x10;

/// The index of the flags register in the V registers.
const FLAGS_REGISTER: usize = 0xF;

/// Represents the state of the emulator.
#[derive(Debug)]
pub enum State {
    New,
    Running,
    WaitingKey { x: u8 },
}

/// The `Emulator` struct represents the CHIP-8 emulator.
///
/// # Fields
///
/// * `pc` - The program counter.
/// * `i` - The index register.
/// * `v_registers` - The V registers.
/// * `sound_timer` - The sound timer.
/// * `delay_timer` - The delay timer.
/// * `stack` - The stack.
/// * `memory` - The memory.
/// * `display` - The display.
/// * `keyboard` - The keyboard.
/// * `rand` - The random number generator.
/// * `state` - The state of the emulator.
pub struct Emulator {
    // Registers
    pc: memory::Address,
    i: memory::Address,
    v_registers: [u8; REGISTER_COUNT],
    sound_timer: u8,
    delay_timer: u8,
    // Memory Segments
    stack: stack::Stack<Address>,
    memory: memory::Memory,
    // Devices
    pub display: display::Display,
    pub keyboard: keyboard::KeyBoard,
    // Helper Structs
    rand: rand::RandGen,
    state: State,
}

impl Emulator {
    /// Creates a new `Emulator` on state `New`.
    ///
    /// # Returns
    ///
    /// * `Emulator` - The newly created emulator.
    pub fn new() -> Self {
        Self {
            pc: memory::Address::ENTRY_POINT,
            i: memory::Address::new(0),
            v_registers: [0; REGISTER_COUNT],
            sound_timer: 0,
            delay_timer: 0,
            stack: stack::Stack::new(),
            memory: memory::Memory::new(),
            display: display::Display::new(),
            keyboard: keyboard::KeyBoard::default(),
            rand: rand::RandGen::new(),
            state: State::New,
        }
    }

    /// Loads a ROM into the emulator.
    ///
    /// # Arguments
    ///
    /// * `reader` - The reader to read the ROM from.
    ///
    /// # Returns
    ///
    /// * `Result<(), RuntimeError>` - The result of the operation.
    ///
    /// # Notes
    ///
    /// * The emulator is reset to its initial state.
    pub fn load_rom<R: Read>(&mut self, reader: R) -> Result<(), error::EmulatorError> {
        self.pc = memory::Address::ENTRY_POINT;
        self.i = memory::Address::new(0);
        self.delay_timer = 0;
        self.sound_timer = 0;
        self.v_registers = [0; REGISTER_COUNT];
        self.stack.clear();
        self.display.clear();
        self.memory.load_rom(reader)?;
        self.state = State::Running;
        Ok(())
    }

    /// Executes a single tick of the emulator.
    ///
    /// # Returns
    ///
    /// * `Result<(), RuntimeError>` - The result of the operation.
    ///
    /// # Notes
    ///
    /// * If the emulator is in the `State::New` state, this function does nothing.
    /// * If the emulator is in the `State::WaitingKey` state and the keyboard is not pressed, this function does nothing.
    /// * If the emulator is in the `State::WaitingKey` state and the keyboard is pressed, the state is changed to `State::Running`.
    pub fn tick(&mut self) -> Result<(), EmulatorError> {
        // Helper Macros
        macro_rules! jump_if {
            ($op:tt, $x:expr, $y:expr) => {
                if $x $op $y { self.pc.add_assign(2)?; }
            };
        }

        macro_rules! V {
            ($reg: expr) => {
                self.v_registers[$reg as usize]
            };
        }

        if let State::New = self.state {
            return Ok(());
        }

        if let State::WaitingKey { x } = self.state {
            match (0..=0xF).find(|key| self.keyboard.is_set(*key)) {
                Some(key) => {
                    V![x] = key;
                    self.state = State::Running
                }
                None => return Ok(()),
            }
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        let opcode = self.opcode()?;

        debug!("| 0x{PC:X} | {opcode}", PC = self.pc.inner());
        let nibbles = opcode.nibbles();
        self.pc.add_assign(2)?;

        match nibbles {
            (0, 0, 0xE, 0) => {
                // Clears the screen.
                self.display.clear();
            }
            (0, 0, 0xE, 0xE) => {
                // Returns from a subroutine.
                self.pc = self.stack.pop()?
            }
            (0x1, _, _, _) => {
                // Jumps to address NNN.
                self.pc = opcode.nnn()
            }
            (0x2, _, _, _) | (0, _, _, _) => {
                // Calls subroutine at Address.
                self.stack.push(self.pc)?;
                self.pc = opcode.nnn();
            }
            (0x3, x, _, _) => {
                // Skips the next instruction if VX equals opcode lower 8-bits
                jump_if!(==, V![x], opcode.kk_byte())
            }
            (0x4, x, _, _) => {
                // Skips the next instruction if VX does not equal opcode lower 8-bits
                jump_if!(!=, V![x], opcode.kk_byte())
            }
            (0x5, x, y, 0) => {
                // Skips the next instruction if VX equals VY
                jump_if!(==, V![x], V![y])
            }
            (0x6, x, _, _) => {
                // Sets VX to NN.
                V![x] = opcode.kk_byte()
            }
            (0x7, x, _, _) => {
                // Adds NN to VX (carry flag is not changed).
                V![x] = V![x].wrapping_add(opcode.kk_byte())
            }
            (0x8, x, y, 0x0) => {
                // Sets VX to the value of VY.
                V![x] = V![y]
            }
            (0x8, x, y, 0x1) => {
                // Sets VX to VX or VY. (bitwise OR operation)
                V![x] |= V![y]
            }
            (0x8, x, y, 0x2) => {
                // Sets VX to VX and VY. (bitwise AND operation)
                V![x] &= V![y]
            }
            (0x8, x, y, 0x3) => {
                // Sets VX to VX xor VY.
                V![x] ^= V![y]
            }
            (0x8, x, y, 0x4) => {
                // The values of Vx and Vy are added together. If the result is greater than 8 bits (i.e., > 255,)
                // VF is set to 1, otherwise 0. Only the lowest 8 bits of the result are kept, and stored in Vx.
                let result = V![x] as u16 + V![y] as u16;
                V![x] = (result & 0xFF) as u8;
                V![FLAGS_REGISTER] = if result & 0xFF00 != 0 { 1 } else { 0 }
            }
            (0x8, x, y, 0x5) => {
                // Set Vx = Vx - Vy, set VF = NOT borrow.
                // If Vx > Vy, then VF is set to 1, otherwise 0. Then Vy is subtracted from Vx, and the results stored in Vx.
                V![FLAGS_REGISTER] = if V![x] > V![y] { 1 } else { 0 };
                V![x] = V![x].wrapping_sub(V![y]);
            }
            (0x8, x, _, 0x6) => {
                // Set Vx = Vx SHR 1.
                // If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0. Then Vx is divided by 2.
                V![FLAGS_REGISTER] = V![x] & 1;
                V![x] >>= 1;
            }
            (0x8, x, y, 0x7) => {
                // Set Vx = Vy - Vx, set VF = NOT borrow.
                // If Vy > Vx, then VF is set to 1, otherwise 0. Then Vx is subtracted from Vy, and the results stored in Vx.
                V![FLAGS_REGISTER] = if V![y] > V![x] { 1 } else { 0 };
                V![x] = V![y].wrapping_sub(V![x]);
            }
            (0x8, x, _, 0xE) => {
                // Set Vx = Vx SHL 1.
                // If the most-significant bit of Vx is 1, then VF is set to 1, otherwise 0. Then Vx is divided by 2.
                V![FLAGS_REGISTER] = (V![x] >> 7) & 1;
                V![x] <<= 1;
            }
            (0x9, x, y, 0) => {
                // Skip next instruction if Vx != Vy.
                // The values of Vx and Vy are compared, and if they are not equal, the program counter is increased by 2.
                if V![x] != V![y] {
                    self.pc.add_assign(2)?;
                }
            }
            (0xA, _, _, _) => {
                // Set I = nnn.
                self.i = opcode.nnn()
            }
            (0xB, _, _, _) => {
                // Jump to location nnn + V0.
                self.pc.add_assign(opcode.nnn().inner() + V![0] as u16)?
            }
            (0xC, x, _, _) => {
                // Set Vx = random byte AND kk. The interpreter generates a random number from 0 to 255, which is then ANDed with the value kk.
                V![x] = self.rand.next() & opcode.kk_byte()
            }
            (0xD, x, y, n) => self.display_n_rows(x, y, n)?,
            (0xE, x, 0x9, 0xE) => {
                // Skip next instruction if key with the value of Vx is pressed.
                if self.keyboard.is_set(V![x]) {
                    self.pc.add_assign(2)?;
                }
            }
            (0xE, x, 0xA, 0x1) => {
                // Skip next instruction if key with the value of Vx is not pressed.
                if !self.keyboard.is_set(V![x]) {
                    self.pc.add_assign(2)?;
                }
            }
            (0xF, x, 0x0, 0x7) => {
                // Set Vx = delay timer value.
                V![x] = self.delay_timer
            }
            (0xF, x, 0x0, 0xA) => {
                // Wait for a key press, store the value of the key in Vx.
                self.state = State::WaitingKey { x }
            }
            (0xF, x, 0x1, 0x5) => {
                // Set delay timer = Vx.
                self.delay_timer = V![x]
            }
            (0xF, x, 0x1, 0x8) => {
                // Set sound timer = Vx.
                self.sound_timer = V![x]
            }
            (0xF, x, 0x1, 0xE) => {
                // Set I = I + Vx.
                self.i.add_assign(V![x] as u16)?
            }
            (0xF, x, 0x2, 0x9) => {
                // Set I = location of sprite for digit Vx.
                self.i = Address::new(V![x] as u16 * 5)
            }
            (0xF, x, 0x3, 0x3) => {
                // Store BCD representation of Vx in memory locations I, I+1, and I+2.
                self.memory
                    .read_range(self.i, &BCD(self.v_registers[x as usize]))?
            }
            (0xF, x, 0x5, 0x5) => {
                // Store registers V0 through Vx in memory starting at location I.
                self.memory
                    .read_range(self.i, &self.v_registers[0..=x as _])?
            }
            (0xF, x, 0x6, 0x5) => {
                // Read registers V0 through Vx from memory starting at location I.
                self.memory
                    .write_range(self.i, &mut self.v_registers[0..=x as _])?
            }
            _ => error!(
                "Unrecognized OpCode: | 0x{PC:X} | {:X?}",
                opcode.nibbles(),
                PC = self.pc.inner()
            ),
        }
        Ok(())
    }

    pub fn opcode(&self) -> Result<opcode::Opcode, EmulatorError> {
        let mut opcode = [0, 0];
        self.memory.write_range(self.pc, &mut opcode)?;
        Ok(opcode::Opcode::new(opcode))
    }

    /**
     * Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
     * The interpreter reads n bytes from memory, starting at the address stored in I.
     * Sprites are XORed onto the existing screen. If this causes any pixels to be erased, VF is set to 1,
     * otherwise it is set to 0. If the sprite is positioned so part of it is outside the coordinates of the display,
     * it wraps around to the opposite side of the screen. See instruction 8xy3 for more information on XOR, and section 2.4, Display, for more information on the Chip-8 screen and sprites.
     */
    fn display_n_rows(&mut self, x: u8, y: u8, n: u8) -> Result<(), EmulatorError> {
        self.v_registers[FLAGS_REGISTER] = 0;
        let (x, y) = (self.v_registers[x as usize], self.v_registers[y as usize]);
        for row in 0..n {
            self.v_registers[FLAGS_REGISTER] |= self.display.set(
                x,
                y % display::HEIGHT as u8 + row,
                self.memory[(self.i.inner() + row as u16).try_into()?],
            )
        }
        Ok(())
    }
}

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
    pub fn stack(&self) -> &stack::Stack<Address> {
        &self.stack
    }
    /// Return the current state of the emulator
    pub fn state(&self) -> &State {
        &self.state
    }
}

// Helper Functions
fn BCD(value: u8) -> [u8; 3] {
    let hundreds = value / 100;
    let tens = (value % 100) / 10;
    let ones = value % 10;
    [hundreds, tens, ones]
}
