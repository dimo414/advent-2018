mod point {
    use std::fmt;
    use std::ops::{Add,AddAssign};
    use regex::Regex;
    use std::str::FromStr;

    #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
    pub struct Point {
        pub x: i32,
        pub y: i32,
    }

    #[inline]
    pub fn point(x: i32, y: i32) -> Point {
        Point { x, y }
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
        type Err = String;

        fn from_str(s: &str) -> Result<Self, String> {
            lazy_static! {
                static ref RE: Regex = Regex::new(r"^([^,]+),([^,]+)$").unwrap();
            }

            let caps = RE.captures(s).ok_or("no match")?;
            let x: i32 = caps.get(1).expect("valid capture group").as_str().trim().parse().map_err(|_| "bad parse")?;
            let y: i32 = caps.get(2).expect("valid capture group").as_str().trim().parse().map_err(|_| "bad parse")?;
            return Ok(point(x, y));
        }
    }

    impl fmt::Display for Point {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "({}, {})", self.x, self.y)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn parse() {
            assert_eq!("3, 4".parse::<Point>(), Ok(point(3, 4)));
            assert_eq!("-3,-4".parse::<Point>(), Ok(point(-3, -4)));
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

    #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
    pub struct Vector {
        pub x: i32,
        pub y: i32,
    }

    #[inline]
    pub fn vector(x: i32, y: i32) -> Vector {
        Vector { x, y }
    }

    impl FromStr for Vector {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, String> {
            // Just reuse point's parser
            let p: super::Point = s.parse()?;
            Ok(vector(p.x, p.y))
        }
    }

    impl fmt::Display for Vector {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "({}, {})", self.x, self.y)
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
