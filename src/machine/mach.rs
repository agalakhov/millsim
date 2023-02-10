//! The milling machine simulator

use super::actions::{Command, Global, Movement, SpindleAction, WaterAction, CoordSwitch};
use crate::{
    errors::SimpleError,
    render::{Circle, Line, Render},
    types::Micrometer,
};

/// Machine configuration
#[derive(Debug)]
pub struct MachineConfig {
    /// Safe Z height to use at beginning and ending of machining cycle
    safe_z: Micrometer,
    /// Minimal allowed S value
    min_speed: u16,
    /// Maxilaml allowed S value
    max_speed: u16,
    /// Minimal allowed F value
    min_feed: u16,
    /// Maximal allowed F value
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
    render: Option<Box<dyn Render>>,

    movement: Option<Movement>,

    x: Option<Micrometer>,
    y: Option<Micrometer>,
    z: Option<Micrometer>,
    speed: Option<u16>,
    feed: Option<u16>,
    tool: Option<u8>,

    spindle_on: bool,
    water_on: bool,

    relative: bool,
}

impl Machine {
    #[allow(dead_code)]
    pub fn with_render(render: Option<Box<dyn Render>>) -> Self {
        Self {
            render,
            ..Self::default()
        }
    }

    #[allow(dead_code)]
    pub fn with_config(cfg: MachineConfig) -> Self {
        Self {
            cfg,
            ..Self::default()
        }
    }

    #[allow(dead_code)]
    pub fn with_render_and_config(render: Option<Box<dyn Render>>, cfg: MachineConfig) -> Self {
        Self {
            cfg,
            render,
            ..Self::default()
        }
    }

    pub fn finalize(self) -> Option<Box<dyn Render>> {
        self.render
    }

    pub fn execute_command(&mut self, code: Command) -> Result<(), SimpleError> {
        if let Some(Global::EndProgram) = code.global {
            if self.spindle_on {
                return Err(SimpleError("Ending program with spindle on".into()));
            }

            if self.water_on {
                return Err(SimpleError("Ending program with coolant on".into()));
            }

            if self.z.unwrap_or(self.cfg.safe_z) < self.cfg.safe_z {
                return Err(SimpleError("Ending program with too low Z".into()));
            }
        }

        self.speed.upd(code.speed);
        self.feed.upd(code.feed);

        let tool_changed = {
            let tc = self.tool.is_some();
            self.tool.upd(code.tool) && tc
        };

        if let Some(csw) = code.coord_switch {
            match csw {
                CoordSwitch::Absolute => self.relative = false,
                CoordSwitch::Relative => self.relative = true,
            }
        }

        struct Coord {
            x: Option<Micrometer>,
            y: Option<Micrometer>,
            z: Option<Micrometer>,
        }

        let coord = if self.relative {
            let (x, y, z) = if let (Some(x), Some(y), Some(z)) = (self.x, self.y, self.z) {
                (x, y, z)
            } else {
                return Err(SimpleError("Relative coordinates can only be used with fully defined position".into()))
            };
            
            Coord {
                x: code.raw_x.map(|a| a + x),
                y: code.raw_y.map(|a| a + y),
                z: code.raw_z.map(|a| a + z),
            }
        } else {
           Coord {
                x: code.raw_x,
                y: code.raw_y,
                z: code.raw_z,
            } 
        };

        let new_move = self.movement.upd(code.movement);

        if let Some(sp) = code.spindle_action {
            match sp {
                SpindleAction::SpindleOnCCW => {
                    return Err(SimpleError("Trying to start spindle backwards".into()))
                }
                SpindleAction::SpindleOnCW => {
                    if !self.water_on {
                        return Err(SimpleError(
                            "Trying to start spindle without ensuring coolant flow".into(),
                        ));
                    }
                    if self.speed.is_none() {
                        return Err(SimpleError(
                            "Trying to start spindle without any speed".into(),
                        ));
                    }
                    self.spindle_on = true;
                }
                SpindleAction::SpindleOff => {
                    if new_move {
                        return Err(SimpleError(
                            "Trying to turn off spindle while moving".into(),
                        ));
                    }
                    coord.x.prohibit("X")?;
                    coord.y.prohibit("Y")?;
                    coord.z.prohibit("Z")?;
                    code.i.prohibit("I")?;
                    code.j.prohibit("J")?;
                    self.spindle_on = false;
                    self.speed = None;
                }
            }
        }

        if let Some(wt) = code.water_action {
            match wt {
                WaterAction::WaterOn => {
                    self.water_on = true;
                }
                WaterAction::WaterOff => {
                    if new_move {
                        return Err(SimpleError(
                            "Trying to turn off coolant while moving".into(),
                        ));
                    }
                    if self.spindle_on {
                        return Err(SimpleError(
                            "Coolant turned off while spindle still running".into(),
                        ));
                    }
                    coord.x.prohibit("X")?;
                    coord.y.prohibit("Y")?;
                    coord.z.prohibit("Z")?;
                    code.i.prohibit("I")?;
                    code.j.prohibit("J")?;
                    self.water_on = false;
                }
            }
        }

        let mv = self.movement.as_ref().filter(|_| {
            new_move || coord.x.is_some() || coord.y.is_some() || coord.z.is_some()
        });

        let mut bad_tool_change = tool_changed;

        if let Some(mv) = mv {
            match mv {
                Movement::FastLine => {
                    if new_move {
                        code.tool.prohibit("D")?;
                    }
                    code.i.prohibit("I")?;
                    code.j.prohibit("J")?;

                    if self.z.is_none() {
                        // No horizontal movement until Z is safe
                        coord.x.prohibit("X")?;
                        coord.y.prohibit("Y")?;
                        let z = coord.z.require("Z")?;

                        if z != self.cfg.safe_z {
                            return Err(SimpleError(
                                "First movement should be to safe Z height".into(),
                            ));
                        }
                        self.z = Some(z);
                    } else if self.x.is_none() || self.y.is_none() {
                        self.z.upd(coord.z);
                        let z = self.z.unwrap();
                        if z < self.cfg.safe_z {
                            return Err(SimpleError(
                                "Unsafe movement without fully defininig the position".into(),
                            ));
                        }

                        self.x.upd(coord.x);
                        self.y.upd(coord.y);
                    } else {
                        self.x.upd(coord.x);
                        self.y.upd(coord.y);
                        self.z.upd(coord.z);
                    }

                    self.line(Line::Fast);
                }

                Movement::Line => {
                    code.tool.prohibit("D")?;
                    code.i.prohibit("I")?;
                    code.j.prohibit("J")?;
                    self.prepare_cut()?;
                    self.x.upd(coord.x);
                    self.y.upd(coord.y);
                    self.z.upd(coord.z);

                    self.line(Line::Cut);
                }

                Movement::CircleCW => {
                    code.tool.prohibit("D")?;
                    coord.z.prohibit("Z")?;
                    self.circle(
                        Circle::Cw,
                        code.i.require("I")?,
                        code.j.require("J")?,
                        coord.x.require("X")?,
                        coord.y.require("Y")?,
                    )?;
                }

                Movement::CircleCCW => {
                    code.tool.prohibit("D")?;
                    coord.z.prohibit("Z")?;
                    self.circle(
                        Circle::Ccw,
                        code.i.require("I")?,
                        code.j.require("J")?,
                        coord.x.require("X")?,
                        coord.y.require("Y")?,
                    )?;
                }

                Movement::ToolChange => {
                    code.tool.require("D")?;
                    coord.x.prohibit("X")?;
                    coord.y.prohibit("Y")?;
                    coord.z.prohibit("Z")?;
                    code.i.prohibit("I")?;
                    code.j.prohibit("J")?;
                    bad_tool_change = false;

                    if self.spindle_on || self.water_on {
                        return Err(SimpleError(
                            "Turn off spindle and coolant before performing tool change".into(),
                        ));
                    }

                    if self.z.unwrap_or(self.cfg.safe_z) < self.cfg.safe_z {
                        return Err(SimpleError(
                            "Must be high enough to perform tool change".into(),
                        ));
                    }

                    self.spindle_on = false;
                    self.water_on = false;
                    self.speed = None;
                    self.feed = None;

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
        } else {
            coord.x.prohibit("X")?;
            coord.y.prohibit("Y")?;
            coord.z.prohibit("Z")?;
            code.i.prohibit("I")?;
            code.j.prohibit("J")?;
        }

        if bad_tool_change {
            return Err(SimpleError("Tool change without stopping".into()))
        }

        Ok(())
    }

    fn prepare_cut(&self) -> Result<(), SimpleError> {
        if !self.spindle_on {
            return Err(SimpleError("Trying to cut with spindle off".into()));
        }

        if !self.water_on {
            return Err(SimpleError("Trying to cut without coolant".into()));
        }

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

    fn choose_tool(&self) -> Micrometer {
        Micrometer::from_mm(6.0) // TODO
    }

    fn line(&mut self, ty: Line) {
        let tool = self.choose_tool();
        if let (Some(render), Some(x), Some(y), Some(z)) =
            (&mut self.render, &self.x, &self.y, &self.z)
        {
            render.line_to(tool, ty, (*x, *y), *z);
        }
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
        let tool = self.choose_tool();
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

        if let Some(render) = &mut self.render {
            render.arc_to(tool, ty, (cx, cy), (x, y));
        }

        self.x = Some(x);
        self.y = Some(y);
        Ok(())
    }
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
