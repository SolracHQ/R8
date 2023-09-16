use super::Address;
use std::{
    io::Read,
    ops::{Index, IndexMut},
};

/// http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#memmap
/// Size of the memory for the Chip8 system.
const MEMORY_SIZE: usize = 0x1000;

/// https://github.com/mattmikolay/chip-8/wiki/Mastering-CHIP%E2%80%908
/// HIP-8 contains built-in font utilities to allow for simple output of characters using the DXYN instruction.
/// All hexadecimal digits (0 - 9, A - F) have corresponding sprite data already stored in the memory of the interpreter.
const FONT_SET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10, 0xF0, 0x80, 0xF0, 0xF0,
    0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0, 0x10, 0xF0, 0xF0, 0x80,
    0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90, 0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0,
    0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0, 0x90, 0xE0, 0x90, 0xE0, 0xF0, 0x80, 0x80, 0x80,
    0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0, 0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80,
];

#[repr(transparent)]
/// Represents the memory of the Chip8 system.
pub struct Memory {
    ram: [u8; MEMORY_SIZE],
}

impl Memory {
    /// Creates a new memory for the Chip8 system.
    pub fn new() -> Self {
        Self {
            ram: [0; MEMORY_SIZE],
        }
    }

    /// Loads a new ROM into memory, restores the fonts, and clears the rest of the memory.
    ///
    /// # Arguments
    ///
    /// * `reader` - The reader to read the ROM from.
    ///
    /// # Returns
    ///
    /// * `Result<(), std::io::Error>` - Returns Ok if successful, otherwise returns an error.
    pub fn load_rom<R: Read>(&mut self, mut reader: R) -> Result<(), std::io::Error> {
        self.read_range(0, &FONT_SET);
        let mut buf = &mut self.ram[super::ENTRY_POINT as usize..];
        while !buf.is_empty() {
            match reader.read(buf) {
                Ok(0) => break,
                Ok(n) => {
                    buf = &mut buf[n..];
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
        if !buf.is_empty() {
            buf.fill(0)
        }
        Ok(())
    }

    // Reads a range of data from memory into a given slice.
    ///
    /// # Arguments
    ///
    /// * `start_address` - The starting address to read from.
    /// * `data` - The slice to read the data into.
    pub fn read_range(&mut self, start_address: Address, data: &[u8]) {
        self.ram[start_address as _..data.len() + start_address as usize].copy_from_slice(data);
    }

    /// Writes a range of data from a given slice into memory.
    ///
    /// # Arguments
    ///
    /// * `start_address` - The starting address to write to.
    /// * `data` - The slice to write the data from.
    pub(super) fn write_range(&mut self, start_address: Address, data: &mut [u8]) {
        data.copy_from_slice(&self.ram[start_address as _..data.len() + start_address as usize])
    }
}

impl Index<Address> for Memory {
    type Output = u8;

    #[inline(always)]
    /// Gets a reference to the byte at the given address in memory.
    ///
    /// # Arguments
    ///
    /// * `index` - The address to get the byte from.
    fn index(&self, index: Address) -> &Self::Output {
        &self.ram[index as usize]
    }
}

impl IndexMut<Address> for Memory {
    #[inline(always)]
    /// Gets a mutable reference to the byte at the given address in memory.
    ///
    /// # Arguments
    ///
    /// * `index` - The address to get the byte from.
    fn index_mut(&mut self, index: Address) -> &mut Self::Output {
        &mut self.ram[index as usize]
    }
}
