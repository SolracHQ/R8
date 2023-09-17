#![allow(non_snake_case)]
use std::io::Read;

use self::error::RuntimeError;
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
const REGISTER_COUNT: usize = 0x10;

/// Only lower 12-bits will be used
type Address = u16;
/// http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#memmap
const ENTRY_POINT: Address = 0x200;

const FLAGS_REGISTER: usize = 0xF;

pub enum State {
    New,
    Running,
    WaitingKey { x: u8 },
}

pub struct Emulator {
    // Registers
    PC: Address,
    I: Address,
    V: [u8; REGISTER_COUNT],
    sound_timer: u8,
    delay_timer: u8,
    // Memory Segments
    stack: stack::Stack,
    memory: memory::Memory,
    // Devices
    pub display: display::Display,
    pub keyboard: keyboard::KeyBoard,
    // Helper Structs
    rand: rand::RandGen,
    state: State,
}

impl Emulator {
    pub fn new() -> Self {
        Self {
            PC: ENTRY_POINT,
            I: 0,
            V: [0; REGISTER_COUNT],
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

    pub fn load_rom<R: Read>(&mut self, reader: R) -> Result<(), error::RuntimeError> {
        self.PC = ENTRY_POINT;
        self.I = 0;
        self.delay_timer = 0;
        self.sound_timer = 0;
        self.V = [0; REGISTER_COUNT];
        self.stack.clear();
        self.display.clear();
        self.memory
            .load_rom(reader)
            .map_err(error::RuntimeError::LoadError)?;
        self.state = State::Running;
        Ok(())
    }

    pub fn tick(&mut self) -> Result<(), RuntimeError> {
        if let State::New = self.state {
            return Ok(());
        }

        if let State::WaitingKey { x } = self.state {
            match (0..=0xF).find(|key| self.keyboard.is_set(*key)) {
                Some(key) => {
                    self.V[x as usize] = key;
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

        let opcode = opcode::Opcode::new(self.memory[self.PC], self.memory[self.PC + 1]);
        debug!("{opcode}");
        let nibbles = opcode.nibbles();
        self.PC += 2;
        match nibbles {
            // Clears the screen.
            (0, 0, 0xE, 0) => {
                self.display.clear();
            }
            // Returns from a subroutine.
            (0, 0, 0xE, 0xE) => self.PC = self.stack.pop()?,
            // Calls machine code routine (RCA 1802 for COSMAC VIP) at address NNN.
            // Not necessary for most ROMs.
            (0, _, _, _) => {
                self.stack.push(self.PC)?;
                self.PC = opcode.address();
            }
            // Jumps to address NNN.
            (0x1, _, _, _) => self.PC = opcode.address(),
            // Calls subroutine at NNN.
            (0x2, _, _, _) => {
                self.stack.push(self.PC)?;
                self.PC = opcode.address();
            }
            // Skips the next instruction if VX equals opcode lower 8-bits
            (0x3, x, _, _) => {
                if self.V[x as usize] == opcode.kk_byte() {
                    self.PC += 2
                }
            }
            // Skips the next instruction if VX does not equal opcode lower 8-bits
            (0x4, x, _, _) => {
                if self.V[x as usize] != opcode.kk_byte() {
                    self.PC += 2
                }
            }
            // Skips the next instruction if VX equals VY
            (0x5, x, y, 0) => {
                if self.V[x as usize] == self.V[y as usize] {
                    self.PC += 2
                }
            }
            // Sets VX to NN.
            (0x6, x, _, _) => self.V[x as usize] = opcode.kk_byte(),
            // Adds NN to VX (carry flag is not changed).
            (0x7, x, _, _) => {
                self.V[x as usize] = self.V[x as usize].wrapping_add(opcode.kk_byte())
            }
            // Sets VX to the value of VY.
            (0x8, x, y, 0x0) => self.V[x as usize] = self.V[y as usize],
            // Sets VX to VX or VY. (bitwise OR operation)
            (0x8, x, y, 0x1) => self.V[x as usize] |= self.V[y as usize],
            // Sets VX to VX and VY. (bitwise AND operation)
            (0x8, x, y, 0x2) => self.V[x as usize] &= self.V[y as usize],
            // Sets VX to VX xor VY.
            (0x8, x, y, 0x3) => self.V[x as usize] ^= self.V[y as usize],
            // The values of Vx and Vy are added together. If the result is greater than 8 bits (i.e., > 255,)
            // VF is set to 1, otherwise 0. Only the lowest 8 bits of the result are kept, and stored in Vx.
            (0x8, x, y, 0x4) => {
                let (tmp, overflow) = self.V[x as usize].overflowing_add(self.V[y as usize]);
                self.V[FLAGS_REGISTER] = if overflow { 1 } else { 0 };
                self.V[x as usize] = tmp;
            }
            // Set Vx = Vx - Vy, set VF = NOT borrow.
            // If Vx > Vy, then VF is set to 1, otherwise 0. Then Vy is subtracted from Vx, and the results stored in Vx.
            (0x8, x, y, 0x5) => {
                let (tmp, overflow) = self.V[x as usize].overflowing_sub(self.V[y as usize]);
                self.V[FLAGS_REGISTER] = if !overflow { 1 } else { 0 };
                self.V[x as usize] = tmp;
            }
            // Set Vx = Vx SHR 1.
            // If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0. Then Vx is divided by 2.
            (0x8, x, _, 0x6) => {
                self.V[FLAGS_REGISTER] = self.V[x as usize] & 1;
                self.V[x as usize] >>= 1;
            }
            // Set Vx = Vy - Vx, set VF = NOT borrow.
            // If Vy > Vx, then VF is set to 1, otherwise 0. Then Vx is subtracted from Vy, and the results stored in Vx.
            (0x8, x, y, 0x7) => {
                let (tmp, overflow) = self.V[y as usize].overflowing_sub(self.V[x as usize]);
                self.V[FLAGS_REGISTER] = if !overflow { 1 } else { 0 };
                self.V[x as usize] = tmp;
            }
            // Set Vx = Vx SHL 1.
            // If the most-significant bit of Vx is 1, then VF is set to 1, otherwise 0. Then Vx is divided by 2.
            (0x8, x, _, 0xE) => {
                self.V[FLAGS_REGISTER] = (self.V[x as usize] & 0b10000000) >> 7;
                self.V[x as usize] <<= 1;
            }
            // Skip next instruction if Vx != Vy.
            // The values of Vx and Vy are compared, and if they are not equal, the program counter is increased by 2.
            (0x9, x, y, 0) if self.V[x as usize] != self.V[y as usize] => self.PC += 2,
            // Set I = nnn.
            // The value of register I is set to nnn.
            (0xA, _, _, _) => self.I = opcode.address(),
            // Jump to location nnn + V0.
            // The program counter is set to nnn plus the value of V0.
            (0xB, _, _, _) => self.PC = opcode.address() + self.V[0] as u16,
            // Set Vx = random byte AND kk.
            // The interpreter generates a random number from 0 to 255, which is then ANDed with the value kk. The results are stored in Vx. See instruction 8xy2 for more information on AND.
            (0xC, x, _, _) => self.V[x as usize] = self.rand.next() & opcode.kk_byte(),
            (0xD, x, y, n) => self.display_n_rows(x, y, n),
            // Skip next instruction if key with the value of Vx is pressed.
            // Checks the keyboard, and if the key corresponding to the value of Vx is currently in the down position, PC is increased by 2.
            (0xE, x, 0x9, 0xE) => {
                if self.keyboard.is_set(self.V[x as usize]) {
                    self.PC += 2
                }
            }
            // Skip next instruction if key with the value of Vx is not pressed.
            // Checks the keyboard, and if the key corresponding to the value of Vx is currently in the up position, PC is increased by 2.
            (0xE, x, 0xA, 0x1) => {
                if !self.keyboard.is_set(self.V[x as usize]) {
                    self.PC += 2
                }
            }
            // Set Vx = delay timer value.
            // The value of DT is placed into Vx.
            (0xF, x, 0x0, 0x7) => self.V[x as usize] = self.delay_timer,
            // Wait for a key press, store the value of the key in Vx.
            // All execution stops until a key is pressed, then the value of that key is stored in Vx.
            (0xF, x, 0x0, 0xA) => self.state = State::WaitingKey { x },
            // Set delay timer = Vx.
            // DT is set equal to the value of Vx.
            (0xF, x, 0x1, 0x5) => self.delay_timer = self.V[x as usize],
            // Set sound timer = Vx.
            // ST is set equal to the value of Vx.
            (0xF, x, 0x1, 0x8) => self.sound_timer = self.V[x as usize],
            // Set I = I + Vx.
            // The values of I and Vx are added, and the results are stored in I.
            (0xF, x, 0x1, 0xE) => self.I += self.V[x as usize] as Address,
            // Set I = location of sprite for digit Vx.
            (0xF, x, 0x2, 0x9) => self.I = self.V[x as usize] as Address * 5,
            // Store BCD representation of Vx in memory locations I, I+1, and I+2.
            (0xF, x, 0x3, 0x3) => {
                self.memory[self.I] = self.V[x as usize] / 100;
                self.memory[self.I + 1] = (self.V[x as usize] % 100) / 10;
                self.memory[self.I + 2] = self.V[x as usize] % 10;
            }
            // Store registers V0 through Vx in memory starting at location I.
            (0xF, x, 0x5, 0x5) => self.memory.read_range(self.I, &self.V[0..=x as _]),
            // Read registers V0 through Vx from memory starting at location I.
            (0xF, x, 0x6, 0x5) => self.memory.write_range(self.I, &mut self.V[0..=x as _]),
            _ => error!("Unrecognized OpCode: {:X?}", opcode.nibbles()),
        }
        Ok(())
    }

    /**
     * Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
     * The interpreter reads n bytes from memory, starting at the address stored in I.
     * Sprites are XORed onto the existing screen. If this causes any pixels to be erased, VF is set to 1,
     * otherwise it is set to 0. If the sprite is positioned so part of it is outside the coordinates of the display,
     * it wraps around to the opposite side of the screen. See instruction 8xy3 for more information on XOR, and section 2.4, Display, for more information on the Chip-8 screen and sprites.
     */
    fn display_n_rows(&mut self, x: u8, y: u8, n: u8) {
        self.V[FLAGS_REGISTER] = 0;
        let (x, y) = (self.V[x as usize], self.V[y as usize]);
        for row in 0..n {
            self.V[FLAGS_REGISTER] |= self.display.set(
                x,
                y % display::HEIGHT as u8 + row,
                self.memory[self.I + row as Address],
            )
        }
    }
}
