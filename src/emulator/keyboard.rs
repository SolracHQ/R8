#[repr(transparent)]
#[derive(Default)]
/// http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#keyboard
/// Represents the keyboard of the Chip8 system as a bitmask.
/// 
/// The original implementation of the Chip8 system had a 16-key hexadecimal keypad with the following layout:
/// 
/// | 1 | 2 | 3 | C |
/// |---|---|---|---|
/// | 4 | 5 | 6 | D |
/// | 7 | 8 | 9 | E |
/// | A | 0 | B | F |
/// 
/// The keys are mapped to the following indexes:
/// 
/// | 1 | 2 | 3 | 4 |
/// |---|---|---|---|
/// | Q | W | E | R |
/// | A | S | D | F |
/// | Z | X | C | V |
pub struct KeyBoard(u16);

impl KeyBoard {

    /// Set the key at the given index
    /// 
    /// # Arguments
    /// 
    /// * `key` - The index of the key to set
    pub fn set(&mut self, key: u8) {
        self.0 |= 1 << key
    }

    /// Unset the key at the given index
    /// 
    /// # Arguments
    /// 
    /// * `key` - The index of the key to unset
    pub fn unset(&mut self, key: u8) {
        self.0 &= !(1 << key)
    }

    /// Check if the key at the given index is set
    /// 
    /// # Arguments
    /// 
    /// * `key` - The index of the key to check
    /// 
    /// # Returns
    /// 
    /// * `bool` - Returns true if the key is set, otherwise returns false
    pub fn is_set(&self, key: u8) -> bool {
        (self.0 >> key) & 1 == 1
    }
}
