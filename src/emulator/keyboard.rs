#[repr(transparent)]
#[derive(Default)]
/// http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#keyboard
pub struct KeyBoard(u16);

impl KeyBoard {
    pub fn set(&mut self, key: u8) {
        self.0 |= 1 << key
    }

    pub fn unset(&mut self, key: u8) {
        self.0 &= !(1 << key)
    }

    pub fn is_set(&self, key: u8) -> bool {
        (self.0 >> key) & 1 == 1
    }
}
