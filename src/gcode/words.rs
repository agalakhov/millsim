//! G-Code words

use super::{errors::SimpleError, types::Micrometer};
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

