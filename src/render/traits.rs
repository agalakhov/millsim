//! Rendering traits

pub use crate::types::Micrometer;
use std::{fmt::Debug, io::Error};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Circle {
    Cw,
    Ccw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Line {
    Fast,
    Cut,
}

pub trait Render: Debug {
    fn line_to(
        &mut self,
        tool: Micrometer,
        ty: Line,
        point: (Micrometer, Micrometer),
        height: Micrometer,
    );

    fn arc_to(
        &mut self,
        tool: Micrometer,
        ty: Circle,
        center: (Micrometer, Micrometer),
        end: (Micrometer, Micrometer),
    );

    fn finalize(self: Box<Self>) -> Result<(), Error>;
}
