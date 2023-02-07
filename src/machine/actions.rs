//! Actions and machine commands

use crate::{
    errors::SimpleError,
    gcode::words::{GWord, MWord, Word, Words},
    types::Micrometer,
};
use std::fmt;
use strum::Display;

#[derive(Debug, Default)]
pub struct Command {
    pub global: Option<Global>,
    pub movement: Option<Movement>,

    pub spindle_action: Option<SpindleAction>,
    pub water_action: Option<WaterAction>,
    pub coord_switch: Option<CoordSwitch>,

    pub raw_x: Option<Micrometer>,
    pub raw_y: Option<Micrometer>,
    pub raw_z: Option<Micrometer>,
    pub i: Option<Micrometer>,
    pub j: Option<Micrometer>,

    pub speed: Option<u16>,
    pub feed: Option<u16>,
    pub tool: Option<u8>,

    pub n: Option<u32>,
    pub p: Option<u16>,

    pub comment: String,

    pub raw: Words,
}

fn is_builtin(l: u8) -> bool {
    l >= 80
}

impl Command {
    pub fn from_gcode(gcode: &[Word]) -> Result<Self, SimpleError> {
        let mut cmd = Self::default();

        for word in gcode {
            cmd.raw.0.push(word.clone());

            use GWord::*;
            use MWord::*;
            use Word::*;
            match word {
                L(n) if is_builtin(*n) => cmd.movement.set(Movement::BuiltinCycle(*n))?,
                L(n) => cmd.global.set(Global::CallSub(*n))?,
                N(n) => cmd.n.setn("N[umber]", *n)?,
                Comment(s) => cmd.comment.push_str(s),
                R(a, b) => (), //unimplemented!(),

                M(M2) => cmd.global.set(Global::EndProgram)?,
                M(M17) => cmd.global.set(Global::ReturnSub)?,

                M(M6) => cmd.movement.set(Movement::ToolChange)?,

                G(G0) => cmd.movement.set(Movement::FastLine)?,
                G(G1) => cmd.movement.set(Movement::Line)?,
                G(G2) => cmd.movement.set(Movement::CircleCW)?,
                G(G3) => cmd.movement.set(Movement::CircleCCW)?,

                G(G90) => cmd.coord_switch.set(CoordSwitch::Absolute)?,
                G(G91) => cmd.coord_switch.set(CoordSwitch::Relative)?,

                M(M3) => cmd.spindle_action.set(SpindleAction::SpindleOnCW)?,
                M(M4) => cmd.spindle_action.set(SpindleAction::SpindleOnCCW)?,
                M(M5) => cmd.spindle_action.set(SpindleAction::SpindleOff)?,

                M(M8) => cmd.water_action.set(WaterAction::WaterOn)?,
                M(M9) => cmd.water_action.set(WaterAction::WaterOff)?,

                S(n) => cmd.speed.setn("S[peed]", *n)?,
                F(n) => cmd.feed.setn("F[eed]", *n)?,
                D(n) => cmd.tool.setn("D (tool)", *n)?,

                X(n) => cmd.raw_x.setn("X", *n)?,
                Y(n) => cmd.raw_y.setn("Y", *n)?,
                Z(n) => cmd.raw_z.setn("Z", *n)?,
                I(n) => cmd.i.setn("I (center X)", *n)?,
                J(n) => cmd.j.setn("J (center Y)", *n)?,

                P(n) => cmd.p.setn("P (repeat count)", *n)?,
            }
        }

        Ok(cmd)
    }
}

trait SetOrError<T: Sized + fmt::Display> {
    fn set(&mut self, value: T) -> Result<(), SimpleError>;
    fn setn(&mut self, name: &str, value: T) -> Result<(), SimpleError>;
}

impl<T: Sized + fmt::Display> SetOrError<T> for Option<T> {
    fn set(&mut self, value: T) -> Result<(), SimpleError> {
        match self {
            None => {
                *self = Some(value);
                Ok(())
            }
            Some(old) => Err(SimpleError(format!(
                "Double command: '{old}' and '{value}'"
            ))),
        }
    }

    fn setn(&mut self, name: &str, value: T) -> Result<(), SimpleError> {
        match self {
            None => {
                *self = Some(value);
                Ok(())
            }
            Some(old) => Err(SimpleError(format!(
                "Double '{name}' command: '{old}' and '{value}'"
            ))),
        }
    }
}

#[derive(Debug, Display)]
pub enum Global {
    #[strum(serialize = "L (subroutine call)")]
    CallSub(u8),
    #[strum(serialize = "M17 (subroutine return)")]
    ReturnSub,
    #[strum(serialize = "M2 (program end)")]
    EndProgram,
}

#[derive(Debug, Display)]
pub enum Movement {
    #[strum(serialize = "G0 (fast move)")]
    FastLine,
    #[strum(serialize = "G1 (linear move)")]
    Line,
    #[strum(serialize = "G2 (circular move CW)")]
    CircleCW,
    #[strum(serialize = "G3 (circular move CCW)")]
    CircleCCW,
    #[strum(serialize = "M6 (tool change)")]
    ToolChange,
    #[strum(serialize = "L (builtin subroutine)")]
    BuiltinCycle(u8),
}

#[derive(Debug, Display)]
pub enum SpindleAction {
    #[strum(serialize = "M3 (spindle on CW)")]
    SpindleOnCW,
    #[strum(serialize = "M4 (spindle on CCW)")]
    SpindleOnCCW,
    #[strum(serialize = "M5 (spindle off)")]
    SpindleOff,
}

#[derive(Debug, Display)]
pub enum WaterAction {
    #[strum(serialize = "M8 (coolant on)")]
    WaterOn,
    #[strum(serialize = "M9 (coolant off)")]
    WaterOff,
}

#[derive(Debug, Display)]
pub enum CoordSwitch {
    #[strum(serialize = "G90 (absolute coordinates)")]
    Absolute,
    #[strum(serialize = "G91 (relative coordinates)")]
    Relative,
}
