//! G-Code parser

use super::{
    errors::SimpleError,
    types::Micrometer,
    words::{GWord, MWord, Word, Words},
};
use nom::{
    branch::alt,
    bytes::complete::{is_a, is_not, tag},
    character::complete::{char, u16, u32, u8},
    combinator::{all_consuming, map, map_res, opt, value},
    multi::many1,
    sequence::{delimited, preceded, separated_pair},
    IResult,
};
use std::fmt;

#[derive(Debug, Clone)]
pub enum Line {
    /// Empty line with no code
    Empty,
    /// Main program "%MPF" designator
    MainProgram(u8),
    /// Sub program "%SPF" designator
    SubProgram(u8),
    /// Code line
    Code(Words),
}

impl Line {
    /// Parse program text line
    pub fn parse(line: &str) -> Result<Line, SimpleError> {
        parse_codes(line).map(|(_, l)| l).map_err(|e| {
            use nom::Err::*;
            SimpleError(match e {
                Incomplete(_) => "Incomplete data".into(),
                Error(e) | Failure(e) => format!("Invalid syntax at '{}'", e.input),
            })
        })
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Line::*;
        match self {
            Empty => Ok(()),
            MainProgram(x) => write!(f, "%MPF{x}"),
            SubProgram(x) => write!(f, "%SPF{x}"),
            Code(v) => v.fmt(f),
        }
    }
}

fn parse_codes(line: &str) -> IResult<&str, Line> {
    let words = (
        map_res(preceded(char('G'), u8), |n| {
            GWord::from_number(n).map(Word::G)
        }),
        map_res(preceded(char('M'), u8), |n| {
            MWord::from_number(n).map(Word::M)
        }),
        map(preceded(char('X'), Micrometer::parse), Word::X),
        map(preceded(char('Y'), Micrometer::parse), Word::Y),
        map(preceded(char('Z'), Micrometer::parse), Word::Z),
        map(preceded(char('I'), Micrometer::parse), Word::I),
        map(preceded(char('J'), Micrometer::parse), Word::J),
        map(preceded(char('N'), u32), Word::N),
        map(preceded(char('S'), u16), Word::S),
        map(preceded(char('F'), u16), Word::F),
        map(preceded(char('L'), u8), Word::L),
        map(preceded(char('P'), u16), Word::P),
        map(preceded(char('D'), u8), Word::D),
        map(
            preceded(char('R'), separated_pair(u8, char('='), Micrometer::parse)),
            |(a, b)| Word::R(a, b),
        ),
        map(delimited(char('('), is_not(")"), opt(char(')'))), |s| {
            Word::Comment(String::from(s))
        }),
    );

    all_consuming(alt((
        map(delimited(tag("%MPF"), u8, spc), Line::MainProgram),
        map(delimited(tag("%SPF"), u8, spc), Line::SubProgram),
        map(many1(delimited(spc, alt(words), spc)), |c| {
            Line::Code(Words(c))
        }),
        value(Line::Empty, spc),
    )))(line)
}

fn spc(s: &str) -> IResult<&str, &str> {
    map(opt(is_a(" ")), |x| x.unwrap_or(""))(s)
}

#[cfg(test)]
mod tests {
    use super::Line;

    #[test]
    fn parse_g() {
        let s = "G0 G1 G2KG3 X15 Y60";
        eprintln!("{:?}", Line::parse(s));
    }
}
