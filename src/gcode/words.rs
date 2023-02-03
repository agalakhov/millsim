//! G-Code words

use super::{errors::SimpleError, types::Micrometer};
use std::fmt;
use strum::FromRepr;

/// All supported code words
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Word {
    /// N line number
    N(u32),
    /// Gxxx commands
    G(GWord),
    /// Mxxx commands
    M(MWord),
    /// D tool number
    D(u8),
    /// S spindle speed
    S(u16),
    /// F milling feed
    F(u16),
    /// I coordinate
    I(Micrometer),
    /// J coordinate
    J(Micrometer),
    /// X coordinate
    X(Micrometer),
    /// Y coordinate
    Y(Micrometer),
    /// Z coordinate
    Z(Micrometer),
    /// L subprogram call
    L(u16),
    /// P subprogram counter
    P(u16),
    /// R parameter
    R(u8, Micrometer),
    /// String comment
    Comment(String),
}

impl fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Word::*;
        match self {
            N(x) => write!(f, "N{x}"),
            G(w) => w.fmt(f),
            M(w) => w.fmt(f),
            D(x) => write!(f, "D{x}"),
            S(x) => write!(f, "S{x}"),
            F(x) => write!(f, "F{x}"),
            I(x) => write!(f, "I{x}"),
            J(x) => write!(f, "J{x}"),
            X(x) => write!(f, "X{x}"),
            Y(x) => write!(f, "Y{x}"),
            Z(x) => write!(f, "Z{x}"),
            L(x) => write!(f, "L{x}"),
            P(x) => write!(f, "P{x}"),
            R(x, y) => write!(f, "P{x}={y}"),
            Comment(c) => write!(f, "({c})"),
        }
    }
}

/// All supported G codes
#[derive(Debug, Clone, PartialEq, Eq, FromRepr)]
pub enum GWord {
    /// Fast feed
    G0 = 0,
    /// Linear feed
    G1 = 1,
    /// Clockwise circular feed
    G2 = 2,
    /// Counter-clockwise circular feed
    G3 = 3,
    /// Use absolute coordinates
    G90 = 90,
    /// Use relative coordinates
    G91 = 91,
}

impl GWord {
    /// Convert integer designator to G code number
    pub fn from_number(n: u8) -> Result<Self, SimpleError> {
        GWord::from_repr(n as usize).ok_or_else(|| SimpleError(format!("Unknown G code 'G{n}'")))
    }
}

impl fmt::Display for GWord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "G{}", *self as u8)
    }
}

/// All supported M codes
#[derive(Debug, Clone, PartialEq, Eq, FromRepr)]
pub enum MWord {
    /// Program end
    M2 = 2,
    /// Start spindle CW (forwards)
    M3 = 3,
    /// Start spindle CCW (backwards)
    M4 = 4,
    /// Stop spindle
    M5 = 5,
    /// Tool change
    M6 = 6,
    /// Start cooling
    M8 = 8,
    /// Stop cooling
    M9 = 9,
    /// Return from the subprogram
    M17 = 17,
}

impl MWord {
    /// Convert integer designator to M code number
    pub fn from_number(n: u8) -> Result<Self, SimpleError> {
        MWord::from_repr(n as usize).ok_or_else(|| SimpleError(format!("Unknown M code 'M{n}'")))
    }
}

impl fmt::Display for MWord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "M{}", *self as u8)
    }
}
