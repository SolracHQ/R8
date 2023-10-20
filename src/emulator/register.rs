use crate::error::EmulatorError;


/// Represents a CHIP-8 Register Index. 
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct RegisterIndex(u8);

impl RegisterIndex {

    /// The RegisterIndex for the Zero Register
    pub const ZERO: RegisterIndex = RegisterIndex(0);

    /// The RegisterIndex for the Flag Register
    pub const FLAG: RegisterIndex = RegisterIndex(0xF);

    /**
     * Creates a new RegisterIndex from a u8 value.
     * 
     * # Arguments
     * 
     * * `value` - The value to create the RegisterIndex from.
     * 
     * # Returns
     * 
     * * `RegisterIndex` - The newly created RegisterIndex.
     */
    pub const fn new(value: u8) -> Self {
        // Chip-8 Only Has 16 V-Registers, so we mask the value to 4 bits
        Self(value & 0x0F)
    }

    /**
     * Creates a new RegisterIndex from a u8 value.
     * 
     * # Arguments
     * 
     * * `value` - The value to create the RegisterIndex from.
     * 
     * # Returns
     * 
     * * `Result<RegisterIndex, EmulatorError>` - The newly created RegisterIndex, or an error if the value is invalid.
     */
    pub fn try_new(value: u8) -> Result<Self, EmulatorError> {
        if value > 0x0F {
            Err(EmulatorError::InvalidRegister(value))
        } else {
            Ok(Self::new(value))
        }
    }
}

impl std::convert::TryFrom<u8> for RegisterIndex {
    type Error = EmulatorError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl std::fmt::UpperHex for RegisterIndex {
    /**
     * Formats the RegisterIndex as uppercase hexadecimal.
     * 
     * # Arguments
     * 
     * * `f` - The formatter to use.
     * 
     * # Returns
     * 
     * * `std::fmt::Result` - The result of the formatting.
     */
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:X}", self.0)
    }
}

/**
 * Represents the V-Registers in the CHIP-8.
 * 
 * # Fields
 * 
 * * `registers` - The registers.
 */
#[repr(transparent)]
#[derive(Default)]
pub struct VRegisters {
    registers: [u8; crate::constants::REGISTER_COUNT],
}

impl VRegisters {
    /**
     * Indexes the VRegisters wihout panicking.
     * 
     * # Arguments
     * 
     * * `index` - The index to get the value from.
     * 
     * # Returns
     * 
     * * `Result<&u8, EmulatorError>` - The value at the index, or an error if the index is invalid.
     */
    pub fn try_index(&self, index: u8) -> Result<&u8, EmulatorError> {
        if index > 0x0F {
            Err(EmulatorError::InvalidRegister(index))
        } else {
            Ok(&self.registers[index as usize])
        }
    }
}

impl std::ops::Index<RegisterIndex> for VRegisters {
    type Output = u8;

    /**
     * Indexes the VRegisters.
     * 
     * # Arguments
     * 
     * * `index` - The index to get the value from.
     * 
     * # Returns
     * 
     * * `u8` - The value at the index.
     */
    fn index(&self, index: RegisterIndex) -> &Self::Output {
        // Safety: We know that the index is valid because we checked it in the constructor
        unsafe { &*self.registers.as_ptr().add(index.0 as usize) }
    }
}

impl std::ops::IndexMut<RegisterIndex> for VRegisters {
    /**
     * Indexes the VRegisters.
     * 
     * # Arguments
     * 
     * * `index` - The index to get the value from.
     * 
     * # Returns
     * 
     * * `u8` - The value at the index.
     */
    fn index_mut(&mut self, index: RegisterIndex) -> &mut Self::Output {
        // Safety: We know that the index is valid because we checked it in the constructor
        unsafe { &mut *self.registers.as_mut_ptr().add(index.0 as usize) }
    }
}

// Impl index range for VRegisters
impl std::ops::Index<std::ops::RangeInclusive<RegisterIndex>> for VRegisters {
    type Output = [u8];

    /**
     * Indexes the VRegisters.
     * 
     * # Arguments
     * 
     * * `index` - The index to get the value from.
     * 
     * # Returns
     * 
     * * `u8` - The value at the index.
     */
    fn index(&self, index: std::ops::RangeInclusive<RegisterIndex>) -> &Self::Output {
        // Safety: We know that the index is valid because we checked it in the constructor
        unsafe {
            std::slice::from_raw_parts(
                self.registers.as_ptr().add(index.start().0 as usize),
                index.end().0 as usize - index.start().0 as usize,
            )
        }
    }
}

impl std::ops::IndexMut<std::ops::RangeInclusive<RegisterIndex>> for VRegisters {
    /**
     * Indexes the VRegisters.
     * 
     * # Arguments
     * 
     * * `index` - The index to get the value from.
     * 
     * # Returns
     * 
     * * `u8` - The value at the index.
     */
    fn index_mut(&mut self, index: std::ops::RangeInclusive<RegisterIndex>) -> &mut Self::Output {
        // Safety: We know that the index is valid because we checked it in the constructor
        unsafe {
            std::slice::from_raw_parts_mut(
                self.registers.as_mut_ptr().add(index.start().0 as usize),
                index.end().0 as usize - index.start().0 as usize,
            )
        }
    }
}