//! The milling machine simulator

use super::actions::{Command, Movement};
use crate::gcode::{errors::SimpleError, Micrometer};

/// Machine configuration
#[derive(Debug)]
pub struct MachineConfig {
    safe_z: Micrometer,
    min_speed: u16,
    max_speed: u16,
    min_feed: u16,
    max_feed: u16,
}

impl Default for MachineConfig {
    fn default() -> Self {
        Self {
            safe_z: Micrometer(150_000),
            min_speed: 500,
            max_speed: 5000,
            min_feed: 10,
            max_feed: 400,
        }
    }
}

/// The machine simulator
#[derive(Debug, Default)]
pub struct Machine {
    cfg: MachineConfig,

    movement: Option<Movement>,

    x: Option<Micrometer>,
    y: Option<Micrometer>,
    z: Option<Micrometer>,
    speed: Option<u16>,
    feed: Option<u16>,
    tool: Option<u8>,
}

impl Machine {
    pub fn execute_command(&mut self, code: Command) -> Result<(), SimpleError> {
        self.speed.upd(code.speed);
        self.feed.upd(code.feed);
        self.tool.upd(code.tool);

        let new_move = self.movement.upd(code.movement);

        if let Some(mv) = &self.movement {
            match mv {
                Movement::FastLine => {
                    if new_move {
                        code.tool.prohibit("D")?;
                    }
                    code.i.prohibit("I")?;
                    code.j.prohibit("J")?;

                    if self.z.is_none() {
                        // No horizontal movement until Z is safe
                        code.x.prohibit("X")?;
                        code.y.prohibit("Y")?;
                        let z = code.z.require("Z")?;

                        if z != self.cfg.safe_z {
                            return Err(SimpleError(
                                "First movement should be to safe Z height".into(),
                            ));
                        }
                        // TODO

                        self.z = Some(z);
                    } else {
                        // TODO
                        self.x.upd(code.x);
                        self.y.upd(code.y);
                        self.z.upd(code.z);
                    }
                }
                Movement::Line => {
                    code.tool.prohibit("D")?;
                    code.i.prohibit("I")?;
                    code.j.prohibit("J")?;
                    self.prepare_cut()?;
                    // TODO
                    self.x.upd(code.x);
                    self.y.upd(code.y);
                    self.z.upd(code.z);
                }
                Movement::CircleCW => {
                    code.tool.prohibit("D")?;
                    code.z.prohibit("Z")?;
                    self.circle(
                        Circle::Cw,
                        code.i.require("I")?,
                        code.j.require("J")?,
                        code.x.require("X")?,
                        code.y.require("Y")?,
                    )?;
                }
                Movement::CircleCCW => {
                    code.tool.prohibit("D")?;
                    code.z.prohibit("Z")?;
                    self.circle(
                        Circle::Ccw,
                        code.i.require("I")?,
                        code.j.require("J")?,
                        code.x.require("X")?,
                        code.y.require("Y")?,
                    )?;
                }
                Movement::ToolChange => {
                    code.tool.require("D")?;
                    code.x.prohibit("X")?;
                    code.y.prohibit("Y")?;
                    code.z.prohibit("Z")?;
                    code.i.prohibit("I")?;
                    code.j.prohibit("J")?;

                    if self.z.unwrap_or(self.cfg.safe_z) < self.cfg.safe_z {
                        return Err(SimpleError(
                            "Must be high enough to perform tool change".into(),
                        ));
                    }

                    self.movement = None;
                    self.z = None;
                }
                Movement::BuiltinCycle(_cycle) => {
                    code.tool.prohibit("D")?;
                    self.prepare_cut()?;

                    // TODO

                    self.movement = None;
                }
            }
        }

        Ok(())
    }

    fn prepare_cut(&self) -> Result<(), SimpleError> {
        let speed = self.speed.unwrap_or(0);
        if speed < self.cfg.min_speed {
            return Err(SimpleError(format!("Speed {speed} is too low")));
        }
        if speed > self.cfg.max_speed {
            return Err(SimpleError(format!("Speed {speed} is too high")));
        }

        let feed = self.feed.unwrap_or(0);
        if feed < self.cfg.min_feed {
            return Err(SimpleError(format!("Feed {feed} is too low")));
        }
        if feed > self.cfg.max_feed {
            return Err(SimpleError(format!("Feed {feed} is too high")));
        }

        if self.x.is_none() || self.y.is_none() || self.z.is_none() {
            return Err(SimpleError("Trying to cut from undefined position".into()));
        }

        if self.tool.is_none() {
            return Err(SimpleError("Trying to cut with no tool".into()));
        }

        Ok(())
    }

    fn circle(
        &mut self,
        ty: Circle,
        i: Micrometer,
        j: Micrometer,
        x: Micrometer,
        y: Micrometer,
    ) -> Result<(), SimpleError> {
        self.prepare_cut()?;
        let start_x = self.x.expect("Bug: no current x");
        let start_y = self.y.expect("Bug: no current y");

        // This machine always works with relative I and J
        let r = i.to_mm().hypot(j.to_mm());
        let cx = start_x + i;
        let cy = start_y + j;
        let ex = x - cx;
        let ey = y - cy;
        let r2 = ex.to_mm().hypot(ey.to_mm());

        let r_mm = Micrometer::from_mm(r);
        if Micrometer::from_mm(r2) != r_mm {
            return Err(SimpleError(format!("Circle end point not on the circle (radius = {r_mm}, start at ({start_x}, {start_y})")));
        }

        let a1 = (-j).to_mm().atan2((-i).to_mm());
        let a2 = ey.to_mm().atan2(ex.to_mm());

        self.x = Some(x);
        self.y = Some(y);
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
enum Circle {
    Cw,
    Ccw,
}

trait Update {
    fn upd(&mut self, other: Self) -> bool;
}

impl<T: Sized> Update for Option<T> {
    fn upd(&mut self, other: Self) -> bool {
        if other.is_some() {
            *self = other;
            true
        } else {
            false
        }
    }
}

trait Require<T: Copy + Sized> {
    fn provided(&self) -> Option<T>;
    fn require(&self, msg: &str) -> Result<T, SimpleError> {
        self.provided()
            .ok_or_else(|| SimpleError(format!("Required parameter '{msg}'")))
    }

    fn prohibit(&self, msg: &str) -> Result<(), SimpleError> {
        if self.provided().is_some() {
            Err(SimpleError(format!("Parameter '{msg}' is dangerous here")))
        } else {
            Ok(())
        }
    }
}

impl<T: Copy + Sized> Require<T> for Option<T> {
    fn provided(&self) -> Option<T> {
        self.as_ref().map(Clone::clone)
    }
}
