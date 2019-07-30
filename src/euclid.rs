// Referenced https://docs.rs/rusttype/0.5.2/src/rusttype/geometry.rs.html
// Other resources:
//   https://crates.io/crates/euclid - https://doc.servo.org/src/euclid/point.rs.html
mod point {
    use std::fmt;
    use std::ops::{Add,AddAssign};
    use regex::Regex;
    use std::str::FromStr;
    use crate::error::ParseError;

    #[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
    pub struct Point {
        pub x: i32,
        pub y: i32,
    }

    #[inline]
    pub const fn point(x: i32, y: i32) -> Point {
        Point { x, y }
    }

    impl Point {
        pub fn grid_distance(&self, other: Point) -> u32 {
            let dx = self.x - other.x;
            let dy = self.y - other.y;
            (dx.abs() + dy.abs()) as u32
        }

        pub fn in_bounds(&self, min: Point, max: Point) -> bool {
            assert!(min.x <= max.x);
            assert!(min.y <= max.y);
            min.x <= self.x && min.y <= self.y && max.x >= self.x && max.y >= self.y
        }
    }

    impl Add<&super::Vector> for Point {
        type Output = Point;

        fn add(self, vec: &super::Vector) -> Point {
            point(self.x + vec.x, self.y + vec.y)
        }
    }

    impl Add<super::Vector> for &Point {
        type Output = Point;

        fn add(self, vec: super::Vector) -> Point {
            point(self.x + vec.x, self.y + vec.y)
        }
    }

    impl Add<super::Vector> for Point {
        type Output = Point;

        fn add(self, vec: super::Vector) -> Point {
            point(self.x + vec.x, self.y + vec.y)
        }
    }

    impl AddAssign<super::Vector> for Point {
        fn add_assign(&mut self, vec: super::Vector) {
            *self = point(self.x + vec.x, self.y + vec.y);
        }
    }

    impl FromStr for Point {
        type Err = ParseError;

        fn from_str(s: &str) -> Result<Self, ParseError> {
            lazy_static! {
                // r"^([^,]+),([^,]+)$" would be more strict - worth it?
                static ref RE: Regex = Regex::new(r"^\(?([^(,]+),([^),]+)\)?$").unwrap();
            }

            let caps = regex_captures!(RE, s)?;
            let x: i32 = capture_group!(caps, 1).trim().parse()?;
            let y: i32 = capture_group!(caps, 2).trim().parse()?;
            return Ok(point(x, y));
        }
    }

    impl fmt::Debug for Point {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "({}, {})", self.x, self.y)
        }
    }

    impl fmt::Display for Point {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{:?}", self)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn parse() {
            assert_eq!("3, 4".parse::<Point>(), Ok(point(3, 4)));
            assert_eq!("-3,-4".parse::<Point>(), Ok(point(-3, -4)));
            assert_eq!("(40,30)".parse::<Point>(), Ok(point(40, 30)));
            assert_eq!("(-3, -5)".parse::<Point>(), Ok(point(-3, -5)));

            assert!("abc".parse::<Point>().is_err());
        }

        #[test]
        fn grid_distances() {
            let check_distance = |p1: Point, p2: Point, d: u32| {
                assert_eq!(p1.grid_distance(p2), d);
                assert_eq!(p2.grid_distance(p1), d);
            };

            check_distance(point(1,1), point(1,1), 0);
            check_distance(point(1,1), point(1,2), 1);
            check_distance(point(1,1), point(2,2), 2);
            check_distance(point(1,1), point(1,5), 4);
            check_distance(point(1,1), point(8,3), 9);
            check_distance(point(1,1), point(-1,-1), 4);
        }

        #[test]
        fn in_bounds_() {
            let zero_zero = point(0, 0);
            let two_two = point(2, 2);
            let five_six = point(5, 6);
            assert!(two_two.in_bounds(zero_zero, two_two));
            assert!(!five_six.in_bounds(zero_zero, two_two));
        }

        #[test]
        fn add() {
            assert_eq!(point(1, 0) + super::super::vector(2, 3), point(3, 3));
        }
    }
}
pub use self::point::{Point,point};

mod vector {
    use std::fmt;
    use std::str::FromStr;
    use crate::error::ParseError;

    #[derive(Copy, Clone, PartialEq, Eq, Hash)]
    pub struct Vector {
        pub x: i32,
        pub y: i32,
    }

    #[inline]
    pub const fn vector(x: i32, y: i32) -> Vector {
        Vector { x, y }
    }

    impl FromStr for Vector {
        type Err = ParseError;

        fn from_str(s: &str) -> Result<Self, ParseError> {
            // Just reuse point's parser
            let p: super::Point = s.parse()?;
            Ok(vector(p.x, p.y))
        }
    }

    impl fmt::Debug for Vector {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "({}, {})", self.x, self.y)
        }
    }

    impl fmt::Display for Vector {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{:?}", self)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn parse() {
            assert_eq!("3, 4".parse::<Vector>(), Ok(vector(3, 4)));
            assert_eq!("-3,-4".parse::<Vector>(), Ok(vector(-3, -4)));
        }
    }
}
pub use self::vector::{Vector,vector};
