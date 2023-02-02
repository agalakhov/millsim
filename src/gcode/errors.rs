//! G-Code processing errors

use std::fmt;

/// Simple error message from bottom level
#[derive(Debug)]
pub struct SimpleError(pub String);

impl SimpleError {
    /// Accompany `SimpleError` with line number
    pub fn at_line(self, line: u64) -> LineError {
        LineError { error: self, line }
    }
}

impl fmt::Display for SimpleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Error: {}", self.0)
    }
}

/// Error message with line number
#[derive(Debug)]
pub struct LineError {
    error: SimpleError,
    line: u64,
}

impl fmt::Display for LineError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "At line {}", self.line)?;
        self.error.fmt(f)
    }
}
