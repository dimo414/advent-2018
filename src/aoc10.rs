use std::collections::HashSet;
use std::cmp;
use std::fs::File;
use std::io::{BufRead, BufReader};

const REAL_DATA: &str = "data/day10.txt";
#[allow(dead_code)]
const TEST_DATA: &str = "data/day10-example.txt";

pub fn advent() {
    let stars = read_data(REAL_DATA);
    let (steps, message) = find_message(stars);
    println!("Message:\n{}", message);
    println!("After {} seconds", steps);
}

fn read_data(path: &str) -> Vec<Star> {
    let reader = BufReader::new(File::open(path).expect("Cannot open"));

    reader.lines().map(|l| l.unwrap().parse().unwrap()).collect()
}

fn bounding_box(stars: &Vec<Star>) -> (Point, Point) {
    if stars.is_empty() {
        panic!("No stars");
    }

    // https://github.com/rust-lang/rfcs/issues/372
    let mut min_x = stars[0].position.x;
    let mut min_y = stars[0].position.y;
    let mut max_x = min_x;
    let mut max_y = min_y;

    for star in stars[1..].iter() {
        min_x = cmp::min(min_x, star.position.x);
        min_y = cmp::min(min_y, star.position.y);
        max_x = cmp::max(max_x, star.position.x);
        max_y = cmp::max(max_y, star.position.y);
    }
    (point(min_x, min_y), point(max_x, max_y))
}

fn area(min: Point, max: Point) -> u64 {
    assert!(min.x <= max.x);
    assert!(min.y <= max.y);
    let len = (max.x - min.x + 1) as u64;
    let width = (max.y - min.y + 1) as u64;
    len * width
}

fn stars_to_string(bounds: (Point, Point), stars: &Vec<Star>) -> String {
    assert!(area(bounds.0, bounds.1) < 10000, "Area too big, violates sanity-check");
    let points: HashSet<_> = stars.iter().map(|s| s.position).collect();
    let mut out = String::new();
    for y in {bounds.0.y-1..bounds.1.y+2} {
        for x in {bounds.0.x-1..bounds.1.x+2} {
            let p = point(x, y);
            if points.contains(&p) {
                out.push('#');
            } else {
                out.push('.');
            }
        }
        out.push('\n');
    }
    out
}

// Not sure if there's a good way to pass in an &Vec<Star> instead
fn find_message(stars: Vec<Star>) -> (u32, String) {
    let mut stars = stars;
    let mut bounds = bounding_box(&stars);
    let mut steps = 0;
    loop {
        let next_stars: Vec<_> = stars.iter().map(Star::step).collect();
        let next_bounds = bounding_box(&next_stars);
        // If the star field starts expanding
        if area(next_bounds.0, next_bounds.1) > area(bounds.0, bounds.1) {
            return (steps, stars_to_string(bounds, &stars));
        }
        stars = next_stars;
        bounds = next_bounds;
        steps += 1;
    }
}

mod star {
    use super::Point;
    use regex::Regex;
    use std::str::FromStr;

    #[derive(Debug, Eq, PartialEq)]
    pub struct Star {
        pub position: Point,
        velocity: Point,
    }

    impl Star {
        pub fn new(position: Point, velocity: Point) -> Star {
            Star { position, velocity }
        }

        pub fn step(&self) -> Star {
            Star::new(self.position + self.velocity, self.velocity)
        }
    }

    impl FromStr for Star {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, String> {
            lazy_static! {
                static ref RE: Regex = Regex::new(r"^position=<(.*)> velocity=<(.*)>$").unwrap();
            }

            let caps = RE.captures(s).ok_or("no match")?;
            let position: Point = caps.get(1).expect("valid capture group").as_str().parse()?;
            let velocity: Point = caps.get(2).expect("valid capture group").as_str().parse()?;

            Ok(Star{ position, velocity })
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use super::super::point;

        #[test]
        fn parse() {
            assert_eq!("position=< 9,  1> velocity=< 0,  2>".parse::<Star>(),
                       Ok(Star { position: point(9, 1), velocity: point(0, 2) } ));
        }

        #[test]
        fn stepping() {
            let star = Star::new(point(10, 4), point(-3, -2));
            let star = star.step();
            assert_eq!(star, Star::new(point(7, 2), point(-3, -2)));
            let star = star.step();
            assert_eq!(star, Star::new(point(4, 0), point(-3, -2)));
        }
    }
}
pub use self::star::Star;

mod euclid {
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

    impl Add for Point {
    type Output = Point;

    fn add(self, other: Point) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
    impl AddAssign for Point {
        fn add_assign(&mut self, other: Point) {
            *self = Point {
                x: self.x + other.x,
                y: self.y + other.y,
            };
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
    }
}
pub use self::euclid::{point,Point};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_file() {
        assert!(read_data(REAL_DATA).len() > 0);
    }

    #[test]
    fn bounding() {
        let points = vec!(point(-2,5), point(-3, 2), point(4, -2));
        let stars: Vec<_> = points.into_iter().map(|p| Star::new(p, point(4,5))).collect();
        assert_eq!(bounding_box(&stars), (point(-3,-2), point(4,5)));
    }

    #[test]
    fn point_area() {
        assert_eq!(area(point(0, 0), point(0, 0)), 1);
        assert_eq!(area(point(-5, -5), point(-2, -2)), 16);
        assert_eq!(area(point(2, 2), point(5, 5)), 16);
        assert_eq!(area(point(-3, -1), point(1, 3)), 25);
        // overflows a u32!
        assert_eq!(area(point(-46163, -46213), point(46547, 46446)), 8590601260);
        assert_eq!(area(point(-46158, -46208), point(46542, 46441)), 8588747650);
    }

    #[test]
    fn to_str() {
        let points = vec!(
            point(-2,-2), point(-1, -1), point(0, 0), point(1, 1), point(1, -1), point(-1, 1));
        let stars: Vec<_> = points.into_iter().map(|p| Star::new(p, point(4,5))).collect();
        assert_eq!(stars_to_string(bounding_box(&stars), &stars),
                   "......\n.#....\n..#.#.\n...#..\n..#.#.\n......\n");
    }

    #[test]
    fn example() {
        let message = "\
            ............\n\
            .#...#..###.\n\
            .#...#...#..\n\
            .#...#...#..\n\
            .#####...#..\n\
            .#...#...#..\n\
            .#...#...#..\n\
            .#...#...#..\n\
            .#...#..###.\n\
            ............\n";

        assert_eq!(find_message(read_data(TEST_DATA)), (3, message.into()));
    }
}