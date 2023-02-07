//! SVG render

use super::traits::{Circle, Line, Micrometer, Render};
use std::{
    io::{Write, Error},
    path::{Path, PathBuf},
    fs::File,
    fmt,
};

/// A SVG render using `piet_svg` library
#[derive(Debug)]
pub struct Svg {
    svg_file: PathBuf,
    items: Vec<DrawingItem>,
    current: Option<DrawingItem>,
    position: Option<(Micrometer, Micrometer)>,
}

impl Svg {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            svg_file: path.as_ref().to_owned(),
            items: Vec::new(),
            current: None,
            position: None,
        }
    }

    fn prepare(&mut self, tool: Micrometer, ty: Line) -> &mut DrawingItem {
        let cur = self.current.get_or_insert_with(|| DrawingItem {
            path: Vec::new(),
            ty,
            width: tool.to_mm(),
        });

        if cur.width != tool.to_mm() || cur.ty != ty {
            let path = self.position.iter().cloned().map(PathEl::Move).collect();
            self.items.push(self.current.replace(DrawingItem {
                path,
                ty,
                width: tool.to_mm(),
            }).unwrap());
        }

        self.current.as_mut().unwrap()
    }
}

impl Render for Svg {
    fn line_to(
        &mut self,
        tool: Micrometer,
        ty: Line,
        point: (Micrometer, Micrometer),
        _height: Micrometer,
    ) {
        let old_pos = self.position;
        let it = self.prepare(tool, ty);
        if old_pos != Some(point) {
            it.path.push(if it.path.is_empty() {
                PathEl::Move(point)
            } else {
                PathEl::Line(point)
            });
        }
        self.position = Some(point);
    }

    fn arc_to(
        &mut self,
        tool: Micrometer,
        ty: Circle,
        center: (Micrometer, Micrometer),
        end: (Micrometer, Micrometer),
    ) {
        let (sx, sy) = self.position.expect("Bug: circle with no start");
        let it = self.prepare(tool, Line::Cut);

        let (cx, cy) = center;
        let (ex, ey) = end;
        let r = (ex - cx).to_mm().hypot((ey - cy).to_mm());

        if (sx, sy) == (ex, ey) {
            // Full circle
            let ix = cx + cx - ex;
            let iy = cy + cy - ey;

            let kind = match ty {
                Circle::Cw => ArcKind::SmallRight,
                Circle::Ccw => ArcKind::SmallLeft,
            };

            it.path.push(PathEl::Arc { r, end: (ix, iy), kind });
            it.path.push(PathEl::Arc { r, end, kind });

        } else {
            let a1 = (sy - cy).to_mm().atan2((sx - cx).to_mm());
            let a2 = (ey - cy).to_mm().atan2((ex - cx).to_mm());
            let a = match ty {
                Circle::Cw => a1 - a2,
                Circle::Ccw => a2 - a1,
            };
            let a = a.to_degrees();
            let a = if a < 0.0 { a + 360.0 } else { a };
            assert!(a >= 0.0);
            assert!(a < 360.0);

            use ArcKind::*;
            let kind = if a > 180.0 {
                match ty {
                    Circle::Cw => LargeRight,
                    Circle::Ccw => LargeLeft,
                }
            } else {
                match ty {
                    Circle::Cw => SmallRight,
                    Circle::Ccw => SmallLeft,
                }
            };

            it.path.push(PathEl::Arc { r, end, kind });
        }

        self.position = Some(end);
    }

    fn finalize(mut self: Box<Self>) -> Result<(), Error> {
        if let Some(cur) = self.current.take() {
            self.items.push(cur);
        }

        let fd = File::create(self.svg_file)?;
        write_svg(fd, self.items)
    }
}

#[derive(Debug)]
enum PathEl {
    Move((Micrometer, Micrometer)),
    Line((Micrometer, Micrometer)),
    Arc {
        r: f64,
        end: (Micrometer, Micrometer),
        kind: ArcKind,
    },
}

impl fmt::Display for PathEl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use PathEl::*;
        match self {
            Move((x, y)) => write!(f, "M{x} {yy}", yy = -*y),
            Line((x, y)) => write!(f, "L{x} {yy}", yy = -*y),
            Arc { r, end: (x, y), kind } => write!(f, "A{r} {r} 0 {kind} {x} {yy}", yy = -*y),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum ArcKind {
    LargeLeft,
    LargeRight,
    SmallLeft,
    SmallRight,
}

impl fmt::Display for ArcKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ArcKind::*;
        let (large, sweep) = match self {
            LargeLeft => (1, 0),
            LargeRight => (1, 1),
            SmallLeft => (0, 0),
            SmallRight => (0, 1),
        };
        write!(f, "{large} {sweep}")
    }
}

#[derive(Debug)]
struct DrawingItem {
    ty: Line,
    width: f64,
    // color: TODO
    path: Vec<PathEl>,
}

fn write_svg(mut fd: impl Write, items: impl IntoIterator<Item = DrawingItem>) -> Result<(), Error> {
    let (width, height) = (400.0, 200.0);
    let (left, bottom) = (-width/2.0, -height/2.0);
    writeln!(fd, "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}mm\" height=\"{height}mm\" viewBox=\"{left} {bottom} {width} {height}\">")?;

    let material = Some(((300.0, 60.3), (300.0, 0.0)));

    if let Some(((w, h), (cx, cy))) = material {
        let x = -cx;
        let y = cy - h;
        write!(fd, "<rect x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\" stroke=\"none\" fill=\"grey\" />")?;
    }

    for item in items {
        let width = item.width;
        let (color, opacity) = match item.ty {
            Line::Fast => ("blue", 0.2),
            Line::Cut => ("green", 0.9),
        };
        write!(fd, "<path fill=\"none\" stroke=\"{color}\" stroke-width=\"{width}\" stroke-opacity=\"{opacity}\" d=\"")?;
        for el in item.path {
            write!(fd, "{el}")?;
        }
        writeln!(fd, "\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/>")?;
    }

    writeln!(fd, "</svg>")
}
