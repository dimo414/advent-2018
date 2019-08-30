use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::BinaryHeap;
use std::cmp::Ordering;
use crate::euclid3d::{Point,vector};

pub fn advent() {
    let nanobots = read_data();
    let strongest = find_strongest(&nanobots).expect("Must have bots");
    println!("Strongest signal: {:?}", strongest);
    println!("Bots reachable from strongest: {}", count_reachable(&strongest, &nanobots));
    let most_reachable = find_most_reachable_coord(&nanobots).expect("Not empty");
    println!("Optimal coord: {}", most_reachable);
    println!("Distance: {}", (most_reachable - Point::ORIGIN).grid_len());
}

fn read_data() -> Vec<Nanobot> {
    BufReader::new(File::open("data/day23.txt").expect("Cannot open")).lines()
        .map(|l| l.unwrap().parse().expect("Invalid entry")).collect()
}

fn find_strongest(nanobots: &[Nanobot]) -> Option<Nanobot> {
    nanobots.iter().fold(None, |r: Option<Nanobot>, c|
        match r {
            Some(n) if c.radius() <= n.radius() => r,
            _ => Some(*c),
        })
}

fn count_reachable(source: &Nanobot, nanobots: &[Nanobot]) -> usize {
    nanobots.iter().filter(|n| source.reachable(&n.pos())).count()
}

// Approach taken from:
// https://raw.githack.com/ypsu/experiments/master/aoc2018day23/vis.html
// First attempted to "shrink" the size of the search space by repeatedly scaling the bots' position
// and range down by 10. This works to a point, but doesn't on its own make the space small enough
// to practically search. Using a heap as done here might have helped, but the loss of precision
// from scaling would likely get in the way.
fn find_most_reachable_coord(nanobots: &[Nanobot]) -> Option<Point> {
    let (min, max) = Point::bounding_box(&nanobots.iter().map(|n| n.pos()).collect::<Vec<_>>())
        .expect("No nanobots found");
    let region = Region::from_min_max(min, max);
    let initial_score = Score::new(region, nanobots);

    let mut heap = BinaryHeap::new();
    heap.push(initial_score);

    while !heap.is_empty() {
        let candidate = heap.pop().expect("Not empty");
        if let Some(coord) = candidate.region().as_point() {
            return Some(coord);
        }

        for subregion in candidate.region().split() {
            heap.push(Score::new(subregion, nanobots));
        }
    }

    None
}

mod score {
    use super::*;

    #[derive(Debug)]
    pub struct Score {
        region: Region,
        in_range: u32,
    }

    impl Score {
        pub fn new(region: Region, nanobots: &[Nanobot]) -> Score {
            let in_range = nanobots.iter()
                .filter(|b| (&region).distance_to(&b.pos()) <= b.radius())
                .count() as u32;
            Score { region, in_range }
        }

        pub fn region(&self) -> Region { self.region }
    }

    // We don't implement Eq because it's not necessary, but Ord requires it exist
    impl PartialEq for Score { fn eq(&self, _: &Self) -> bool { unimplemented!() } }

    impl Eq for Score {}

    impl Ord for Score {
        fn cmp(&self, other: &Score) -> Ordering {
            self.in_range.cmp(&other.in_range) // max in-range
                .then_with(|| {
                    let self_dist = (self.region.origin() - Point::ORIGIN).grid_len();
                    let other_dist = (other.region.origin() - Point::ORIGIN).grid_len();
                    other_dist.cmp(&self_dist) // min dist
                })
                .then_with(|| other.region.size().cmp(&self.region.size())) // min size
        }
    }

    impl PartialOrd for Score {
        fn partial_cmp(&self, other: &Score) -> Option<Ordering> { Some(self.cmp(other)) }
    }
}
pub use self::score::Score;

mod region {
    use super::*;

    #[derive(Copy, Clone, Debug)]
    pub struct Region {
        origin: Point,
        size: u32,
    }

    impl Region {
        fn new(origin: Point, size: u32) -> Region {
            assert!(size >= 1);
            Region { origin, size }
        }

        pub fn from_min_max(min: Point, max: Point) -> Region {
            assert!(min.x <= max.x);
            assert!(min.y <= max.y);
            assert!(min.z <= max.z);
            Region::new(min, (max - min).grid_len().next_power_of_two())
        }

        pub fn origin(&self) -> Point { self.origin }
        pub fn size(&self) -> u32 { self.size }

        pub fn as_point(&self) -> Option<Point> {
            match self.size {
                1 => Some(self.origin),
                _ => None,
            }
        }

        pub fn distance_to(&self, coord: &Point) -> u32 {
            let s = self.size as i32 - 1;
            let axis_dist = |o, c| {
                if c < o { o - c } else if c > o + s { c - (o + s) } else { 0 }
            };

            (axis_dist(self.origin.x, coord.x).abs()
                + axis_dist(self.origin.y, coord.y).abs()
                + axis_dist(self.origin.z, coord.z).abs()) as u32
        }

        pub fn split(&self) -> Vec<Region> {
            assert!(self.size > 1); // sanity-check
            let size = self.size / 2;
            let s = size as i32;
            vec!(
                vector(0, 0, 0),
                vector(s, 0, 0),
                vector(0, s, 0),
                vector(0, 0, s),
                vector(s, s, 0),
                vector(s, 0, s),
                vector(0, s, s),
                vector(s, s, s),
            ).iter().map(|v| self.origin + v).map(|o| Region::new(o, size)).collect()
        }
    }
}
pub use self::region::Region;

mod nanobot {
    use super::*;
    use std::str::FromStr;
    use regex::Regex;

    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub struct Nanobot {
        pos: Point,
        radius: u32,
    }

    impl Nanobot {
        #[inline]
        pub fn new(pos: Point, radius: u32) -> Nanobot { Nanobot { pos, radius } }

        pub fn pos(&self) -> Point { self.pos }
        pub fn radius(&self) -> u32 { self.radius }

        pub fn reachable(&self, target: &Point) -> bool {
            (self.pos - target).grid_len() <= self.radius
        }
    }

    impl FromStr for Nanobot {
        type Err = String;

        fn from_str(s: &str) -> Result<Nanobot, String> {
            lazy_static! {
            static ref RE: Regex = Regex::new(r"^pos=<(.+)>, r=(\d+)$").unwrap();
        }

            let caps = regex_captures!(RE, s)?;
            let pos: Point = capture_group!(caps, 1).parse().map_err(|_| "NOPE".to_string())?;
            let radius: u32 = capture_group!(caps, 2).parse().map_err(|_| "NOPE".to_string())?;
            Ok(Nanobot::new(pos, radius))
        }
    }
}
pub use self::nanobot::Nanobot;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::euclid3d::point;

    #[test]
    fn read_file() { read_data(); }

    #[test]
    fn example_strongest() {
        let nanobots: Vec<Nanobot> = vec!(
            "pos=<0,0,0>, r=4",
            "pos=<1,0,0>, r=1",
            "pos=<4,0,0>, r=3",
            "pos=<0,2,0>, r=1",
            "pos=<0,5,0>, r=3",
            "pos=<0,0,3>, r=1",
            "pos=<1,1,1>, r=1",
            "pos=<1,1,2>, r=1",
            "pos=<1,3,1>, r=1").iter().map(|s| s.parse().unwrap()).collect();

        let strongest = find_strongest(&nanobots);
        assert_eq!(strongest, Some(Nanobot::new(point(0,0,0), 4)));
        assert_eq!(count_reachable(&strongest.unwrap(), &nanobots), 7);
    }

    #[test]
    fn most_reachable() {
        let nanobots: Vec<Nanobot> = vec!(
            "pos=<10,12,12>, r=2",
            "pos=<12,14,12>, r=2",
            "pos=<16,12,12>, r=4",
            "pos=<14,14,14>, r=6",
            "pos=<50,50,50>, r=200",
            "pos=<10,10,10>, r=5",
        ).iter().map(|s| s.parse().unwrap()).collect();

        let naive_coord = find_most_reachable_coord(&nanobots);
        assert_eq!(naive_coord, Some(point(12, 12, 12)));
    }
}