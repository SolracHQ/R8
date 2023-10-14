/// http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#2.2
/// The chip-8 stack size is traditionally 16 (`0x10`).
pub const STACK_SIZE: usize = 0x10;

/// The `Stack` struct represents a stack data structure for storing `Address` values that are the return point on call instructions.
///
/// # Fields
///
/// * `array` - The array that stores the `Address` values.
/// * `top` - The top of the stack.
///
/// # Type Parameters
///
/// * `T` - The type of the items to store on the stack.
///
/// # Notes
///
/// It is generic to facilite testing.
pub struct Stack<T: Copy + Default> {
    array: [T; STACK_SIZE],
    top: usize,
}

impl<T> Stack<T>
where
    T: Copy + Default,
{
    /// Creates a new `Stack` with all elements initialized to 0 and the top of the stack pointing to the first position.
    ///
    /// # Returns
    ///
    /// * `Stack<T>` - The new stack.
    pub fn new() -> Self {
        Self {
            array: [T::default(); STACK_SIZE],
            top: 0,
        }
    }

    /// Pushes an item onto the stack.
    ///
    /// # Arguments
    ///
    /// * `item` - The item to push onto the stack.
    ///
    /// # Returns
    ///
    /// * `Result<(), RuntimeError>` - Returns Ok if the item was pushed onto the stack, otherwise returns an error.
    pub fn push(&mut self, item: T) -> Result<(), super::error::EmulatorError> {
        if self.top >= STACK_SIZE {
            Err(super::error::EmulatorError::StackOverFlow)
        } else {
            self.array[self.top] = item;
            self.top += 1;
            Ok(())
        }
    }

    /// Pops an item from the stack.
    ///
    /// # Returns
    ///
    /// * `Result<T, RuntimeError>` - Returns Ok if the item was popped from the stack, otherwise returns an error.
    pub fn pop(&mut self) -> Result<T, super::error::EmulatorError> {
        if self.top == 0 {
            Err(super::error::EmulatorError::StackUnderFlow)
        } else {
            self.top -= 1;
            Ok(self.array[self.top])
        }
    }

    /// Returns the number of items on the stack.
    ///
    /// # Returns
    ///
    /// * `usize` - The number of items on the stack.
    pub fn len(&self) -> usize {
        self.top
    }

    /// Clears the stack by setting the top of the stack to 0.
    pub fn clear(&mut self) {
        self.top = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let stack: Stack<u8> = Stack::new();
        assert!(stack.top == 0);
        assert!(stack.array.iter().all(|&x| x == 0));
    }

    #[test]
    fn test_push() {
        let mut stack = Stack::new();
        assert!(matches!(stack.push(1), Ok(())));
        assert!(stack.top == 1);
        assert!(stack.array[0] == 1);
    }

    #[test]
    fn test_push_overflow() {
        let mut stack = Stack::new();
        for i in 0..STACK_SIZE {
            assert!(matches!(stack.push(i), Ok(())));
        }
        assert!(matches!(
            stack.push(100),
            Err(super::super::error::EmulatorError::StackOverFlow)
        ));
    }

    #[test]
    fn test_pop() {
        let mut stack = Stack::new();
        stack.push(1).unwrap();
        assert!(matches!(stack.pop(), Ok(1)));
        assert!(stack.top == 0);
    }

    #[test]
    fn test_pop_underflow() {
        let mut stack: Stack<()> = Stack::new();
        assert!(matches!(
            stack.pop(),
            Err(super::super::error::EmulatorError::StackUnderFlow)
        ));
    }

    #[test]
    fn test_clear() {
        let mut stack = Stack::new();
        for i in 0..5 {
            stack.push(i).unwrap();
        }
        stack.clear();
        assert!(stack.top == 0);
    }
}
