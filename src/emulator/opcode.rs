use std::fmt::Display;

use super::memory::Address;

/// Represents a CHIP-8 opcode.
#[repr(transparent)]
pub struct Opcode(u16);

impl Opcode {
    /// Creates a new opcode from two bytes.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The two bytes to create the opcode from.
    ///
    /// # Returns
    ///
    /// * `Opcode` - The opcode created from the two bytes.
    pub fn new(bytes: [u8; 2]) -> Self {
        Self(u16::from_be_bytes(bytes))
    }

    /// Extracts the nibbles from the opcode.
    ///
    /// What is a nibble? A nibble is a 4-bit value, or half a byte.
    ///
    /// # Returns
    ///
    /// * `(u8, u8, u8, u8)` - The nibbles of the opcode.
    pub fn nibbles(&self) -> (u8, u8, u8, u8) {
        (
            ((self.0 & 0xF000) >> 12) as u8,
            ((self.0 & 0x0F00) >> 8) as u8,
            ((self.0 & 0x00F0) >> 4) as u8,
            (self.0 & 0x000F) as u8,
        )
    }

    /// Extracts the lower 12 bits of the opcode.
    ///
    /// # Returns
    ///
    /// * `Address` - The lower 12 bits of the opcode.
    pub fn nnn(&self) -> Address {
        Address::new(self.0 & 0xFFF)
    }

    /// Extracts the lower 8 bits of the opcode.
    ///
    /// # Returns
    ///
    /// * `u8` - The lower 8 bits of the opcode.
    pub fn kk_byte(&self) -> u8 {
        self.0 as u8
    }
}

impl Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let nibbles = self.nibbles();
        match nibbles {
            (0, 0, 0xE, 0) => write!(f, "CLS"),
            (0, 0, 0xE, 0xE) => write!(f, "RET"),
            (0, _, _, _) => write!(f, "SYS 0x{address:X}", address = self.nnn().inner()),
            (0x1, _, _, _) => write!(f, "JP 0x{address:X}", address = self.nnn().inner()),
            (0x2, _, _, _) => write!(f, "CALL 0x{address:X}", address = self.nnn().inner()),
            (0x3, x, _, _) => write!(f, "SE V{x:X}, 0x{kk:X}", kk = self.kk_byte()),
            (0x4, x, _, _) => write!(f, "SNE V{x:X}, 0x{kk:X}", kk = self.kk_byte()),
            (0x5, x, y, 0) => write!(f, "SE V{x:X}, V{y:X}"),
            (0x6, x, _, _) => write!(f, "LD V{x:X} 0x{kk:X}", kk = self.kk_byte()),
            (0x7, x, _, _) => write!(f, "ADD V{x:X} 0x{kk:X}", kk = self.kk_byte()),
            (0x8, x, y, 0x0) => write!(f, "LD V{x:X}, V{y:X}"),
            (0x8, x, y, 0x1) => write!(f, "OR V{x:X}, V{y:X}"),
            (0x8, x, y, 0x2) => write!(f, "AND V{x:X}, V{y:X}"),
            (0x8, x, y, 0x3) => write!(f, "XOR V{x:X}, V{y:X}"),
            (0x8, x, y, 0x4) => write!(f, "ADD V{x:X}, V{y:X}"),
            (0x8, x, y, 0x5) => write!(f, "SUB V{x:X}, V{y:X}"),
            (0x8, x, _, 0x6) => write!(f, "SHR V{x:X}"),
            (0x8, x, y, 0x7) => write!(f, "SUBN V{x:X}, V{y:X}"),
            (0x8, x, _, 0xE) => write!(f, "SHL V{x:X}"),
            (0x9, x, y, 0) => write!(f, "SNE V{x:X}, V{y:X}"),
            (0xA, _, _, _) => write!(f, "LD I, 0x{address:X}", address = self.nnn().inner()),
            (0xB, _, _, _) => write!(f, "JP V0, 0x{address:X}", address = self.nnn().inner()),
            (0xC, x, _, _) => write!(f, "RND V{x:X}, 0x{kk:X}", kk = self.kk_byte()),
            (0xD, x, y, n) => write!(f, "DRW V{x:X}, V{y:X}, 0x{n:X}"),
            (0xE, x, 0x9, 0xE) => write!(f, "SKP V{x:X}"),
            (0xE, x, 0xA, 0x1) => write!(f, "SKNP V{x:X}"),
            (0xF, x, 0x0, 0x7) => write!(f, "LD V{x:X}, DT"),
            (0xF, x, 0x0, 0xA) => write!(f, "LD V{x:X}, K"),
            (0xF, x, 0x1, 0x5) => write!(f, "LD DT, V{x:X}"),
            (0xF, x, 0x1, 0x8) => write!(f, "LD ST, V{x:X}"),
            (0xF, x, 0x1, 0xE) => write!(f, "ADD I, V{x:X}"),
            (0xF, x, 0x2, 0x9) => write!(f, "LD F, V{x:X}"),
            (0xF, x, 0x3, 0x3) => write!(f, "LD B, V{x:X}"),
            (0xF, x, 0x5, 0x5) => write!(f, "LD [I], V{x:X}"),
            (0xF, x, 0x6, 0x5) => write!(f, "LD V{x:X}, [I]"),
            _ => write!( f, "0x{:X} 0x{:X}",(self.0 >> 8) as u8, (self.0 & 0xFF) as u8),
        }
    }
}
