use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub fn advent() {
    let coords = read_data();
    println!("Number of disjoint constellations: {}", num_constellations(coords));
}

fn read_data() -> Vec<Point> {
    BufReader::new(File::open("data/day25.txt").expect("Cannot open")).lines()
        .map(|l| l.unwrap().parse().expect("Invalid entry")).collect()
}

fn num_constellations(coords: Vec<Point>) -> usize {
    let mut disjoint = Disjoint::new(coords.iter().cloned());
    let distances = pairwise_distances(&coords);
    for (p1, dists) in distances.iter() {
        for (p2, dist) in dists.iter() {
            if dist <= &3 {
                disjoint.union(p1, p2);
            }
        }
    }
    disjoint.to_sets().len()
}

// The returned map is not bi-directional; only one direction is recorded (e.g. A->B but not B->A)
fn pairwise_distances(coords: &[Point]) -> HashMap<Point, HashMap<Point, u32>> {
    let mut result = HashMap::new();
    for (i, p1) in coords.iter().enumerate() {
        let mut distances = HashMap::new();
        for p2 in coords[i+1..].iter() {
            if p1 == p2 { continue; }
            distances.insert(*p2, p1 - p2);
        }
        result.insert(*p1, distances);
    }
    result
}

// Pared-down fork of euclid/euclid3d, since we don't need much of the functionality
mod point {
    use std::ops::Sub;
    use regex::Regex;
    use std::str::FromStr;
    use crate::error::ParseError;

    #[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
    pub struct Point {
        pub x: i32,
        pub y: i32,
        pub z: i32,
        pub w: i32,
    }

    #[inline]
    pub const fn point(x: i32, y: i32, z: i32, w: i32) -> Point {
        Point { x, y, z, w }
    }

    impl Sub for &Point {
        type Output = u32;

        fn sub(self, point: &Point) -> u32 {
            ((self.x - point.x).abs() + (self.y - point.y).abs() +
                (self.z - point.z).abs() + (self.w - point.w).abs()) as u32
        }
    }

    impl FromStr for Point {
        type Err = ParseError;

        fn from_str(s: &str) -> Result<Self, ParseError> {
            lazy_static! {
                static ref RE: Regex = Regex::new(r"^\(?([^(,]+),([^),]+),([^),]+),([^),]+)\)?$").unwrap();
            }

            let caps = regex_captures!(RE, s)?;
            let x: i32 = capture_group!(caps, 1).trim().parse()?;
            let y: i32 = capture_group!(caps, 2).trim().parse()?;
            let z: i32 = capture_group!(caps, 3).trim().parse()?;
            let w: i32 = capture_group!(caps, 4).trim().parse()?;
            return Ok(point(x, y, z, w));
        }
    }
}
pub use self::point::{Point,point};

mod disjoint {
    use super::Point;
    use std::collections::{HashMap, HashSet};
    use std::cell::RefCell;
    use itertools::Itertools;
    use std::cmp::Reverse;

    pub struct Disjoint {
        sets: RefCell<HashMap<Point, Point>>
    }

    impl Disjoint {
        pub fn new(coords: impl IntoIterator<Item=Point>) -> Disjoint {
            Disjoint { sets: RefCell::new(coords.into_iter().map(|p| (p, p)).collect()) }
        }

        pub fn to_sets(&self) -> Vec<HashSet<Point>> {
            let mut sets = HashMap::new();
            let keys: Vec<_> = self.sets.borrow().keys().cloned().collect();
            for coord in keys {
                sets.entry(self.find(&coord)).or_insert(HashSet::new()).insert(coord);
            }
            sets.into_iter().map(|e| e.1).sorted_by_key(|s| Reverse(s.len())).collect()
        }

        pub fn union(&mut self, coord1: &Point, coord2: &Point) {
            let root1 = self.find(coord1).expect("Must exist in the set");
            let root2 = self.find(coord2).expect("Must exist in the set");
            if root1 == root2 { return; }
            self.sets.borrow_mut().insert(root1, root2);
        }

        fn find(&self, coord: &Point) -> Option<Point> {
            Disjoint::find_impl(coord, &mut *self.sets.borrow_mut())
        }

        fn find_impl(coord: &Point, sets: &mut HashMap<Point, Point>) -> Option<Point> {
            let parent = sets.get(coord);
            if parent.is_none() { println!("NOT FOUND: {:?} - {:?}", coord, sets.keys()); return None; }
            let parent = *parent.expect("is some");
            if coord == &parent { return Some(parent); }
            let root = Disjoint::find_impl(&parent, sets).expect("Must exist in the set");
            sets.insert(*coord, root);
            Some(root)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use super::super::point;

        #[test]
        fn basic() {
            let pnt = |x| point(x, 0, 0, 0);
            let mut disjoint = Disjoint::new({1..9}.map(|x| pnt(x)));
            disjoint.union(&pnt(8), &pnt(6));
            disjoint.union(&pnt(3), &pnt(4));
            disjoint.union(&pnt(5), &pnt(2));
            disjoint.union(&pnt(1), &pnt(8));
            disjoint.union(&pnt(2), &pnt(6));

            let expected = vec!(
                vec!(pnt(1), pnt(2), pnt(5), pnt(6), pnt(8)).iter().cloned().collect(),
                vec!(pnt(3), pnt(4)).iter().cloned().collect(),
                vec!(pnt(7)).iter().cloned().collect()
            );

            assert_eq!(disjoint.to_sets(), expected);
        }
    }
}
pub use self::disjoint::Disjoint;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_file() { read_data(); }

    parameterized_test::create!{ examples, (coords, constellations), {
        let coords: Vec<Point> = coords.iter().map(|s| s.parse().expect("valid")).collect();
        assert_eq!(num_constellations(coords), constellations);
    }}
    examples! {
        ex1: (vec!("0,0,0,0","3,0,0,0","0,3,0,0","0,0,3,0","0,0,0,3","0,0,0,6","9,0,0,0","12,0,0,0")
            , 2),
        ex2: (vec!("-1,2,2,0","0,0,2,-2","0,0,0,-2","-1,2,0,0","-2,-2,-2,2","3,0,2,-1","-1,3,2,2",
            "-1,0,-1,0","0,2,1,-2","3,0,0,0"), 4),
        ex3: (vec!("1,-1,0,1","2,0,-1,0","3,2,-1,0","0,0,3,1","0,0,-1,-1","2,3,-2,0","-2,2,0,0",
            "2,-2,0,-1","1,-1,0,-1","3,2,0,2"), 3),
        ex4: (vec!("1,-1,-1,-2","-2,-2,0,1","0,2,1,3","-2,3,-2,1","0,2,3,-2","-1,-1,1,-2"
            ,"0,-2,-1,0","-2,2,3,-1","1,2,2,0","-1,-2,0,-2"), 8),
    }
}