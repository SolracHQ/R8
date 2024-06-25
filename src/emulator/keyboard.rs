#[repr(transparent)]
#[derive(Default)]
/// http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#keyboard
/// Represents the keyboard of the Chip8 system as a bitmask.
pub struct KeyBoard(u16);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Represents the keys on the Chip8 keyboard
pub enum Key {
    K0 = 0x0,
    K1 = 0x1,
    K2 = 0x2,
    K3 = 0x3,
    K4 = 0x4,
    K5 = 0x5,
    K6 = 0x6,
    K7 = 0x7,
    K8 = 0x8,
    K9 = 0x9,
    KA = 0xA,
    KB = 0xB,
    KC = 0xC,
    KD = 0xD,
    KE = 0xE,
    KF = 0xF,
}

impl Key {
    pub fn all() -> core::slice::Iter<'static, Key> {
        const KEYS: [Key; 16] = [
            Key::K0,
            Key::K1,
            Key::K2,
            Key::K3,
            Key::K4,
            Key::K5,
            Key::K6,
            Key::K7,
            Key::K8,
            Key::K9,
            Key::KA,
            Key::KB,
            Key::KC,
            Key::KD,
            Key::KE,
            Key::KF,
        ];
        KEYS.iter()
    }
}

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
