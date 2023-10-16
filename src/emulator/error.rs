/// Error types for the emulator.
///
/// This is a list of all the errors that can occur while running the emulator.
#[derive(Debug)]
pub enum EmulatorError {
    /// An error occurred while loading the ROM.
    LoadError(std::io::Error),
    /// The stack is full and cannot push any more items.
    StackOverFlow,
    /// The stack is empty and cannot pop any more items.
    StackUnderFlow,
    /// The address is not valid.
    InvalidAddress(u16),
    // The address is out of bounds.
    OutOfBounds(u16),
    /// The register is not valid.
    InvalidRegister(u8),
}

impl std::fmt::Display for EmulatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmulatorError::StackOverFlow => write!(
                f,
                "Stack Overflow: Unable to push item, the stack is already full."
            ),
            EmulatorError::StackUnderFlow => write!(
                f,
                "Stack Underflow: Unable to pop item, the stack is empty."
            ),
            EmulatorError::LoadError(e) => write!(f, "Cannot Load the ROM: {e}"),
            EmulatorError::InvalidAddress(address) => {
                write!(f, "Invalid Address: The address {address} is not valid.")
            }
            EmulatorError::OutOfBounds(end_address) => {
                write!(
                    f,
                    "Out of Bounds: The address {end_address} is out of bounds. [0x000, 0xFFF]"
                )
            }
            EmulatorError::InvalidRegister(x) => write!(
                f,
                "Invalid Register: The register {x} is not valid. [0x0, 0xF]")
        }
    }
}

impl std::error::Error for EmulatorError {}
