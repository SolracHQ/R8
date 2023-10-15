use std::fmt::Display;

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
    SeByte { x: u8, byte: u8 },
    /// 0x4XNN - SNE VX, NN
    ///
    /// Skip next instruction if VX != NN.
    SneByte { x: u8, byte: u8 },
    /// 0x5XY0 - SE VX, VY
    ///
    /// Skip next instruction if VX == VY.
    SeRegister { x: u8, y: u8 },
    /// 0x6XNN - LD VX, NN
    ///
    /// Set VX = NN.
    LdByte { x: u8, byte: u8 },
    /// 0x7XNN - ADD VX, NN
    ///
    /// Set VX = VX + NN.
    AddByte { x: u8, byte: u8 },
    /// 0x8XY0 - LD VX, VY
    ///
    /// Set VX = VY.
    LdRegister { x: u8, y: u8 },
    /// 0x8XY1 - OR VX, VY
    ///
    /// Set VX = VX OR VY.
    Or { x: u8, y: u8 },
    /// 0x8XY2 - AND VX, VY
    ///
    /// Set VX = VX AND VY.
    And { x: u8, y: u8 },
    /// 0x8XY3 - XOR VX, VY
    ///
    /// Set VX = VX XOR VY.
    Xor { x: u8, y: u8 },
    /// 0x8XY4 - ADD VX, VY
    ///
    /// Set VX = VX + VY, set VF = carry.
    AddRegister { x: u8, y: u8 },
    /// 0x8XY5 - SUB VX, VY
    ///
    /// Set VX = VX - VY, set VF = NOT borrow.
    Sub { x: u8, y: u8 },
    /// 0x8XY6 - SHR VX {, VY}
    ///
    /// Set VX = VX SHR 1.
    Shr { x: u8 },
    /// 0x8XY7 - SUBN VX, VY
    ///
    /// Set VX = VY - VX, set VF = NOT borrow.
    Subn { x: u8, y: u8 },
    /// 0x8XYE - SHL VX {, VY}
    ///
    /// Set VX = VX SHL 1.
    Shl { x: u8 },
    /// 0x9XY0 - SNE VX, VY
    ///
    /// Skip next instruction if VX != VY.
    SneRegister { x: u8, y: u8 },
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
    Rnd { x: u8, byte: u8 },
    /// 0xDXYN - DRW VX, VY, N
    ///
    /// Display N-byte sprite starting at memory location I at (VX, VY), set VF = collision.
    Drw { x: u8, y: u8, n: u8 },
    /// 0xEX9E - SKP VX
    ///
    /// Skip next instruction if key with the value of VX is pressed.
    Skp { x: u8 },
    /// 0xEXA1 - SKNP VX
    ///
    /// Skip next instruction if key with the value of VX is not pressed.
    Sknp { x: u8 },
    /// 0xFX07 - LD VX, DT
    ///
    /// Set VX = delay timer value.
    LdVxDT { x: u8 },
    /// 0xFX0A - LD VX, K
    ///
    /// Wait for a key press, store the value of the key in VX.
    LdVxK { x: u8 },
    /// 0xFX15 - LD DT, VX
    ///
    /// Set delay timer = VX.
    LdDTVx { x: u8 },
    /// 0xFX18 - LD ST, VX
    ///
    /// Set sound timer = VX.
    LdSTVx { x: u8 },
    /// 0xFX1E - ADD I, VX
    ///
    /// Set I = I + VX.
    AddIVx { x: u8 },
    /// 0xFX29 - LD F, VX
    ///
    /// Set I = location of sprite for digit VX.
    LdFVx { x: u8 },
    /// 0xFX33 - LD B, VX
    ///
    /// Store BCD representation of VX in memory locations I, I+1, and I+2.
    LdBVx { x: u8 },
    /// 0xFX55 - LD [I], VX
    ///
    /// Store registers V0 through VX in memory starting at location I.
    LdIVx { x: u8 },
    /// 0xFX65 - LD VX, [I]
    ///
    /// Read registers V0 through VX from memory starting at location I.
    LdVxI { x: u8 },
    /// Invalid opcode.
    Invalid(u16),
}

impl From<[u8; 2]> for Opcode {
    /// Converts a 2-byte array into an opcode.
    /// 
    /// # Arguments
    /// 
    /// * `value` - The 2-byte array to convert.
    fn from(value: [u8; 2]) -> Self {
        Self::from(u16::from_be_bytes(value))
    }
}

impl From<u16> for Opcode {
    /// Map a u16 value to the corresponding opcode.
    /// 
    /// # Arguments
    /// 
    /// * `value` - The u16 value to convert.
    fn from(value: u16) -> Self {
        // Macros to help with parsing the opcode
        macro_rules! nibble {
            ($n:expr) => {
                (value >> (12 - (4 * $n)) & 0xF) as u8
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
        match value {
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
                x: nibble!(1),
                byte: byte!(),
            },
            0x4000..=0x4FFF => Self::SneByte {
                x: nibble!(1),
                byte: byte!(),
            },
            0x5000..=0x5FFF => match nibble!(3) {
                0x0 => Self::SeRegister {
                    x: nibble!(1),
                    y: nibble!(2),
                },
                _ => Self::Invalid(value),
            },
            0x6000..=0x6FFF => Self::LdByte {
                x: nibble!(1),
                byte: byte!(),
            },
            0x7000..=0x7FFF => Self::AddByte {
                x: nibble!(1),
                byte: byte!(),
            },
            0x8000..=0x8FFF => match nibble!(3) {
                0x0 => Self::LdRegister {
                    x: nibble!(1),
                    y: nibble!(2),
                },
                0x1 => Self::Or {
                    x: nibble!(1),
                    y: nibble!(2),
                },
                0x2 => Self::And {
                    x: nibble!(1),
                    y: nibble!(2),
                },
                0x3 => Self::Xor {
                    x: nibble!(1),
                    y: nibble!(2),
                },
                0x4 => Self::AddRegister {
                    x: nibble!(1),
                    y: nibble!(2),
                },
                0x5 => Self::Sub {
                    x: nibble!(1),
                    y: nibble!(2),
                },
                0x6 => Self::Shr { x: nibble!(1) },
                0x7 => Self::Subn {
                    x: nibble!(1),
                    y: nibble!(2),
                },
                0xE => Self::Shl { x: nibble!(1) },
                _ => Self::Invalid(value),
            },
            0x9000..=0x9FFF => match nibble!(3) {
                0x0 => Self::SneRegister {
                    x: nibble!(1),
                    y: nibble!(2),
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
                x: nibble!(1),
                byte: byte!(),
            },
            0xD000..=0xDFFF => Self::Drw {
                x: nibble!(1),
                y: nibble!(2),
                n: nibble!(3),
            },
            0xE000..=0xEFFF => match (nibble!(2), nibble!(3)) {
                (0x9, 0xE) => Self::Skp { x: nibble!(1) },
                (0xA, 0x1) => Self::Sknp { x: nibble!(1) },
                _ => Self::Invalid(value),
            },
            0xF000..=0xFFFF => match (nibble!(2), nibble!(3)) {
                (0x0, 0x7) => Self::LdVxDT { x: nibble!(1) },
                (0x0, 0xA) => Self::LdVxK { x: nibble!(1) },
                (0x1, 0x5) => Self::LdDTVx { x: nibble!(1) },
                (0x1, 0x8) => Self::LdSTVx { x: nibble!(1) },
                (0x1, 0xE) => Self::AddIVx { x: nibble!(1) },
                (0x2, 0x9) => Self::LdFVx { x: nibble!(1) },
                (0x3, 0x3) => Self::LdBVx { x: nibble!(1) },
                (0x5, 0x5) => Self::LdIVx { x: nibble!(1) },
                (0x6, 0x5) => Self::LdVxI { x: nibble!(1) },
                _ => Self::Invalid(value),
            },
        }
    }
}

impl Display for Opcode {
    /// Formats the opcode for display.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cls => write!(f, "CLS"),
            Self::Ret => write!(f, "RET"),
            Self::Sys { address } => write!(f, "SYS 0x{:X}", address.inner()),
            Self::Jp { address } => write!(f, "JP 0x{:X}", address.inner()),
            Self::Call { address } => write!(f, "CALL 0x{:X}", address.inner()),
            Self::SeByte { x, byte } => write!(f, "SE V{:X}, 0x{:X}", x, byte),
            Self::SneByte { x, byte } => write!(f, "SNE V{:X}, 0x{:X}", x, byte),
            Self::SeRegister { x, y } => write!(f, "SE V{:X}, V{:X}", x, y),
            Self::LdByte { x, byte } => write!(f, "LD V{:X}, 0x{:X}", x, byte),
            Self::AddByte { x, byte } => write!(f, "ADD V{:X}, 0x{:X}", x, byte),
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
            Self::LdI { address } => write!(f, "LD I, 0x{:X}", address.inner()),
            Self::JpV0 { address } => write!(f, "JP V0, 0x{:X}", address.inner()),
            Self::Rnd { x, byte } => write!(f, "RND V{:X}, 0x{:X}", x, byte),
            Self::Drw { x, y, n } => write!(f, "DRW V{:X}, V{:X}, 0x{:X}", x, y, n),
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
            Self::Invalid(value) => write!(f, "0x{:X}", value),
        }
    }
}
