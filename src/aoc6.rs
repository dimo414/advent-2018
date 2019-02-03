use std::collections::{BTreeMap, HashSet};
use std::char;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn advent() {
    let coords = read_data();
    let labels = label_points(&coords);
    let grid = coverage_for_points(&labels);

    // takes up ~360 columns
    //println!("Final:\n{}", grid);
    println!("Largest Area: {}", find_largest_enclosed_area(&labels, &grid));

    let grid = sum_distances(&coords);
    println!("Less than 10k: {}", count_lessthan(&grid, 10000));
}


fn read_data() -> Vec<taxicab::Point> {
    let reader = BufReader::new(File::open("data/day6.txt").expect("Cannot open"));

    reader.lines().map(|l| l.unwrap().parse().unwrap()).collect()
}

#[derive(Debug, Eq, PartialEq)]
enum Coordinate {
    Labeled(String),
    Equidistant(u32),
    Nearest(String, u32),
}

impl fmt::Display for Coordinate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        //write!(f, "{:?}", self)
        match self {
            Coordinate::Labeled(s) => write!(f, "{}", s),
            Coordinate::Equidistant(_) => write!(f, "."),
            //Coordinate::Nearest(_, d) => write!(f, "{}", d),
            Coordinate::Nearest(l, _) => write!(f, "{}", l.to_lowercase()),
        }
    }
}

fn populate_distances(grid: &mut taxicab::Grid<Coordinate>, start_point: taxicab::Point) {
    match grid.get(start_point) {
        Some(Coordinate::Labeled(label)) => {
            // eager copy so the reference isn't passed into the closure
            let label = label.to_string();
            for point in grid.points_iter() {
                let distance = point.distance(start_point);
                grid.get_entry(point)
                    .and_modify(|e| {
                        match e {
                            Coordinate::Labeled(_) => {},
                            Coordinate::Equidistant(d) => {
                                if *d > distance {
                                    *e = Coordinate::Nearest(label.clone(), distance);
                                }
                            },
                            Coordinate::Nearest(_, d) => {
                                if *d > distance {
                                    *e = Coordinate::Nearest(label.clone(), distance);
                                } else if *d == distance {
                                    *e = Coordinate::Equidistant(distance);
                                }
                            }
                        }
                    })
                    .or_insert(Coordinate::Nearest(label.clone(), distance));
            }
        },
        x => panic!("Unexpected value in {}: {:?}", start_point, x),
    }
}

fn label_points(points: &Vec<taxicab::Point>) -> BTreeMap<String, taxicab::Point> {
    let mut l = 'A' as u32 - 1;
    let mut l_gen = || {
        l+=1; char::from_u32(l).expect("should be valid").to_string() };
    points.iter().map(|p| (l_gen(), *p)).collect()
}

fn coverage_for_points(labels: &BTreeMap<String, taxicab::Point>) -> taxicab::Grid<Coordinate> {
    let mut grid = taxicab::grid();
    for (label, point) in labels.iter() {
        grid.insert(*point, Coordinate::Labeled(label.clone()));
    }
    // Populating distances could be done incrementally, but since the grid is resized by insertions
    // it's necessary to insert all points first and then populate the distances
    for point in labels.values() {
        populate_distances(&mut grid, *point);
    }

    grid
}

fn compute_area(grid: &taxicab::Grid<Coordinate>, label_point: taxicab::Point) -> u32 {
    let mut sum = 0;
    let label = match grid.get(label_point).expect("absent") {
        Coordinate::Labeled(s) => s,
        _ => panic!("expected label"),
    };
    for point in grid.points_iter() {
        if let Some(c) = grid.get(point) {
            if let Coordinate::Labeled(s) = c {
                if s == label { sum += 1; }
            }
            if let Coordinate::Nearest(s, _) = c {
                if s == label { sum += 1; }
            }
        }
    }
    sum
}

fn find_largest_enclosed_area(labels: &BTreeMap<String, taxicab::Point>, grid: &taxicab::Grid<Coordinate>) -> u32 {
    let mut enclosed_labels: HashSet<_> = labels.keys().collect();
    let mut remove_label = |c: Option<&Coordinate>| {
        match c.expect("within bounds and already populated") {
            Coordinate::Labeled(l) => { enclosed_labels.remove(l); },
            Coordinate::Nearest(l, _) => { enclosed_labels.remove(l); },
            Coordinate::Equidistant(_) => {},
        }
    };

    let (min, max) = grid.bounds().expect("No points in empty grid");
    for x in {min.x..max.x} {
        remove_label(grid.get(taxicab::point(x, min.y)));
        remove_label(grid.get(taxicab::point(x, max.y)));
    }
    for y in {min.y..max.y} {
        remove_label(grid.get(taxicab::point(min.x, y)));
        remove_label(grid.get(taxicab::point(max.x, y)));
    }

    let mut largest = None;
    for enclosed_label in enclosed_labels {
        let point = *labels.get(enclosed_label).expect("present");
        let area = compute_area(&grid, point);
        match largest {
            Some((_, a)) => if a < area { largest = Some((point, area)); },
            None => { largest = Some((point, area)); },
        }
    }

    largest.expect("present").1
}

fn sum_distances(coords: &Vec<taxicab::Point>) -> taxicab::Grid<u32> {
    let mut grid = taxicab::grid();
    for coord in coords.iter() {
        grid.expand_bounds(*coord);
    }

    for point in grid.points_iter() {
        let sum: u32 = coords.iter().map(|c| c.distance(point)).sum();
        grid.insert(point, sum);
    }
    grid
}

fn count_lessthan(grid: &taxicab::Grid<u32>, limit: u32) -> u32 {
    grid.points_iter().map(|p| if *grid.get(p).unwrap() < limit { 1 } else { 0 }).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_file() {
        assert!(read_data().len() > 0);
    }

    #[test]
    fn distances() {
        let mut grid: taxicab::Grid<Coordinate> = taxicab::grid();
        grid.insert(taxicab::point(2, 2), Coordinate::Labeled("A".into()));
        grid.insert(taxicab::point(4, 2), Coordinate::Labeled("B".into()));
        grid.insert(taxicab::point(1, 1), Coordinate::Equidistant(10));
        grid.insert(taxicab::point(4, 4), Coordinate::Equidistant(10));

        populate_distances(&mut grid, taxicab::point(2, 2));
        assert_eq!(grid.get(taxicab::point(1, 1)), Some(&Coordinate::Nearest("A".into(), 2)));
        assert_eq!(grid.get(taxicab::point(4, 4)), Some(&Coordinate::Nearest("A".into(), 4)));

        populate_distances(&mut grid, taxicab::point(4, 2));
        assert_eq!(grid.get(taxicab::point(1, 1)),  Some(&Coordinate::Nearest("A".into(), 2)));
        assert_eq!(grid.get(taxicab::point(4, 4)),  Some(&Coordinate::Nearest("B".into(), 2)));
        assert_eq!(grid.get(taxicab::point(3, 2)),  Some(&Coordinate::Equidistant(1)));
        assert_eq!(grid.get(taxicab::point(3, 3)), Some(&Coordinate::Equidistant(2)));
        assert_eq!(grid.get(taxicab::point(2, 2)),  Some(&Coordinate::Labeled("A".into())));
        assert_eq!(grid.get(taxicab::point(4, 2)),  Some(&Coordinate::Labeled("B".into())));
    }

    #[test]
    fn example_pt1() {
        let coords: Vec<_> = vec!((1, 1), (1, 6), (8, 3), (3, 4), (5, 5), (8, 9)).iter()
            .map(|t| taxicab::point(t.0, t.1)).collect();

        let labels = label_points(&coords);
        let grid = coverage_for_points(&labels);

        // Note that, except for (3,4) and (5,5), the areas are actually infinite. If the grid's
        // bounds change these values will also need to be updated.
        assert_eq!(compute_area(&grid, taxicab::point(1, 1)), 7);
        assert_eq!(compute_area(&grid, taxicab::point(1, 6)), 9);
        assert_eq!(compute_area(&grid, taxicab::point(8, 3)), 12);
        assert_eq!(compute_area(&grid, taxicab::point(3, 4)), 9);
        assert_eq!(compute_area(&grid, taxicab::point(5, 5)), 17);
        assert_eq!(compute_area(&grid, taxicab::point(8, 9)), 10);

        assert_eq!(find_largest_enclosed_area(&labels, &grid), 17);
    }

    #[test]
    fn example_pt2() {
        let coords: Vec<_> = vec!((1, 1), (1, 6), (8, 3), (3, 4), (5, 5), (8, 9)).iter()
            .map(|t| taxicab::point(t.0, t.1)).collect();

        let grid = sum_distances(&coords);

        assert_eq!(count_lessthan(&grid, 32), 16);
    }
}

// https://en.wikipedia.org/wiki/Taxicab_geometry
// Referenced https://docs.rs/rusttype/0.5.2/src/rusttype/geometry.rs.html
// Other resources:
//   https://crates.io/crates/euclid - https://doc.servo.org/src/euclid/point.rs.html
mod taxicab {
    use std::collections::HashMap;
    use std::collections::hash_map::Entry;
    use std::cmp;
    use std::error;
    use std::fmt;
    use std::fmt::Write;
    use std::num;
    use std::result::Result;
    use std::str::FromStr;
    use regex::Regex;

    #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
    pub struct Point {
        pub x: i32,
        pub y: i32,
    }

    #[inline]
    pub fn point(x: i32, y: i32) -> Point {
        Point { x, y }
    }

    impl Point {
        pub fn distance(&self, other: Point) -> u32 {
            let dx = self.x - other.x;
            let dy = self.y - other.y;
            (dx.abs() + dy.abs()) as u32
        }
    }

    impl fmt::Display for Point {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "({}, {})", self.x, self.y)
        }
    }

    #[derive(Debug, Eq, PartialEq)]
    pub enum PointError {
        Malformed(String),
        InvalidInt(num::ParseIntError),
    }

    impl fmt::Display for PointError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                PointError::Malformed(ref str) => write!(f, "Malformed {}!", str),
                PointError::InvalidInt(ref err) => err.fmt(f),
            }
        }
    }

    impl error::Error for PointError {
        fn source(&self) -> Option<&(dyn error::Error + 'static)> {
            match *self {
                PointError::Malformed(_) => None,
                PointError::InvalidInt(ref err) => Some(err),
            }
        }
    }

    impl From<num::ParseIntError> for PointError {
        fn from(err: num::ParseIntError) -> PointError {
            PointError::InvalidInt(err)
        }
    }


    impl FromStr for Point {
        type Err = PointError;

        fn from_str(s: &str) -> Result<Self, PointError> {
            lazy_static! {
                static ref RE: Regex = Regex::new(r"^\(?([^(,]+),([^),]+)\)?$").unwrap();
            }

            if let Some(caps) = RE.captures(s) {
                let x: i32 = caps.get(1).expect("valid capture group").as_str().trim().parse()?;
                let y: i32 = caps.get(2).expect("valid capture group").as_str().trim().parse()?;
                return Ok(point(x, y));
            }

            Err(PointError::Malformed("No Match".into()))
        }
    }

    fn points_between(min: Point, max: Point) -> impl Iterator<Item = Point> {
        assert!(min.x <= max.x);
        assert!(min.y <= max.y);
        // https://users.rust-lang.org/t/product-of-iterators/2219
        // notice the x,y order is flipped, so that we iterate left-to-right first
        iproduct!(min.y..max.y+1, min.x..max.x+1).map(|p| point(p.1, p.0))
    }

    #[derive(Debug, PartialEq, Eq)]
    pub struct Grid<T> {
        map: HashMap<Point, T>,
        min_max: Option<(Point, Point)>,
    }

    #[inline]
    pub fn grid<T>() -> Grid<T> {
        Grid { map: HashMap::new(), min_max: None }
    }

    impl<T> Grid<T> {
        pub fn insert(&mut self, k: Point, v: T) {
            self.expand_bounds(k);
            self.map.insert(k, v);
        }

        pub fn expand_bounds(&mut self, p: Point) {
            match self.min_max {
                Some((min, max)) => {
                    let min = point(cmp::min(min.x, p.x), cmp::min(min.y, p.y));
                    let max = point(cmp::max(max.x, p.x), cmp::max(max.y, p.y));
                    self.min_max = Some((min, max));
                },
                None => self.min_max = Some((p, p)),
            }
        }

        pub fn bounds(&self) -> Option<(Point, Point)> {
            self.min_max
        }

        pub fn get(&self, k: Point) -> Option<&T> {
            self.map.get(&k)
        }

        pub fn get_entry(&mut self, k: Point) -> Entry<Point, T> {
            self.map.entry(k)
        }

        pub fn points_iter(&self) -> impl Iterator<Item = Point> {
            match self.min_max {
                Some((min, max)) => points_between(min, max),
                None => panic!("Can't get an empty iterator presently :("),
            }
        }
    }

    impl<T: fmt::Display> fmt::Display for Grid<T> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            // See also https://stackoverflow.com/q/30320083#comment92146674_30320443
            match self.min_max {
                Some((min, max)) => {
                    let mut out = String::new();
                    write!(&mut out, "Min: {} - Max: {}\n", min, max).expect("impossible");
                    let min_bound = point(min.x-1, min.y-1);
                    let max_bound = point(max.x+1, max.y+1);
                    for point in points_between(min_bound, max_bound) {
                        match self.map.get(&point) {
                            Some(t) => { out.push_str(&t.to_string()[0..1]); },
                            None => { out.push(' '); },
                        }
                        if point.x == max_bound.x {
                            out.push('\n');
                        }
                    }
                    write!(f, "{}", out)
                },
                None => write!(f, "Empty"),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use std::collections::hash_map::Entry::{Occupied, Vacant};
        use super::*;

        #[test]
        fn tc_point() {
            let a = point(1, 1);
            let b = point(8, 3);
            assert_eq!(a.distance(b), 9);
            assert_eq!(a.distance(b), b.distance(a));
        }

        #[test]
        fn parse_point() {
            assert_eq!("4, 4".parse::<Point>(), Ok(point(4, 4)));
            assert_eq!("(40,30)".parse::<Point>(), Ok(point(40, 30)));
            assert_eq!("(-3, -5)".parse::<Point>(), Ok(point(-3, -5)));
        }

        #[test]
        fn grid_str() {
            let mut grid: Grid<u32> = grid();
            assert_eq!(grid.to_string(), "Empty");
            grid.insert(point(1, 2), 10);
            assert_eq!(grid.to_string(), "Min: (1, 2) - Max: (1, 2)\n   \n 1 \n   \n");
            grid.insert(point(3, 4), 5);
            assert_eq!(
                grid.to_string(), "Min: (1, 2) - Max: (3, 4)\n     \n 1   \n     \n   5 \n     \n");
        }

        #[test]
        fn grid_points_between() {
            let mut grid: Grid<u32> = grid();
            grid.insert(point(2, 2), 10);
            grid.insert(point(1, 1), 5);
            let all_points: Vec<_> = grid.points_iter().map(|p| (p, grid.get(p))).collect();
            let expected = vec!(
                (point(1, 1), Some(&5)),
                (point(2, 1), None),
                (point(1, 2), None),
                (point(2, 2), Some(&10))
            );
            assert_eq!(all_points, expected);
        }

        #[test]
        fn grid_entry() {
            let mut grid: Grid<u32> = grid();
            let e = grid.get_entry(point(1, 1));
            let absent = match e {
                Vacant(_) => true,
                Occupied(_) => false,
            };
            assert!(absent);
            e.or_insert(10);
            assert_eq!(grid.get(point(1, 1)), Some(&10));
        }
    }
}