//! SVG rendering using Cairo

use super::traits::{Micrometer, Circle, Render};
use std::path::Path;
use cairo::{
    Error,
    SvgSurface,
};


/// SVG renderer using Cairo
#[derive(Debug)]
pub struct CairoSvg {
}

impl CairoSvg {
    pub fn open(file: impl AsRef<Path>) -> Result<Self, Error> {
        ш..let fd = SvgSurface::
    }
}

impl Render for CairoSvg {
    fn move_to(&mut self, point: (Micrometer, Micrometer), height: Micrometer) {
    }

    fn line_to(&mut self, point: (Micrometer, Micrometer), height: Micrometer) {
    }

    fn arc_to(
        &mut self,
        ty: Circle,
        center: (Micrometer, Micrometer),
        end: (Micrometer, Micrometer),
    ) {
    }
}
