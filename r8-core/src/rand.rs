use std::num::Wrapping;

/// Function to get the current time in microseconds since UNIX_EPOCH
///
/// # Returns
///
/// * `u128` - The current time in microseconds since UNIX_EPOCH
fn get_epoch_micros() -> u128 {
  std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .map_or(5555u128, |d| d.as_micros())
}

/// Struct to represent a pseudo-random number generator
///
/// # Fields
///
/// * `multiplier` - The multiplier for the linear congruential generator
/// * `increment` - The increment for the linear congruential generator
/// * `modulus` - The modulus for the linear congruential generator
pub struct RandGen {
  multiplier: Wrapping<u128>,
  increment: Wrapping<u128>,
  modulus: Wrapping<u128>,
  state: Wrapping<u128>,
}

impl RandGen {
  /// Function to initialize a new instance of RandGen
  ///
  /// # Returns
  ///
  /// * `RandGen` - The new instance of RandGen
  pub fn new() -> Self {
    let seed = get_epoch_micros(); // Using the current time as seed
    Self {
      multiplier: Wrapping(6364136223846793005),
      increment: Wrapping(1442695040888963407),
      modulus: Wrapping(u128::MAX),
      state: Wrapping(seed), // Initial state X_0 is set to the seed
    }
  }

  /// Function to get the next random number
  ///
  /// # Returns
  ///
  /// * `u8` - The next random number
  pub fn next(&mut self) -> u8 {
    self.state = (self.multiplier * self.state + self.increment) % self.modulus;
    (self.state.0 >> 56) as u8
  }
}
