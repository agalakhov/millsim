//! Program checker and decoder

use super::actions::{Command, Global};
use crate::gcode::{
    errors::{LineError, SimpleError},
    words::{MWord, Word, Words},
    GCodeFile, Line,
};
use std::{collections::BTreeMap, fmt};

#[derive(Debug)]
struct CodeLine {
    file_line: u64,
    words: Words,
}

impl CodeLine {
    fn executable_code(&self) -> impl Iterator<Item = Word> + '_ {
        self.words.0.iter().filter(|w| w.is_executable()).cloned()
    }
}

#[derive(Debug)]
struct CodeBlock {
    file_line: u64,
    code: Vec<CodeLine>,
}

/// Decoded program
#[derive(Debug)]
pub struct Program {
    main_programs: BTreeMap<u8, CodeBlock>,
    sub_programs: BTreeMap<u8, CodeBlock>,
}

impl Program {
    pub fn from_file(file: GCodeFile) -> Result<Self, LineError> {
        enum Prog {
            Unknown,
            Main(u8),
            Sub(u8),
        }
        let mut program = Prog::Unknown;
        let mut main_programs = BTreeMap::<u8, CodeBlock>::new();
        let mut sub_programs = BTreeMap::<u8, CodeBlock>::new();
        let mut def_line = 0;
        for (file_line, code_line) in file.into_code() {
            let code = match code_line {
                Line::Empty => continue,
                Line::MainProgram(n) => {
                    def_line = file_line;
                    program = Prog::Main(n);
                    continue;
                }
                Line::SubProgram(n) => {
                    def_line = file_line;
                    program = Prog::Sub(n);
                    continue;
                }
                Line::Code(words) => CodeLine { file_line, words },
            };

            let entry = match program {
                Prog::Unknown => {
                    return Err(SimpleError("Code line with no program".into()).at_line(file_line))
                }
                Prog::Main(n) => main_programs.entry(n),
                Prog::Sub(n) => sub_programs.entry(n),
            };

            entry
                .or_insert(CodeBlock {
                    file_line: def_line,
                    code: Vec::new(),
                })
                .code
                .push(code);
        }

        check_last_executable(&main_programs, ProgramType::Main)?;
        check_last_executable(&sub_programs, ProgramType::Sub)?;

        Ok(Program {
            main_programs,
            sub_programs,
        })
    }

    pub fn execute(&self, idx: Option<u8>) -> Result<Executor, SimpleError> {
        (if let Some(idx) = idx {
            self.main_programs
                .get(&idx)
                .ok_or_else(|| SimpleError(format!("Program %{idx} not found")))
        } else {
            self.main_programs
                .first_key_value()
                .ok_or_else(|| SimpleError("No main programs found".into()))
                .map(|(_k, v)| v)
        })
        .map(|p| &p.code[..])
        .map(|p| Executor::start(&self.sub_programs, p))
    }
}

#[derive(Debug)]
struct StackItem<'t> {
    repeats: u16,
    code: &'t [CodeLine],
    full_code: &'t [CodeLine],
}

impl<'t> StackItem<'t> {
    fn new(code: &'t [CodeLine], repeats: u16) -> Self {
        Self {
            code,
            repeats,
            full_code: code,
        }
    }
}

/// Iterator over executable statements
#[derive(Debug)]
pub struct Executor<'t> {
    stack: Vec<StackItem<'t>>,
    sub_programs: &'t BTreeMap<u8, CodeBlock>,
}

impl<'t> Executor<'t> {
    fn start(sub_programs: &'t BTreeMap<u8, CodeBlock>, code: &'t [CodeLine]) -> Self {
        Self {
            stack: vec![StackItem::new(code, 0)],
            sub_programs,
        }
    }

    fn exec(&mut self, line: &CodeLine) -> Result<Command, SimpleError> {
        let cmd = Command::from_gcode(&line.words.0)?;

        if let Some(g) = &cmd.global {
            match g {
                Global::CallSub(n) => {
                    let sub = self
                        .sub_programs
                        .get(n)
                        .ok_or_else(|| SimpleError(format!("Subroutine L{n} not found")))?;
                    let repeats = cmd.p.ok_or(SimpleError(format!(
                        "Repeats count for subroutine L{n} not defined"
                    )))?;
                    self.stack.push(StackItem::new(&sub.code, repeats));
                    Ok(cmd)
                }
                Global::ReturnSub => {
                    if self.stack.len() <= 1 {
                        Err(SimpleError(
                            "Subroutine return (M17) without subroutine call".into(),
                        ))
                    } else if !self
                        .stack
                        .last()
                        .expect("Bug: stack is empty")
                        .code
                        .is_empty()
                    {
                        Err(SimpleError(
                            "Subroutine return (M17) is not the last statement".into(),
                        ))
                    } else {
                        let p = self.stack.pop().expect("Bug: popping from empty stack");
                        if p.repeats > 0 {
                            let repeats = p.repeats - 1;
                            self.stack.push(StackItem::new(p.full_code, repeats));
                        }
                        Ok(cmd)
                    }
                }
                Global::EndProgram => {
                    if self.stack.len() > 1 {
                        Err(SimpleError("Program end (M2) in a subroutine".into()))
                    } else if !self
                        .stack
                        .last()
                        .expect("Bug: stack is empty")
                        .code
                        .is_empty()
                    {
                        Err(SimpleError(
                            "Program end (M2) is not the last statement".into(),
                        ))
                    } else {
                        Ok(cmd)
                    }
                }
            }
        } else {
            Ok(cmd)
        }
    }
}

impl Iterator for Executor<'_> {
    type Item = Result<(u64, Command), LineError>;

    #[allow(unstable_name_collisions)] // TODO for take_first() - remove as it gets stabilized
    fn next(&mut self) -> Option<Self::Item> {
        let code = self
            .stack
            .last_mut()
            .expect("Bug: execution stack is empty")
            .code
            .take_first()?;
        Some(
            self.exec(code)
                .map(|c| (code.file_line, c))
                .map_err(|e| e.at_line(code.file_line)),
        )
    }
}

// TODO remove as slice::take_first() gets stabilized
trait TakeFirst<T> {
    fn take_first<'t>(self: &mut &'t Self) -> Option<&'t T>;
}

impl<T> TakeFirst<T> for [T] {
    fn take_first<'t>(self: &mut &'t Self) -> Option<&'t T> {
        let (first, rem) = self.split_first()?;
        *self = rem;
        Some(first)
    }
}

enum ProgramType {
    Main,
    Sub,
}

impl ProgramType {
    fn final_word(&self) -> Word {
        use ProgramType::*;
        Word::M(match self {
            Main => MWord::M2,
            Sub => MWord::M17,
        })
    }
}

impl fmt::Display for ProgramType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ProgramType::*;
        let s = match self {
            Main => "Main program",
            Sub => "Subprogram",
        };
        s.fmt(f)
    }
}

fn check_last_executable(
    programs: &BTreeMap<u8, CodeBlock>,
    ty: ProgramType,
) -> Result<(), LineError> {
    for (p, code) in programs {
        let c = code
            .code
            .iter()
            .rev()
            .map(|line| line.executable_code().collect())
            .skip_while(Vec::is_empty)
            .take(1)
            .last();

        let w = ty.final_word();
        if c != Some(vec![w.clone()]) {
            return Err(
                SimpleError(format!("{ty} #{p} does not end with {w}")).at_line(code.file_line)
            );
        }
    }

    Ok(())
}
