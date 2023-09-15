use super::Address;

/// http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#2.2
/// The stack size is traditionally 16 (`0x10`).
pub const STACK_SIZE: usize = 0x10;

/// The `Stack` struct represents a stack data structure for storing `Address` values that are the return point on call instructions.
pub struct Stack {
    array: [Address; STACK_SIZE],
    top: usize,
}

impl Stack {
    /// Creates a new `Stack` with all elements initialized to 0 and the top of the stack pointing to the first position.
    pub fn new() -> Stack {
        Stack {
            array: [0; STACK_SIZE],
            top: 0,
        }
    }
    /// Pushes an item onto the stack. Returns `Err(RuntimeError::StackOverFlow)` if the stack is full.
    pub fn push(&mut self, item: Address) -> Result<(), super::error::RuntimeError> {
        if self.top >= STACK_SIZE {
            Err(super::error::RuntimeError::StackOverFlow)
        } else {
            self.array[self.top as usize] = item;
            self.top += 1;
            Ok(())
        }
    }
    /// Pops an item from the stack. Returns `Err(RuntimeError::StackUnderFlow)` if the stack is empty.
    pub fn pop(&mut self) -> Result<Address, super::error::RuntimeError> {
        if self.top == 0 {
            Err(super::error::RuntimeError::StackUnderFlow)
        } else {
            self.top -= 1;
            Ok(self.array[self.top as usize])
        }
    }

    pub fn clear(&mut self) {
        self.top = 0;
    }
}
