use std::{
  io::Read,
  ops::{Index, IndexMut},
};

use super::error::EmulatorError;

/// Represents an address in memory.
///
/// # Fields
///
/// * `0x000` - `0xFFF` - The address in memory.
///
/// # Note
///
/// This is a newtype around `u16` to make it more clear that it represents an address.
/// Chip-8 Only have 12 bits of address space, so the upper 4 bits are always 0.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct Address(u16);

impl Address {
  /// The address of the fonts in memory.
  pub const FONTS_INDEX: Self = Self(0);
  /// The address of the entry point in memory.
  /// http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#memmap
  pub const ENTRY_POINT: Self = Self(0x200);

  /// Creates a new address.
  ///
  /// # Arguments
  ///
  /// * `address` - The address to create.
  ///
  /// # Returns
  ///
  /// * `Result<Address, super::RuntimeError>` - Returns Ok if the address is valid, otherwise returns an error.
  pub fn try_new(address: u16) -> Result<Self, super::EmulatorError> {
    if address > 0xFFF {
      Err(super::EmulatorError::InvalidAddress(address))
    } else {
      Ok(Self(address))
    }
  }

  /// Creates a new address.
  ///
  /// Only use this if you are sure the address is valid.
  ///
  /// # Arguments
  ///
  /// * `address` - The address to create.
  ///
  /// # Returns
  ///
  /// * `Address` - The address created.
  pub fn new(address: u16) -> Self {
    // The address must be always valid.
    // if invalid address is passed will be truncated to 12 bits.
    Self(address & 0xFFF)
  }

  /// Adds a `u16` to the address in place.
  ///
  /// # Arguments
  ///
  /// * `other` - The `u16` to add to the address.
  ///
  /// # Returns
  ///
  /// * `Result<(), RuntimeError>` - Returns Ok if the address is valid, otherwise returns an error.
  pub fn add_assign(&mut self, other: u16) -> Result<(), EmulatorError> {
    *self = Self::try_new(self.0 + other)?;
    Ok(())
  }

  /// Returns the address as a `u16`.
  ///
  /// # Returns
  ///
  /// * `u16` - The inner address.
  pub fn inner(&self) -> u16 {
    self.0
  }
}

impl TryFrom<u16> for Address {
  type Error = super::EmulatorError;

  fn try_from(value: u16) -> Result<Self, Self::Error> {
    Self::try_new(value)
  }
}

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

/// Represents the memory of the Chip8 system.
///
/// # Fields
///
/// * `ram` - The memory of the Chip8 system.
#[repr(transparent)]
pub struct Memory {
  ram: [u8; MEMORY_SIZE],
}

impl Memory {
  /// Creates a new memory for the Chip8 system.
  ///
  /// # Returns
  ///
  /// * `Memory` - The memory created.
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
  /// * `Result<(), RuntimeError>` - Returns Ok if successful, otherwise returns an error.
  ///
  /// # Note
  ///
  /// This function will clear the memory before loading the ROM.
  pub fn load_rom<R: Read>(&mut self, mut reader: R) -> Result<(), EmulatorError> {
    // Load the fonts at the start of the memory.
    self.read_range(Address::FONTS_INDEX, &FONT_SET)?;

    // Clear the memory between the fonts and the entry point.
    self.ram[Address::FONTS_INDEX.0 as usize + FONT_SET.len()..Address::ENTRY_POINT.0 as usize]
      .fill(0);

    // Load the ROM.
    let mut buf = &mut self.ram[Address::ENTRY_POINT.0 as usize..];
    while !buf.is_empty() {
      match reader.read(buf) {
        Ok(0) => break,
        Ok(n) => {
          buf = &mut buf[n..];
        }
        Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => {}
        Err(e) => return Err(EmulatorError::LoadError(e)),
      }
    }
    // Clear the rest of the memory.
    if !buf.is_empty() {
      buf.fill(0)
    }
    Ok(())
  }

  /// Reads a range of data from memory into a given slice.
  ///
  /// # Arguments
  ///
  /// * `start_address` - The starting address to read from.
  /// * `data` - The slice to read the data into.
  ///
  /// # Returns
  ///
  /// * `Result<(), RuntimeError>` - Returns Ok if successful, otherwise returns an error.
  pub fn read_range(&mut self, start_address: Address, data: &[u8]) -> Result<(), EmulatorError> {
    // Check if the address is valid.
    if data.len() + start_address.0 as usize > 0xFFF {
      return Err(EmulatorError::OutOfBounds(
        data.len() as u16 + start_address.0,
      ));
    }
    /*
    I do this in this way to avoid the code to panic if the address is invalid.
    since I already checked if the address is valid, I can use unsafe code.
    SAFETY: The address is always valid. (sice come from safe code)
    SAFETY: The data slice is always valid. (sice come from safe code)
    SAFETY: The data slice is always the correct length. (sice come from safe code)
    SAFETY: The data slice is always aligned. (sice come from safe code)
    SAFETY: The data slice is always initialized. (sice come from safe code)
    SAFETY: Since &mut self.ram is unique, the data slice is unique (no overlapping)
    */
    unsafe {
      std::ptr::copy_nonoverlapping(
        data.as_ptr(),
        self.ram.as_mut_ptr().add(start_address.0 as _),
        data.len(),
      )
    };
    Ok(())
  }

  /// Writes a range of data from a given slice into memory.
  ///
  /// # Arguments
  ///
  /// * `start_address` - The starting address to write to.
  /// * `data` - The slice to write the data from.
  pub fn write_range(&self, start_address: Address, data: &mut [u8]) -> Result<(), EmulatorError> {
    if start_address.0 as usize + data.len() > 0xFFF {
      return Err(EmulatorError::OutOfBounds(
        data.len() as u16 + start_address.0,
      ));
    }
    /*
    I do this in this way to avoid the code to panic if the address is invalid.
    since I already checked if the address is valid, I can use unsafe code.
    SAFETY: The address is always valid. (sice come from safe code)
    SAFETY: The data slice is always valid. (sice come from safe code)
    SAFETY: The data slice is always the correct length. (sice come from safe code)
    SAFETY: The data slice is always aligned. (sice come from safe code)
    SAFETY: The data slice is always initialized. (sice come from safe code)
    SAFETY: Since data: &mut [u8] is unique, the data slice is unique (no overlapping)
     */
    unsafe {
      std::ptr::copy_nonoverlapping(
        self.ram.as_ptr().add(start_address.0 as _),
        data.as_mut_ptr(),
        data.len(),
      )
    };
    Ok(())
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
  ///
  /// # Returns
  ///
  /// * `&u8` - A reference to the byte at the given address.
  fn index(&self, index: Address) -> &Self::Output {
    // SAFETY: The address by dessign is always valid.
    unsafe { &*self.ram.as_ptr().add(index.0 as usize) }
  }
}

impl IndexMut<Address> for Memory {
  #[inline(always)]
  /// Gets a mutable reference to the byte at the given address in memory.
  ///
  /// # Arguments
  ///
  /// * `index` - The address to get the byte from.
  ///
  /// # Returns
  ///
  /// * `&mut u8` - A mutable reference to the byte at the given address.
  fn index_mut(&mut self, index: Address) -> &mut Self::Output {
    // SAFETY: The address is always valid.
    unsafe { &mut *self.ram.as_mut_ptr().add(index.0 as usize) }
  }
}
