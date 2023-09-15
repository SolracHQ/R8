use super::Address;
use std::{
    io::Read,
    ops::{Index, IndexMut},
};

/// http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#memmap
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

pub struct Memory {
    ram: [u8; MEMORY_SIZE],
}

impl Memory {
    pub fn new() -> Self {
        Self {
            ram: [0; MEMORY_SIZE],
        }
    }

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
        Ok(())
    }

    pub fn read_range(&mut self, start_address: Address, data: &[u8]) {
        self.ram[start_address as _..data.len() + start_address as usize].copy_from_slice(data);
    }

    pub fn write_range(&mut self, start_address: Address, data: &mut [u8]) {
        data.copy_from_slice(&self.ram[start_address as _..data.len() + start_address as usize])
    }
}

impl Index<Address> for Memory {
    type Output = u8;

    #[inline(always)]
    fn index(&self, index: Address) -> &Self::Output {
        &self.ram[index as usize]
    }
}

impl IndexMut<Address> for Memory {
    #[inline(always)]
    fn index_mut(&mut self, index: Address) -> &mut Self::Output {
        &mut self.ram[index as usize]
    }
}
