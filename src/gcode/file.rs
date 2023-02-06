//! G-code file parser

use super::{
    errors::{LineError, SimpleError},
    parser::Line,
};
use std::{
    fmt,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

/// Parsed G-Code file
pub struct GCodeFile {
    code: Vec<Line>,
}

impl GCodeFile {
    /// Load file from disk
    pub fn load(path: impl AsRef<Path>) -> Result<Self, LineError> {
        let fd =
            File::open(path).map_err(|e| SimpleError(format!("Can't open file: {e}")).no_line())?;
        let fd = BufReader::new(fd);

        let code: Result<_, LineError> = fd
            .lines()
            .enumerate()
            .map(|(no, line)| {
                let no = no as u64 + 1;
                line.map_err(|e| SimpleError(format!("I/O error {e}")).at_line(no))
                    .and_then(|line| Line::parse(&line).map_err(|e| e.at_line(no)))
            })
            .collect();
        let code = code?;

        Ok(Self { code })
    }

    /// Iterate over file contents
    pub fn code(&self) -> impl Iterator<Item = (u64, &Line)> {
        self.code
            .iter()
            .enumerate()
            .map(|(no, line)| (no as u64 + 1, line))
    }

    /// Iterate over file contents, consume file
    pub fn into_code(self) -> impl Iterator<Item = (u64, Line)> {
        self.code
            .into_iter()
            .enumerate()
            .map(|(no, line)| (no as u64 + 1, line))
    }

    /// Make printable version of code
    pub fn printable(&self) -> Printable {
        Printable(self)
    }
}

/// Printable version of G-Code file
pub struct Printable<'t>(&'t GCodeFile);

impl fmt::Display for Printable<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (_, line) in self.0.code() {
            writeln!(f, "{line}")?;
        }
        Ok(())
    }
}
