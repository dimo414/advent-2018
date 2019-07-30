use std::fs;

const REAL_DATA: &str = "data/day18.txt";
#[cfg(test)]
const TEST_DATA: &str = "data/day18-example.txt";

pub fn advent() {
    let mut landscape = read_data(REAL_DATA);
    landscape.tick(10);
    let counts = landscape.counts();
    println!("Resources after {} minutes:\t{}",
             landscape.minute(), counts.get(&State::TREES).unwrap() * counts.get(&State::YARD).unwrap());
    let cycle_size = landscape.find_cycle_size();
    println!("Found cycle after: {} minutes - Cycle Size: {}", landscape.minute(), cycle_size);
    landscape.tick_to(1000000000);
    let counts = landscape.counts();
    println!("Resources after {} minutes:\t{}",
             landscape.minute(), counts.get(&State::TREES).unwrap() * counts.get(&State::YARD).unwrap());
}

fn read_data(path: &str) -> Landscape {
    fs::read_to_string(path).expect("Cannot open").parse().unwrap()
}

mod landscape {
    use std::collections::hash_map::{HashMap, Entry};
    use std::fmt;
    use std::str::FromStr;
    use crate::euclid::{point, Point, vector};

    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
    pub enum State {
        OPEN,
        TREES,
        YARD,
    }

    pub struct Landscape {
        acres: HashMap<Point, State>,
        minute: usize,
        cycle_size: Option<usize>,
    }

    impl Landscape {
        pub fn counts(&self) -> HashMap<State, usize> {
            let mut counts = HashMap::new();
            for state in self.acres.values() {
                match counts.entry(*state) {
                    Entry::Vacant(count) => { count.insert(1); },
                    Entry::Occupied(mut count) => { *count.get_mut() += 1; },
                }
            }
            counts
        }

        pub fn minute(&self) -> usize {
            self.minute
        }

        pub fn tick_to(&mut self, minute: usize) {
            assert!(minute >= self.minute);
            self.tick(minute - self.minute);
        }

        pub fn tick(&mut self, minutes: usize) {
            let minutes= match self.cycle_size {
                Some(size) => minutes % size,
                None => minutes,
            };
            for _ in {0..minutes} {
                self.tick_impl();
            }
        }

        pub fn find_cycle_size(&mut self) -> usize {
            let mut points = self.acres.keys().cloned().collect::<Vec<_>>();
            points.sort();
            let points = points;

            let to_cache_key = |acres: &HashMap<Point, State>|
                points.iter().map(|p| *acres.get(p).expect("Unexpected point")).collect::<Vec<State>>();

            let mut seen_states = HashMap::new();
            loop {
                if let Some(start) = seen_states.insert(to_cache_key(&self.acres), seen_states.len()) {
                    self.cycle_size = Some(seen_states.len() - start);
                    break;
                }
                assert!(seen_states.len() < 1000); // sanity check
                self.tick_impl();
            }
            self.cycle_size.expect("Was just set")
        }

        fn tick_impl(&mut self) {
            let mut new_acres = HashMap::new();

            for (coord, state) in self.acres.iter() {
                let neighbors = self.neighbors(coord);
                let new_state = match state {
                    State::OPEN => {
                        if neighbors.into_iter().filter(|&s| s == State::TREES).count() >= 3 {
                            State::TREES
                        } else {
                            *state
                        }
                    },
                    State::TREES => {
                        if neighbors.into_iter().filter(|&s| s == State::YARD).count() >= 3 {
                            State::YARD
                        } else {
                            *state
                        }
                    },
                    State::YARD => {
                        if neighbors.contains(&State::YARD) && neighbors.contains(&State::TREES) {
                            *state
                        } else {
                            State::OPEN
                        }
                    },
                };
                new_acres.insert(*coord, new_state);
            }

            self.minute += 1;
            self.acres = new_acres
        }

        fn neighbors(&self, coord: &Point) -> Vec<State> {
            let mut result = Vec::new();
            for x in {-1..2} {
                for y in {-1..2} {
                    if x == 0 && y == 0 { continue; }
                    if let Some(state) = self.acres.get(&(coord + vector(x, y))) {
                        result.push(*state);
                    }
                }
            }
            result
        }
    }

    impl FromStr for Landscape {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, String> {
            let mut acres =  HashMap::new();
            let mut coord = point(0, 0);

            for c in s.chars() {
                match c {
                    '.' => { acres.insert(coord, State::OPEN); },
                    '|' => { acres.insert(coord, State::TREES); },
                    '#' => { acres.insert(coord, State::YARD); },
                    '\n' => { coord = point(-1, coord.y + 1); },
                    _ => { return Err(format!("Unexpected char {} at {}", c, coord)); },
                };
                coord = point(coord.x + 1, coord.y);
            }

            Ok(Landscape { acres, minute: 0, cycle_size: None })
        }
    }

    impl fmt::Display for Landscape {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            if self.acres.is_empty() {
                return write!(f, "");
            }

            let mut out = String::new();
            // Two searches isn't ideal, but it's fine
            let max_x = self.acres.keys().map(|p| p.x).max().expect("isn't empty");
            let max_y = self.acres.keys().map(|p| p.y).max().expect("isn't empty");

            for y in 0..max_y+1 {
                for x in 0..max_x+1 {
                    let coord = point(x, y);
                    let c = match self.acres.get(&coord) {
                        Some(State::OPEN) => '.',
                        Some(State::TREES) => '|',
                        Some(State::YARD) => '#',
                        None => panic!("Unexpected coord: {}", coord),
                    };
                    out.push(c);
                }
                out.push('\n');
            }
            write!(f, "{}", out)
        }
    }
}
pub use landscape::{Landscape, State};

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use super::*;

    #[test]
    fn read_file() {
        read_data(REAL_DATA);
    }

    #[test]
    fn example() {
        let mut landscape = read_data(TEST_DATA);
        landscape.tick(10);
        println!("{}", landscape);
        let counts: HashMap<State, usize> =
            [(State::OPEN, 100-37-31),
                (State::TREES, 37),
                (State::YARD, 31)]
                .iter().cloned().collect();
        assert_eq!(landscape.counts(), counts);

        assert_eq!(landscape.find_cycle_size(), 1);

        let counts: HashMap<State, usize> = [(State::OPEN, 100)]
                .iter().cloned().collect();
        assert_eq!(landscape.counts(), counts);
    }
}