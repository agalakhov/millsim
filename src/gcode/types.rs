//! Types for G-Code interpreter

use derive_more::{Add, AddAssign, Sub, SubAssign};
use std::fmt;

use nom::{
    branch::alt,
    character::complete::{char, one_of, u32},
    combinator::{consumed, map, opt},
    sequence::{preceded, tuple},
    IResult,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Add, AddAssign, Sub, SubAssign)]
pub struct Micrometer(pub i64);

impl Micrometer {
    /// Convert millimeter float to micrometers
    ///
    /// # Panics
    /// Panics if `mm` does not fit into `f64`, is `Inf` or `NaN`
    #[allow(dead_code)]
    pub fn from_mm(mm: f64) -> Self {
        let f = (mm * 1_000.0).round();
        let i = f as i64;
        if i as f64 != f {
            panic!("Impossible float to integer conversion")
        }
        Self(i)
    }

    /// Parse from `nom`
    pub fn parse(input: &str) -> IResult<&str, Micrometer> {
        fn decimal(input: &str) -> IResult<&str, u32> {
            map(consumed(u32::<&str, _>), |(s, n)| match s.len() {
                3 => n,
                a @ 0..=2 => n * 10_u32.pow(3 - a as u32),
                a => n / 10_u32.pow(a as u32 - 3),
            })(input)
        }

        map(
            tuple((
                opt(map(one_of("+-"), |s| s == '-')),
                alt((
                    map(preceded(char('.'), decimal), |d| (0, Some(d))),
                    tuple((u32, opt(preceded(char('.'), decimal)))),
                )),
            )),
            |(sign, (x, d))| {
                let x = x as i64 * 1000 + d.unwrap_or(0) as i64;
                let x = if sign.unwrap_or(false) { -x } else { x };
                Micrometer(x)
            },
        )(input)
    }

    /// Convert micrometers to millimeter float
    #[allow(dead_code)]
    pub fn to_mm(self) -> f64 {
        (self.0 as f64) / 1_000.0
    }
}

impl fmt::Display for Micrometer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let a = self.0 / 1000;
        let b = (self.0 % 1000).abs();
        write!(f, "{a}.{b:03}")
    }
}

#[cfg(test)]
mod tests {
    use super::Micrometer;

    #[test]
    fn mm_to_um() {
        let um = Micrometer::from_mm(1.0);
        assert_eq!(um, Micrometer(1000));
        let um = Micrometer::from_mm(-1.0);
        assert_eq!(um, Micrometer(-1000));
        let um = Micrometer::from_mm(0.001);
        assert_eq!(um, Micrometer(1));
        let um = Micrometer::from_mm(-0.001);
        assert_eq!(um, Micrometer(-1));
    }

    #[test]
    fn um_to_mm() {
        let mm = Micrometer(1000).to_mm();
        assert_eq!(mm, 1.0);
        let mm = Micrometer(-1000).to_mm();
        assert_eq!(mm, -1.0);
        let mm = Micrometer(1).to_mm();
        assert_eq!(mm, 0.001);
        let mm = Micrometer(-1).to_mm();
        assert_eq!(mm, -0.001);
    }

    #[test]
    fn um_format() {
        let um = Micrometer(7042);
        let s = format!("{um}");
        assert_eq!(s.as_str(), "7.042");
        let um = Micrometer(-7042);
        let s = format!("{um}");
        assert_eq!(s.as_str(), "-7.042");
    }

    #[test]
    fn um_parse() {
        fn um(m: &str) -> Micrometer {
            Micrometer::parse(m).unwrap().1
        }

        assert_eq!(um("1"), Micrometer(1000));
        assert_eq!(um("+1"), Micrometer(1000));
        assert_eq!(um("-1"), Micrometer(-1000));
        assert_eq!(um("1."), Micrometer(1000));
        assert_eq!(um("+1."), Micrometer(1000));
        assert_eq!(um("-1."), Micrometer(-1000));

        assert_eq!(um("1.1"), Micrometer(1100));
        assert_eq!(um("1.01"), Micrometer(1010));
        assert_eq!(um("1.001"), Micrometer(1001));
        assert_eq!(um("1.0001"), Micrometer(1000));
        assert_eq!(um("-1.1"), Micrometer(-1100));
        assert_eq!(um("-1.01"), Micrometer(-1010));
        assert_eq!(um("-1.001"), Micrometer(-1001));
        assert_eq!(um("-1.0001"), Micrometer(-1000));
        assert_eq!(um("-1.1000000"), Micrometer(-1100));

        assert_eq!(um(".42"), Micrometer(420));
        assert_eq!(um("-.42"), Micrometer(-420));
    }
}
