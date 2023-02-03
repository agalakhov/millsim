//! G-code file parser

use super::{
    errors::{LineError, SimpleError},
    parser::Line,
};
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

/// Parsed G-Code file
pub struct GCodeFile {
    code: Vec<Line>,
}

impl GCodeFile {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, LineError> {
        let fd = File::open(path)
            .map_err(|e| SimpleError(format!("Can't open file: {e}")).no_line())?;
        let fd = BufReader::new(fd);

        let code: Result<_, LineError> = fd
            .lines()
            .enumerate()
            .map(|(no, line)| {
                let no = no as u64;
                line.map_err(|e| SimpleError(format!("I/O error {e}")).at_line(no))
                    .and_then(|line| Line::parse(&line).map_err(|e| e.at_line(no)))
            })
            .collect();
        let code = code?;

        Ok(Self { code })
    }

    pub fn code(&self) -> impl Iterator<Item = (u64, &Line)> {
        self.code
            .iter()
            .enumerate()
            .map(|(no, line)| (no as u64, line))
    }
}
