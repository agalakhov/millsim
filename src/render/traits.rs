//! Rendering traits

pub use crate::types::Micrometer;
use std::fmt::Debug;

#[derive(Debug, Clone, Copy)]
pub enum Circle {
    Cw,
    Ccw,
}

pub trait Render: Debug {
    fn move_to(&mut self, point: (Micrometer, Micrometer), height: Micrometer);
    fn line_to(&mut self, point: (Micrometer, Micrometer), height: Micrometer);

    fn arc_to(
        &mut self,
        ty: Circle,
        center: (Micrometer, Micrometer),
        end: (Micrometer, Micrometer),
    );
}
