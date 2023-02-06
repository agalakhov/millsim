mod gcode;
mod machine;

use gcode::{errors::SimpleError, GCodeFile, LineError};
use machine::{Machine, Program};
use std::path::Path;

fn run(path: impl AsRef<Path>) -> Result<(), LineError> {
    let file = GCodeFile::load(path)?;

    let program = Program::from_file(file)?;

    let mut machine = Machine::default();
    for cmd in program.execute(None).map_err(SimpleError::no_line)? {
        let (line, cmd) = cmd?;
        println!("{}", cmd.raw);
        machine.execute_command(cmd).map_err(|e| e.at_line(line))?;
    }

    Ok(())
}

fn main() {
    let file = "bremse.ngc";

    if let Err(e) = run(file) {
        eprintln!("While parsing '{file}':");
        eprintln!("{e}");
    }
}
