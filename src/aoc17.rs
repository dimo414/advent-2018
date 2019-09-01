use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn advent() {
    let scan = read_data();
    let flow = Flow::new(&scan);
    println!("Reachable tiles: {}", flow.reachable());
    println!("Retained water:  {}", flow.retained());
}

// TODO unit test reading this? Maybe too slow to actually construct the Scan
fn read_data() -> Scan {
    let reader = BufReader::new(File::open("data/day17.txt").expect("Cannot open"));
    let lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();
    Scan::new(&lines)
}

#[cfg(test)]
fn example_rules() -> Vec<&'static str> {
    vec!(
        "x=495, y=2..7",
        "y=7, x=495..501",
        "x=501, y=3..7",
        "x=498, y=2..4",
        "x=506, y=1..2",
        "x=498, y=10..13",
        "x=504, y=10..13",
        "y=13, x=498..504")
}

mod scan {
    use regex::Regex;
    use std::collections::HashSet;
    use std::fmt;
    use crate::euclid::{point, Point};

    pub struct Scan {
        spring: Point,
        clay: HashSet<Point>,
        bounds: Option<(Point, Point)>,
    }

    impl Scan {
        // https://stackoverflow.com/a/38185570/113632
        pub fn new<T: AsRef<str>>(veins: &[T]) -> Scan {
            assert!(!veins.is_empty()); // ensures that bounds wont stay empty
            let mut scan = Scan {
                spring: point(500, 0),
                clay: HashSet::new(),
                bounds: None,
            };

            for vein in veins {
                scan.scan_vein(vein.as_ref());
            }

            scan
        }

        fn scan_vein(&mut self, vein: &str) {
            lazy_static! {
                static ref RE: Regex = Regex::new(r"^([xy])=(\d+), ([xy])=(\d+)..(\d+)$").unwrap();
            }

            let caps = RE.captures(vein).expect("Invalid record");
            let fixed_dir = capture_group!(caps, 1);
            let fixed_coord: i32 = capture_group!(caps, 2).parse().expect("Invalid num");
            let vein_dir = capture_group!(caps, 3);
            let vein_min: i32 = capture_group!(caps, 4).parse().expect("Invalid num");
            let vein_max: i32 = capture_group!(caps, 5).parse().expect("Invalid num");

            assert!(vein_dir == "x" || vein_dir == "y");
            assert!(fixed_dir != vein_dir);

            let vein = vein_min..vein_max+1;
            let vein: Vec<Point> = match fixed_dir {
                "x" => { vein.map(|y| point(fixed_coord, y)).collect() },
                "y" => { vein.map(|x| point(x, fixed_coord)).collect() },
                _ => panic!("Unexepcted coord identifier: {}", fixed_dir),
            };
            self.clay.extend(vein);
            self.rebound();
        }

        fn rebound(&mut self) {
            // Include one left and right of the actual bounds, for flow
            self.bounds = Point::bounding_box(self.clay.iter().cloned())
                .map(|(min, max)| (point(min.x-1, min.y), point(max.x+1, max.y)));
        }

        pub fn spring(&self) -> Point {
            self.spring
        }

        pub fn clay_at(&self, coord: &Point) -> bool {
            self.clay.contains(coord)
        }

        pub fn bounds(&self) -> (Point, Point) {
            self.bounds.expect("Should be impossible")
        }

        // Bounds of the scan, i.e. may have a min-Y greater than 0
        pub fn in_scan_bounds(&self, coord: &Point) -> bool {
            let (min, max) = self.bounds();
            coord.in_bounds(min, max)
        }

        // Bounds of the scan+spring, i.e. min-Y is always 0
        pub fn in_spring_bounds(&self, coord: &Point) -> bool {
            let (min, max) = self.bounds();
            coord.in_bounds(point(min.x, 0), max)
        }

        pub fn display_helper(&self, f: &mut fmt::Formatter, rest: &HashSet<Point>, flow: &HashSet<Point>) -> fmt::Result {
            if self.clay.is_empty() {
                return write!(f, "");
            }

            let mut out = String::new();
            let (min, max) = self.bounds();
            for y in 0..max.y+2 { // don't use the min bound of the scan
                for x in min.x..max.x+1 {
                    let coord = point(x, y);
                    // TODO is it worth asserting that the coord is in only one of these branches?
                    if coord == self.spring {
                        out.push('+');
                    } else if rest.contains(&coord) {
                        out.push('~');
                    } else if flow.contains(&coord) {
                        out.push('|');
                    } else if self.clay.contains(&coord) {
                        out.push('#');
                    } else {
                        out.push('.');
                    }
                }
                out.push('\n');
            }
            write!(f, "{}", out)
        }
    }

    impl fmt::Display for Scan {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            lazy_static! {
                static ref EMPTY_SET: HashSet<Point> = HashSet::new();
            }
            self.display_helper(f, &EMPTY_SET, &EMPTY_SET)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::super::*;
        use super::*;

        #[test]
        fn try_parse_sample() {
            let scan = Scan::new(&example_rules());
            let expected =
                "......+.......\n............#.\n.#..#.......#.\n.#..#..#......\n\
                 .#..#..#......\n.#.....#......\n.#.....#......\n.#######......\n\
                 ..............\n..............\n....#.....#...\n....#.....#...\n\
                 ....#.....#...\n....#######...\n..............\n";
            assert_eq!(scan.to_string(), expected);
        }
    }
}
pub use scan::Scan;

mod flow {
    use std::collections::HashSet;
    use std::fmt;
    use crate::euclid::{Point, vector, Vector};
    use super::*;

    pub struct Flow<'a> {
        scan: &'a Scan,
        rest: HashSet<Point>,
        flow: HashSet<Point>,
    }

    impl<'a> Flow<'a> {
        pub fn new(scan: &Scan) -> Flow {
            let mut flow = Flow {
                scan,
                rest: HashSet::new(),
                flow: HashSet::new(),
            };

            let mut last_rest = flow.rest.len();
            let mut last_flow = flow.flow.len();
            loop {
                flow.advance();
                let cur_rest = flow.rest.len();
                let cur_flow = flow.flow.len();
                if last_rest == cur_rest && last_flow == cur_flow {
                    break;
                }
                last_rest = cur_rest;
                last_flow = cur_flow;
            }

            flow
        }

        fn advance(&mut self) {
            // for each flowing particle
            // if clear below (not clay or rest) move down
            // otherwise
              // move left and right if clear
              // if not clear search for opposing wall
                // if found move row to rest

            self.flow.insert(self.scan.spring());
            let mut new_flow = HashSet::new();
            for coord in self.flow.iter() {
                let coord_below = *coord + vector(0, 1);
                if self.can_flow(&coord_below) {
                    if self.scan.in_spring_bounds(&coord_below) {
                        new_flow.insert(coord_below);
                    }
                    // else flows out of bounds, ignore
                    continue;
                }
                let coord_left = *coord + vector(-1, 0);
                let coord_right = *coord + vector(1, 0);
                let mut should_search = false;
                if self.can_flow(&coord_left) {
                    assert!(self.scan.in_spring_bounds(&coord_left));
                    new_flow.insert(coord_left);
                } else {
                    should_search = true;
                }
                if self.can_flow(&coord_right) {
                    assert!(self.scan.in_spring_bounds(&coord_right));
                    new_flow.insert(coord_right);
                } else {
                    should_search = true;
                }

                if should_search {
                    if let Some((left, right)) = self.row_at_rest(&coord) {
                        let mut cur = left;
                        while cur != right {
                            new_flow.remove(&cur);
                            self.rest.insert(cur);
                            cur += vector(1, 0);
                        }
                        new_flow.remove(&cur);
                        self.rest.insert(cur);
                    }
                }
            }

            self.flow = new_flow.difference(&self.rest).cloned().collect();
        }

        fn can_flow(&self, coord: &Point) -> bool {
            !(self.scan.clay_at(coord) || self.rest.contains(coord))
        }

        fn row_at_rest(&self, coord: &Point) -> Option<(Point, Point)> {
            // search left while coord_below is blocked to find wall
            // then search right while coord_below is blocked to find wall
            // if any coord_below is not blocked short-circuit and return None

            let search_dir = |v: Vector| {
                let mut furthest = *coord;
                loop {
                    if self.can_flow(&(furthest + vector(0, 1))) {
                        // can flow down, not at rest
                        return None;
                    }
                    let next = furthest + v;
                    if self.scan.clay_at(&next) {
                        // abutting a wall, stop searching
                        return Some(furthest);
                    }
                    furthest = next;
                }
            };
            let furthest_left = search_dir(vector(-1, 0));
            let furthest_right = search_dir(vector(1, 0));
            // TODO cleaner way to transform (Option<P>, Option<P>) to Option<(P, P)>?
            if furthest_left.is_some() && furthest_right.is_some() {
                Some((furthest_left.unwrap(), furthest_right.unwrap()))
            } else {
                None
            }
        }

        pub fn reachable(&self) -> usize {
            self.retained() +
                self.flow.iter().filter(|p| self.scan.in_scan_bounds(p)).count()
        }

        pub fn retained(&self) -> usize {
            self.rest.iter().filter(|p| self.scan.in_scan_bounds(p)).count()
        }
    }


    impl<'a> fmt::Display for Flow<'a> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            self.scan.display_helper(f, &self.rest, &self.flow)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn simple_flow() {
            let veins: Vec<String> = vec!("y=3, x=498..501", "x=497, y=2..3", "x=502, y=2..3")
                .iter().map(|s| s.to_string()).collect();
            let scan = Scan::new(&veins);
            let flow = Flow::new(&scan);
            assert_eq!(flow.reachable(), 8); // overflow is out of bounds?
            assert_eq!(flow.retained(), 4);
            assert_eq!(flow.to_string(), "....+...\n||||||||\n|#~~~~#|\n|######|\n........\n");
        }

        #[test]
        fn flow() {
            let scan = Scan::new(&example_rules());
            println!("{}", scan);
            let flow = Flow::new(&scan);
            println!("{}", flow);
            assert_eq!(flow.reachable(), 57);
            assert_eq!(flow.retained(), 29);
            assert_eq!(flow.to_string(),
                       "......+.......\n......|.....#.\n.#..#||||...#.\n.#..#~~#|.....\n\
                       .#..#~~#|.....\n.#~~~~~#|.....\n.#~~~~~#|.....\n.#######|.....\n\
                       ........|.....\n...|||||||||..\n...|#~~~~~#|..\n...|#~~~~~#|..\n\
                       ...|#~~~~~#|..\n...|#######|..\n..............\n");
        }
    }
}
pub use flow::Flow;