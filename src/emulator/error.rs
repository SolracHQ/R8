#[derive(Debug)]
pub enum RuntimeError {
    LoadError(std::io::Error),
    StackOverFlow,
    StackUnderFlow,
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::StackOverFlow => write!(
                f,
                "Stack Overflow: Unable to push item, the stack is already full."
            ),
            RuntimeError::StackUnderFlow => write!(
                f,
                "Stack Underflow: Unable to pop item, the stack is empty."
            ),
            RuntimeError::LoadError(e) => write!(f, "Cannot Load the ROM: {e}"),
        }
    }
}

impl std::error::Error for RuntimeError {}
