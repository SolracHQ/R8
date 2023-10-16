#[repr(transparent)]
pub struct Timer(u8);

impl Timer {
    /// Creates a new `Timer` with a value of 0.
    ///
    /// # Returns
    ///
    /// * `Timer` - The newly created timer.
    pub fn new() -> Self {
        Self(0)
    }

    /// Returns the value of the timer.
    ///
    /// # Returns
    ///
    /// * `u8` - The value of the timer.
    pub fn get(&self) -> u8 {
        self.0
    }

    /// Sets the value of the timer.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to set the timer to.
    pub fn set(&mut self, value: u8) {
        self.0 = value;
    }

    /// Decrements the timer by 1.
    pub fn decrement(&mut self) {
        if self.0 > 0 {
            self.0 -= 1;
        }
    }
}