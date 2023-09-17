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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let stack = Stack::new();
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
            assert!(matches!(stack.push(i as Address), Ok(())));
        }
        assert!(matches!(
            stack.push(100),
            Err(super::super::error::RuntimeError::StackOverFlow)
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
        let mut stack = Stack::new();
        assert!(matches!(
            stack.pop(),
            Err(super::super::error::RuntimeError::StackUnderFlow)
        ));
    }

    #[test]
    fn test_clear() {
        let mut stack = Stack::new();
        for i in 0..5 {
            stack.push(i as Address).unwrap();
        }
        stack.clear();
        assert!(stack.top == 0);
    }
}
