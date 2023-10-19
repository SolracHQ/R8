use std::fmt::Display;

use crate::{register::RegisterIndex, error::EmulatorError};

use super::memory::Address;

/// Represents a Chip-8 opcode.
pub enum Opcode {
    /// Ox00E0 - CLS
    ///
    /// Clear the display.
    Cls,
    /// 0x00EE - RET
    ///
    /// Return from a subroutine.
    Ret,
    /// 0x0NNN - SYS NNN
    ///
    /// Jump to a machine code routine at NNN.
    Sys { address: Address },
    /// 0x1NNN - JP NNN
    ///
    /// Jump to location NNN.
    Jp { address: Address },
    /// 0x2NNN - CALL NNN
    ///
    /// Call subroutine at NNN.
    Call { address: Address },
    /// 0x3XNN - SE VX, NN
    ///
    /// Skip next instruction if VX == NN.
    SeByte { x: RegisterIndex, byte: u8 },
    /// 0x4XNN - SNE VX, NN
    ///
    /// Skip next instruction if VX != NN.
    SneByte { x: RegisterIndex, byte: u8 },
    /// 0x5XY0 - SE VX, VY
    ///
    /// Skip next instruction if VX == VY.
    SeRegister { x: RegisterIndex, y: RegisterIndex },
    /// 0x6XNN - LD VX, NN
    ///
    /// Set VX = NN.
    LdByte { x: RegisterIndex, byte: u8 },
    /// 0x7XNN - ADD VX, NN
    ///
    /// Set VX = VX + NN.
    AddByte { x: RegisterIndex, byte: u8 },
    /// 0x8XY0 - LD VX, VY
    ///
    /// Set VX = VY.
    LdRegister { x: RegisterIndex, y: RegisterIndex },
    /// 0x8XY1 - OR VX, VY
    ///
    /// Set VX = VX OR VY.
    Or { x: RegisterIndex, y: RegisterIndex },
    /// 0x8XY2 - AND VX, VY
    ///
    /// Set VX = VX AND VY.
    And { x: RegisterIndex, y: RegisterIndex },
    /// 0x8XY3 - XOR VX, VY
    ///
    /// Set VX = VX XOR VY.
    Xor { x: RegisterIndex, y: RegisterIndex },
    /// 0x8XY4 - ADD VX, VY
    ///
    /// Set VX = VX + VY, set VF = carry.
    AddRegister { x: RegisterIndex, y: RegisterIndex },
    /// 0x8XY5 - SUB VX, VY
    ///
    /// Set VX = VX - VY, set VF = NOT borrow.
    Sub { x: RegisterIndex, y: RegisterIndex },
    /// 0x8XY6 - SHR VX {, VY}
    ///
    /// Set VX = VX SHR 1.
    Shr { x: RegisterIndex },
    /// 0x8XY7 - SUBN VX, VY
    ///
    /// Set VX = VY - VX, set VF = NOT borrow.
    Subn { x: RegisterIndex, y: RegisterIndex },
    /// 0x8XYE - SHL VX {, VY}
    ///
    /// Set VX = VX SHL 1.
    Shl { x: RegisterIndex },
    /// 0x9XY0 - SNE VX, VY
    ///
    /// Skip next instruction if VX != VY.
    SneRegister { x: RegisterIndex, y: RegisterIndex },
    /// 0xANNN - LD I, NNN
    ///
    /// Set I = NNN.
    LdI { address: Address },
    /// 0xBNNN - JP V0, NNN
    ///
    /// Jump to location NNN + V0.
    JpV0 { address: Address },
    /// 0xCXNN - RND VX, NN
    ///
    /// Set VX = random byte AND NN.
    Rnd { x: RegisterIndex, byte: u8 },
    /// 0xDXYN - DRW VX, VY, N
    ///
    /// Display N-byte sprite starting at memory location I at (VX, VY), set VF = collision.
    Drw { x: RegisterIndex, y: RegisterIndex, n: u8 },
    /// 0xEX9E - SKP VX
    ///
    /// Skip next instruction if key with the value of VX is pressed.
    Skp { x: RegisterIndex },
    /// 0xEXA1 - SKNP VX
    ///
    /// Skip next instruction if key with the value of VX is not pressed.
    Sknp { x: RegisterIndex },
    /// 0xFX07 - LD VX, DT
    ///
    /// Set VX = delay timer value.
    LdVxDT { x: RegisterIndex },
    /// 0xFX0A - LD VX, K
    ///
    /// Wait for a key press, store the value of the key in VX.
    LdVxK { x: RegisterIndex },
    /// 0xFX15 - LD DT, VX
    ///
    /// Set delay timer = VX.
    LdDTVx { x: RegisterIndex },
    /// 0xFX18 - LD ST, VX
    ///
    /// Set sound timer = VX.
    LdSTVx { x: RegisterIndex },
    /// 0xFX1E - ADD I, VX
    ///
    /// Set I = I + VX.
    AddIVx { x: RegisterIndex },
    /// 0xFX29 - LD F, VX
    ///
    /// Set I = location of sprite for digit VX.
    LdFVx { x: RegisterIndex },
    /// 0xFX33 - LD B, VX
    ///
    /// Store BCD representation of VX in memory locations I, I+1, and I+2.
    LdBVx { x: RegisterIndex },
    /// 0xFX55 - LD [I], VX
    ///
    /// Store registers V0 through VX in memory starting at location I.
    LdIVx { x: RegisterIndex },
    /// 0xFX65 - LD VX, [I]
    ///
    /// Read registers V0 through VX from memory starting at location I.
    LdVxI { x: RegisterIndex },
    /// Invalid opcode.
    Invalid(u16),
}

impl TryFrom<[u8; 2]> for Opcode {

    type Error = EmulatorError;
    
    /// Converts a 2-byte array into an opcode.
    /// 
    /// # Arguments
    /// 
    /// * `value` - The 2-byte array to convert.
    fn try_from(value: [u8; 2]) -> Result<Self, Self::Error> {
        Self::try_from(u16::from_be_bytes(value))
    }
}

impl TryFrom<u16> for Opcode {

    type Error = EmulatorError;

    /// Map a u16 value to the corresponding opcode.
    /// 
    /// # Arguments
    /// 
    /// * `value` - The u16 value to convert.
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        // Macros to help with parsing the opcode
        macro_rules! nibble {
            ($n:expr) => {
                (value >> (12 - (4 * $n)) & 0xF) as u8
            };
        }

        macro_rules! register {
            ($n:expr) => {
                RegisterIndex::try_new(nibble!($n))?
            };
        }

        macro_rules! address {
            () => {
                Address::new(value & 0xFFF)
            };
        }

        macro_rules! byte {
            () => {
                (value & 0xFF) as u8
            };
        }

        // Map the opcode to the corresponding enum variant
        let opcode = match value {
            0x00E0 => Self::Cls,
            0x00EE => Self::Ret,
            0x0000..=0x0FFF => Self::Sys {
                address: address!(),
            },
            0x1000..=0x1FFF => Self::Jp {
                address: address!(),
            },
            0x2000..=0x2FFF => Self::Call {
                address: address!(),
            },
            0x3000..=0x3FFF => Self::SeByte {
                x: register!(1),
                byte: byte!(),
            },
            0x4000..=0x4FFF => Self::SneByte {
                x: register!(1),
                byte: byte!(),
            },
            0x5000..=0x5FFF => match nibble!(3) {
                0x0 => Self::SeRegister {
                    x: register!(1),
                    y: register!(2),
                },
                _ => Self::Invalid(value),
            },
            0x6000..=0x6FFF => Self::LdByte {
                x: register!(1),
                byte: byte!(),
            },
            0x7000..=0x7FFF => Self::AddByte {
                x: register!(1),
                byte: byte!(),
            },
            0x8000..=0x8FFF => match nibble!(3) {
                0x0 => Self::LdRegister {
                    x: register!(1),
                    y: register!(2),
                },
                0x1 => Self::Or {
                    x: register!(1),
                    y: register!(2),
                },
                0x2 => Self::And {
                    x: register!(1),
                    y: register!(2),
                },
                0x3 => Self::Xor {
                    x: register!(1),
                    y: register!(2),
                },
                0x4 => Self::AddRegister {
                    x: register!(1),
                    y: register!(2),
                },
                0x5 => Self::Sub {
                    x: register!(1),
                    y: register!(2),
                },
                0x6 => Self::Shr { x: register!(1) },
                0x7 => Self::Subn {
                    x: register!(1),
                    y: register!(2),
                },
                0xE => Self::Shl { x: register!(1) },
                _ => Self::Invalid(value),
            },
            0x9000..=0x9FFF => match nibble!(3) {
                0x0 => Self::SneRegister {
                    x: register!(1),
                    y: register!(2),
                },
                _ => Self::Invalid(value),
            },
            0xA000..=0xAFFF => Self::LdI {
                address: address!(),
            },
            0xB000..=0xBFFF => Self::JpV0 {
                address: address!(),
            },
            0xC000..=0xCFFF => Self::Rnd {
                x: register!(1),
                byte: byte!(),
            },
            0xD000..=0xDFFF => Self::Drw {
                x: register!(1),
                y: register!(2),
                n: nibble!(3),
            },
            0xE000..=0xEFFF => match (nibble!(2), nibble!(3)) {
                (0x9, 0xE) => Self::Skp { x: register!(1) },
                (0xA, 0x1) => Self::Sknp { x: register!(1) },
                _ => Self::Invalid(value),
            },
            0xF000..=0xFFFF => match (nibble!(2), nibble!(3)) {
                (0x0, 0x7) => Self::LdVxDT { x: register!(1) },
                (0x0, 0xA) => Self::LdVxK { x: register!(1) },
                (0x1, 0x5) => Self::LdDTVx { x: register!(1) },
                (0x1, 0x8) => Self::LdSTVx { x: register!(1) },
                (0x1, 0xE) => Self::AddIVx { x: register!(1) },
                (0x2, 0x9) => Self::LdFVx { x: register!(1) },
                (0x3, 0x3) => Self::LdBVx { x: register!(1) },
                (0x5, 0x5) => Self::LdIVx { x: register!(1) },
                (0x6, 0x5) => Self::LdVxI { x: register!(1) },
                _ => Self::Invalid(value),
            },
        };
        Ok(opcode)
    }
}

impl Display for Opcode {
    /// Formats the opcode for display.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cls => write!(f, "CLS"),
            Self::Ret => write!(f, "RET"),
            Self::Sys { address } => write!(f, "SYS #{:X}", address.inner()),
            Self::Jp { address } => write!(f, "JP #{:X}", address.inner()),
            Self::Call { address } => write!(f, "CALL #{:X}", address.inner()),
            Self::SeByte { x, byte } => write!(f, "SE V{:X}, #{:X}", x, byte),
            Self::SneByte { x, byte } => write!(f, "SNE V{:X}, #{:X}", x, byte),
            Self::SeRegister { x, y } => write!(f, "SE V{:X}, V{:X}", x, y),
            Self::LdByte { x, byte } => write!(f, "LD V{:X}, #{:X}", x, byte),
            Self::AddByte { x, byte } => write!(f, "ADD V{:X}, #{:X}", x, byte),
            Self::LdRegister { x, y } => write!(f, "LD V{:X}, V{:X}", x, y),
            Self::Or { x, y } => write!(f, "OR V{:X}, V{:X}", x, y),
            Self::And { x, y } => write!(f, "AND V{:X}, V{:X}", x, y),
            Self::Xor { x, y } => write!(f, "XOR V{:X}, V{:X}", x, y),
            Self::AddRegister { x, y } => write!(f, "ADD V{:X}, V{:X}", x, y),
            Self::Sub { x, y } => write!(f, "SUB V{:X}, V{:X}", x, y),
            Self::Shr { x } => write!(f, "SHR V{:X}", x),
            Self::Subn { x, y } => write!(f, "SUBN V{:X}, V{:X}", x, y),
            Self::Shl { x } => write!(f, "SHL V{:X}", x),
            Self::SneRegister { x, y } => write!(f, "SNE V{:X}, V{:X}", x, y),
            Self::LdI { address } => write!(f, "LD I, #{:X}", address.inner()),
            Self::JpV0 { address } => write!(f, "JP V0, #{:X}", address.inner()),
            Self::Rnd { x, byte } => write!(f, "RND V{:X}, #{:X}", x, byte),
            Self::Drw { x, y, n } => write!(f, "DRW V{:X}, V{:X}, #{:X}", x, y, n),
            Self::Skp { x } => write!(f, "SKP V{:X}", x),
            Self::Sknp { x } => write!(f, "SKNP V{:X}", x),
            Self::LdVxDT { x } => write!(f, "LD V{:X}, DT", x),
            Self::LdVxK { x } => write!(f, "LD V{:X}, K", x),
            Self::LdDTVx { x } => write!(f, "LD DT, V{:X}", x),
            Self::LdSTVx { x } => write!(f, "LD ST, V{:X}", x),
            Self::AddIVx { x } => write!(f, "ADD I, V{:X}", x),
            Self::LdFVx { x } => write!(f, "LD F, V{:X}", x),
            Self::LdBVx { x } => write!(f, "LD B, V{:X}", x),
            Self::LdIVx { x } => write!(f, "LD [I], V{:X}", x),
            Self::LdVxI { x } => write!(f, "LD V{:X}, [I]", x),
            Self::Invalid(value) => write!(f, "#{:X}", value),
        }
    }
}
