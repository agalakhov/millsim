//! G-code file parser

use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};
use super::parser::Line;

/// Parsed G-Code file
pub struct GCodeFile {}

impl GCodeFile {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ()> {
        let mut res = Self {};

        let fd = File::open(path).expect("Can't open file");
        let fd = BufReader::new(fd);

        for line in fd.lines() {
            let line = line.unwrap();
            let line = Line::parse(&line);
            print!("\x1b[{}m", if line.is_ok() {
                "40"
            } else {
                "41"
            });
            println!("{line:?}\x1b[0m");
        }

        Ok(res)
    }
}
