use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn advent() {
    let claims = read_data();
    let mut fabric = grid::Grid::new(1000, 1000);
    for claim in &claims {
        record_claim(&claim, &mut fabric);
    }

    let mut conflicts = 0;
    for inch in fabric.iter() {
        if *inch.2 > 1 {
            conflicts += 1;
        }
    }
    println!("Conflicts: {}", conflicts);

    for claim in &claims {
        if check_claim(&claim, &fabric) {
            println!("Valid Claim: {}", claim.id);
            return;
        }
    }
    panic!("No valid claims found.");
}

fn record_claim(claim: &claim::Claim, fabric: &mut grid::Grid) {
    for x in claim.x..claim.x+claim.w {
        for y in claim.y..claim.y+claim.h {
            fabric.incr(x, y);
        }
    }
}

fn check_claim(claim: &claim::Claim, fabric: &grid::Grid) -> bool {
    for x in claim.x..claim.x+claim.w {
        for y in claim.y..claim.y+claim.h {
            let claims = fabric[(x, y)];
            if claims < 1 {
                panic!("Coordinate [{}, {}] should have already been claimed", x, y);
            }
            if claims > 1 {
                return false;
            }
        }
    }
    return true;
}

fn read_data() -> Vec<claim::Claim> {
    let reader = BufReader::new(File::open("data/day3.txt").expect("Cannot open"));
    // http://xion.io/post/code/rust-iter-patterns.html suggests .collect()-ing the Vec<Result>
    // but it seems to be more verbose (in this case) than just unwrapping in the map(), perhaps
    // because std::io::Lines is an iterator. We also don't really need the temporary Vec.
    //let lines: Result<Vec<_>, _> = reader.lines().collect();
    //lines.unwrap().iter().map(|l| l.parse().unwrap()).collect()
    reader.lines().map(|l| l.unwrap().parse().unwrap()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_file() {
        assert!(read_data().len() > 0);
    }

    #[test]
    fn record_claims() {
        let expected = {
            let mut expected = grid::Grid::new(11, 9);
            for x in { 3..8 } {
                for y in { 2..6 } {
                    expected.incr(x, y);
                }
            }
            expected
        };


        let claim: claim::Claim = "#123 @ 3,2: 5x4".parse::<>().unwrap();

        let mut fabric = grid::Grid::new(11, 9);
        record_claim(&claim, &mut fabric);
        assert_eq!(fabric, expected);
    }

    #[test]
    fn check_claims() {
        let claims: Vec<_> = ["#1 @ 1,3: 4x4", "#2 @ 3,1: 4x4", "#3 @ 5,5: 2x2"].iter()
            .map(|s| s.parse().unwrap()).collect();
        let mut fabric = grid::Grid::new(8, 8);

        for claim in &claims {
            record_claim(&claim, &mut fabric);
        }

        let mut valid_claim= 0;
        for claim in &claims {
            if check_claim(&claim, &fabric) {
                valid_claim = claim.id;
            }
        }
        assert_eq!(valid_claim, 3);
    }
}

// Models a two dimensional grid backed by a single Vec.
// Initially considered a 2D array, like
// https://www.reddit.com/r/rust/comments/3l0dau/dynamic_heapallocated_multidimensional_arrays/cv2fn5y
// but the type signatures were odd, at best.
mod grid {
    use std::fmt;
    use std::fmt::Write;
    use std::ops::Index;

    #[derive(Debug, Eq, PartialEq)]
    pub struct Grid {
        grid: Vec<i32>,
        width: usize,
    }

    impl Grid {
        pub fn new(width: usize, height: usize) -> Grid {
            Grid { grid: vec![0; width * height], width: width }
        }

        pub fn get(&self, x: usize, y: usize) -> Option<&i32> {
            self.grid.get(x + self.width * y)
        }

        pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut i32> {
            self.grid.get_mut(x + self.width * y)
        }

        pub fn incr(&mut self, x: usize, y: usize) {
            *self.get_mut(x, y).unwrap() += 1;
        }

        pub fn iter<'a>(&'a self) -> GridIter<'a> {
            GridIter {
                inner: self,
                pos: 0,
            }
        }
    }

    impl Index<(usize, usize)> for Grid {
        type Output = i32;

        fn index(&self, coordinate: (usize, usize)) -> &i32 {
            self.get(coordinate.0, coordinate.1).unwrap()
        }
    }

    impl fmt::Display for Grid {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            // See also https://stackoverflow.com/q/30320083#comment92146674_30320443
            let mut out = String::new();
            for i in 0..self.grid.len() {
                let value = match self.grid[i] {
                    v if v < 10 => v.to_string(),
                    _ => "*".to_string(),
                };
                write!(&mut out, " {}", value).expect("writing to string");
                if i % self.width == self.width - 1 {
                    out.push('\n');
                }
            }
            write!(f, "{}", out)
        }
    }

    // https://blog.guillaume-gomez.fr/articles/2017-03-09+Little+tour+of+multiple+iterators+implementation+in+Rust
    pub struct GridIter<'a> {
        inner: &'a Grid,
        pos: usize,
    }

    impl<'a> Iterator for GridIter<'a> {
        type Item = (usize, usize, &'a i32);

        fn next(&mut self) -> Option<Self::Item> {
            if self.pos >= self.inner.grid.len() {
                None
            } else {
                self.pos += 1;
                let pos = self.pos - 1;
                let x = pos % self.inner.width;
                let y = pos / self.inner.width;
                self.inner.get(x, y).map(|v| (x, y, v))
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn grid_str() {
            let grid = Grid::new(2, 3);
            assert_eq!(grid.to_string(), " 0 0\n 0 0\n 0 0\n");
        }

        #[test]
        fn grid_incr() {
            let mut grid = Grid::new(5, 5);
            assert_eq!(grid[(2, 2)], 0);
            grid.incr(2, 2);
            assert_eq!(grid[(2, 2)], 1);
        }

        #[test]
        fn grid_bounds() {
            let mut grid = Grid::new(3, 2);
            *grid.get_mut(0, 0).unwrap() = 1;
            *grid.get_mut(2, 0).unwrap() = 2;
            *grid.get_mut(0, 1).unwrap() = 3;
            *grid.get_mut(2, 1).unwrap() = 4;
            assert_eq!(grid.to_string(), " 1 0 2\n 3 0 4\n");
        }

        #[test]
        fn grid_iter() {
            let grid = Grid::new(2, 2);
            let mut grid_iter = grid.iter();
            assert_eq!(grid_iter.next().unwrap(), (0, 0, &0));
            assert_eq!(grid_iter.next().unwrap(), (1, 0, &0));
            assert_eq!(grid_iter.next().unwrap(), (0, 1, &0));
            assert_eq!(grid_iter.next().unwrap(), (1, 1, &0));
            assert_eq!(grid_iter.next(), None);
        }
    }
}

mod claim {
    use std::error;
    use std::fmt;
    use std::num;
    use std::str::FromStr;
    use regex::{Captures, Regex};

    // https://blog.burntsushi.net/rust-error-handling/
    #[derive(Debug)]
    pub enum ClaimError {
        Malformed(String),
        InvalidInt(num::ParseIntError),
        Unknown, // only necessary because Claim parsing demos different approaches
    }

    impl fmt::Display for ClaimError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                ClaimError::Malformed(ref str) => write!(f, "Malformed {}!", str),
                ClaimError::InvalidInt(ref err) => err.fmt(f),
                ClaimError::Unknown => write!(f, "UNKNOWN"),
            }
        }
    }

    impl error::Error for ClaimError {
        fn source(&self) -> Option<&(dyn error::Error + 'static)> {
            match *self {
                ClaimError::Malformed(_) => None,
                ClaimError::InvalidInt(ref err) => Some(err),
                ClaimError::Unknown => None,
            }
        }
    }

    impl From<num::ParseIntError> for ClaimError {
        fn from(err: num::ParseIntError) -> ClaimError {
            ClaimError::InvalidInt(err)
        }
    }

    #[derive(Debug, PartialEq)]
    pub struct Claim {
        pub id: usize,
        pub x: usize,
        pub y: usize,
        pub w: usize,
        pub h: usize
    }

    type Result<T> = std::result::Result<T, ClaimError>;

    impl FromStr for Claim {
        type Err = ClaimError;

        fn from_str(s: &str) -> Result<Self> {
            lazy_static! {
                static ref RE: Regex =
                    Regex::new(r"^#([0-9]+) @ ([0-9]+),([0-9]+): ([0-9]+)x([0-9]+)$").unwrap();
            }
            let caps: Captures = RE.captures(s).ok_or_else(|| ClaimError::Malformed(s.to_string()))?;

            // Several different get_as_int() implementations, for the sake of example.
            // Each implementation is roughly equivalent, save for the exact error semantics,
            // but the last implementation is probably preferable

            let get_as_int = |caps: &Captures, i|
                caps.get(i).ok_or(ClaimError::Unknown)?
                    .as_str().parse::<usize>().map_err(ClaimError::InvalidInt);
            let id = get_as_int(&caps, 1)?;

            // Convert capture to Result, then if OK parse capture to Result
            let get_as_int = |caps: &Captures, i|
                caps.get(i).ok_or(ClaimError::Unknown)
                    .and_then(|c| c.as_str().parse::<usize>().map_err(ClaimError::InvalidInt));
            let x = get_as_int(&caps, 2)?;

            // Wrap the parse result in an Optional if capture is present, then unwrap to Result
            let get_as_int = |caps: &Captures, i|
                caps.get(i).map(|c| c.as_str().parse::<usize>().map_err(ClaimError::InvalidInt))
                    .unwrap_or_else(|| Err(ClaimError::Unknown));
            let y = get_as_int(&caps, 3)?;

            // Convert parsed Result to Optional, then flatmap (and_then) it with the capture
            // Optional, then convert Optional to Result
            let get_as_int = |caps: &Captures, i|
                caps.get(i).and_then(|c| c.as_str().parse::<usize>().ok())
                    .ok_or_else(|| ClaimError::Unknown);
            let w = get_as_int(&caps, 4)?;

            // Ignore capturing group errors, and rely on the From trait impl to convert from
            // Result<usize, ParseIntError> to usize with the ? operator
            let get_as_int = |caps: &Captures, i|
                caps.get(i).expect("valid capture group").as_str().parse::<usize>();
            let h = get_as_int(&caps, 5)?;

            Ok(Claim { id: id, x: x, y: y, w: w, h: h })
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn examples_part1() {
            let claim: Claim = "#123 @ 3,2: 5x4".parse().unwrap();
            assert_eq!(claim, Claim { id: 123, x: 3, y: 2, w: 5, h: 4});
        }
    }
}