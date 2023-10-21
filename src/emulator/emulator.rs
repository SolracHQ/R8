use std::io::Read;

use log::{debug, error};

use crate::{
    display::Display,
    error::EmulatorError,
    keyboard::KeyBoard,
    memory::{Address, Memory},
    opcode::Opcode,
    rand::RandGen,
    register::{RegisterIndex, VRegisters},
    stack::Stack, timer::Timer,
};

/// Represents the state of the emulator.
#[derive(Debug)]
pub enum State {
    New,
    Running,
    WaitingKey { x: RegisterIndex },
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
    pub(crate) pc: Address,
    pub(crate) i: Address,
    pub(crate) registers: VRegisters,
    pub(crate) sound_timer: Timer,
    pub(crate) delay_timer: Timer,
    // Memory Segments
    pub(crate) stack: Stack<Address>,
    pub(crate) memory: Memory,
    // Devices
    pub display: Display,
    pub keyboard: KeyBoard,
    // Helper Structs
    pub(crate) rand: RandGen,
    pub(crate) state: State,
}

impl Emulator {
    /// Creates a new `Emulator` on state `New`.
    ///
    /// # Returns
    ///
    /// * `Emulator` - The newly created emulator.
    pub fn new() -> Self {
        Self {
            pc: Address::ENTRY_POINT,
            i: Address::new(0),
            registers: VRegisters::default(),
            sound_timer: Timer::new(),
            delay_timer: Timer::new(),
            stack: Stack::new(),
            memory: Memory::new(),
            display: Display::new(),
            keyboard: KeyBoard::default(),
            rand: RandGen::new(),
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
    pub fn load_rom<R: Read>(&mut self, reader: R) -> Result<(), EmulatorError> {
        self.pc = Address::ENTRY_POINT;
        self.i = Address::new(0);
        self.delay_timer = Timer::new();
        self.sound_timer = Timer::new();
        self.registers = VRegisters::default();
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
        match self.state {
            State::New => return Ok(()),
            State::WaitingKey { x } => {
                let Some(key) = (0..=0xF).find(|&key| self.keyboard.is_set(key)) else {
                    return Ok(());
                };
                self.registers[x] = key;
                self.state = State::Running;
            }
            _ => {}
        }

        self.sound_timer.decrement();
        self.delay_timer.decrement();

        // Fetch the opcode
        let opcode = self.fetch_opcode()?;

        debug!("| 0x{PC:X} | {opcode}", PC = self.pc.inner());

        self.execute_opcode(opcode)?;

        Ok(())
    }

    /// Fetches the next opcode from memory.
    ///
    /// # Returns
    ///
    /// * `Result<Opcode, RuntimeError>` - The next opcode or an error if the opcode could not be fetched.
    pub fn fetch_opcode(&self) -> Result<Opcode, EmulatorError> {
        let mut opcode = [0, 0];
        self.memory.write_range(self.pc, &mut opcode)?;
        opcode.try_into()
    }

    /// Executes an opcode (instruction) on the emulator.
    ///
    /// # Arguments
    ///
    /// * `opcode` - The opcode to execute.
    ///
    /// # Returns
    ///
    /// * `Result<(), RuntimeError>` - () if the opcode was executed successfully or an error if the cpu encountered any problems.
    pub fn execute_opcode(&mut self, opcode: Opcode) -> Result<(), EmulatorError> {
        // Macro to jump if a condition is met
        macro_rules! jump_if {
            ($op:tt, $x:expr, $y:expr) => {
                if $x $op $y { self.pc.add_assign(2)?; }
            };
        }
        // Macro to facilitate access to the V registers
        macro_rules! V {
            (0) => {
                self.registers[RegisterIndex::ZERO]
            };
            (FLAGS) => {
                self.registers[RegisterIndex::FLAG]
            };
            ($reg: expr) => {
                // Cehck reg is u8
                self.registers[$reg]
            };
            (0 => $end: expr) => {
                // Cehck reg is u8
                self.registers[RegisterIndex::ZERO..=$end]
            };
        }

        // Increment the program counter by 2
        self.pc.add_assign(2)?;

        match opcode {
            Opcode::Cls => self.display.clear(),
            Opcode::Ret => self.pc = self.stack.pop()?,
            Opcode::Jp { address } => self.pc = address,
            Opcode::Sys { address } | Opcode::Call { address } => {
                self.stack.push(self.pc)?;
                self.pc = address;
            }
            Opcode::SeByte { x, byte } => jump_if!(==, V![x], byte),
            Opcode::SneByte { x, byte } => jump_if!(!=, V![x], byte),
            Opcode::SeRegister { x, y } => jump_if!(==, V![x], V![y]),
            Opcode::LdByte { x, byte } => V![x] = byte,
            Opcode::AddByte { x, byte } => V![x] = V![x].wrapping_add(byte),
            Opcode::LdRegister { x, y } => V![x] = V![y],
            Opcode::Or { x, y } => V![x] |= V![y],
            Opcode::And { x, y } => V![x] &= V![y],
            Opcode::Xor { x, y } => V![x] ^= V![y],
            Opcode::AddRegister { x, y } => {
                let result = V![x] as u16 + V![y] as u16;
                V![x] = (result & 0xFF) as u8;
                V![FLAGS] = if result & 0xFF00 != 0 { 1 } else { 0 }
            }
            Opcode::Sub { x, y } => {
                V![FLAGS] = if V![x] > V![y] { 1 } else { 0 };
                V![x] = V![x].wrapping_sub(V![y]);
            }
            Opcode::Shr { x } => {
                V![FLAGS] = V![x] & 1;
                V![x] >>= 1;
            }
            Opcode::Subn { x, y } => {
                V![FLAGS] = if V![y] > V![x] { 1 } else { 0 };
                V![x] = V![y].wrapping_sub(V![x]);
            }
            Opcode::Shl { x } => {
                V![FLAGS] = (V![x] >> 7) & 1;
                V![x] <<= 1;
            }
            Opcode::SneRegister { x, y } => {
                if V![x] != V![y] {
                    self.pc.add_assign(2)?
                }
            }
            Opcode::LdI { address } => self.i = address,
            Opcode::JpV0 { address } => self.pc.add_assign(address.inner() + V![0] as u16)?,
            Opcode::Rnd { x, byte } => V![x] = self.rand.next() & byte,
            Opcode::Drw { x, y, n } => {
                V![FLAGS] = 0;
                let (x, y) = (V![x], V![y]);
                for row in 0..n {
                    V![FLAGS] |= self.display.set(
                        x,
                        y % crate::constants::HEIGHT as u8 + row,
                        self.memory[(self.i.inner() + row as u16).try_into()?],
                    )
                }
            }
            Opcode::Skp { x } => {
                if self.keyboard.is_set(V![x] & 0xF) {
                    self.pc.add_assign(2)?;
                }
            }
            Opcode::Sknp { x } => {
                if !self.keyboard.is_set(V![x] & 0xF) {
                    self.pc.add_assign(2)?;
                }
            }
            Opcode::LdVxDT { x } => V![x] = self.delay_timer.get(),
            Opcode::LdVxK { x } => self.state = State::WaitingKey { x },
            Opcode::LdDTVx { x } => self.delay_timer.set(V![x]),
            Opcode::LdSTVx { x } => self.sound_timer.set(V![x]),
            Opcode::AddIVx { x } => self.i.add_assign(V![x] as u16)?,
            Opcode::LdFVx { x } => self.i = Address::new((V![x] & 0xF) as u16 * 5),
            Opcode::LdBVx { x } => self.memory.read_range(self.i, &bcd(V![x]))?,
            Opcode::LdIVx { x } => self.memory.read_range(self.i, &V![0 => x])?,
            Opcode::LdVxI { x } => self.memory.write_range(self.i, &mut V![0 => x])?,
            Opcode::Invalid(data) => {
                error!(
                    "Unrecognized OpCode: | 0x{PC:X} | {:X?}",
                    data,
                    PC = self.pc.inner()
                )
            }
        }

        Ok(())
    }
}

impl Default for Emulator {
    fn default() -> Self {
        Self::new()
    }
}

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