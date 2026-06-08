use std::fmt;

/// Errors produced during any season of the compiler pipeline.
#[derive(Debug, Clone, PartialEq)]
pub enum GroveError {
    /// Lexer error during Spring.
    Lex { message: String, pos: usize },
    /// Parser error during Spring.
    Parse { message: String, pos: usize },
    /// Type-check / scope error during Summer.
    TypeCheck { message: String },
    /// Optimizer error during Autumn.
    Optimize { message: String },
    /// Emission error during Winter.
    Emit { message: String },
}

impl fmt::Display for GroveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GroveError::Lex { message, pos } => write!(f, "lex error at {pos}: {message}"),
            GroveError::Parse { message, pos } => write!(f, "parse error at {pos}: {message}"),
            GroveError::TypeCheck { message } => write!(f, "type error: {message}"),
            GroveError::Optimize { message } => write!(f, "optimize error: {message}"),
            GroveError::Emit { message } => write!(f, "emit error: {message}"),
        }
    }
}

impl std::error::Error for GroveError {}
