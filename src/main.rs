mod errors;
mod gcode;
mod machine;
mod render;
mod types;

use errors::{LineError, SimpleError};
use gcode::GCodeFile;
use machine::{Machine, Program};
use render::{svg::Svg, Render};
use std::{io::Write, path::Path};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

fn run(path: impl AsRef<Path>, out_path: Option<impl AsRef<Path>>) -> Result<(), LineError> {
    let file = GCodeFile::load(path)?;

    let program = Program::from_file(file)?;
    let render = out_path.map(|p| Box::new(Svg::new(p)) as Box<dyn Render>);

    let mut machine = Machine::with_render(render);
    for cmd in program.execute(None).map_err(SimpleError::no_line)? {
        let (line, cmd) = cmd?;
        println!("{}", cmd.raw);
        machine.execute_command(cmd).map_err(|e| e.at_line(line))?;
    }

    if let Some(render) = machine.finalize() {
        render
            .finalize()
            .map_err(|e| SimpleError(format!("Can't write output file: {e}")).no_line())?;
    }

    Ok(())
}

fn main() {
    let file = "bremse.ngc";
    let out = "bremse.svg";

    if let Err(e) = run(file, Some(out)) {
        let mut stderr = StandardStream::stderr(ColorChoice::Auto);
        stderr
            .set_color(
                ColorSpec::new()
                    .set_fg(Some(Color::Red))
                    .set_bold(true)
                    .set_intense(true),
            )
            .ok();
        writeln!(stderr, "While parsing '{file}':").ok();
        writeln!(stderr, "{e}").ok();
        stderr.reset().ok();
    }
}
