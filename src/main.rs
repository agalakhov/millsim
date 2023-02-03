mod gcode;

use std::path::Path;
use gcode::{GCodeFile, LineError};

fn run(path: impl AsRef<Path>) -> Result<(), LineError> {
    let file = GCodeFile::load(path)?;

    Ok(())
}

fn main() {
    let file = "bremse.ngc";

    if let Err(e) = run(file) {
        eprintln!("While parsing '{file}':");
        eprintln!("{e}");
    }
}
