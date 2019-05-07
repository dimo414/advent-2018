use std::collections::HashSet;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn advent() {
    let mut pots = read_data();
    pots.advance(20);
    println!("{:?}", pots);
    pots.advance(50000000000 - 20);
    println!("{}: {}", pots.generation(), pots.score());
}

lazy_static! {
    static ref STATE_RE: Regex = Regex::new(r"^initial state: ([#.]+)$").unwrap();
    static ref RULE_RE: Regex = Regex::new(r"^([#.]{5}) => ([#.])$").unwrap();
}

fn read_data() -> Pots {
    let reader = BufReader::new(File::open("data/day12.txt").expect("Cannot open"));
    let lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();

    let initial_state = STATE_RE.captures(&lines[0]).expect("must match")
        .get(1).expect("valid capture group").as_str();
    Pots::new(initial_state, &lines[2..])
}

fn to_bool(c: char) -> bool {
    match c {
        '#' => true,
        '.' => false,
        _ => panic!("invalid char: {:?}", c),
    }
}

fn window(s: &str) -> [bool;5] {
    assert_eq!(s.len(), 5);

    // https://stackoverflow.com/a/29570662/113632
    let mut array = [false; 5];
    let v: Vec<bool> = s.chars().map(to_bool).collect();
    array.copy_from_slice(&v);
    array
}

fn rules(lines: &[String]) -> HashSet<[bool;5]> {
    let mut set = HashSet::new();
    for line in lines {
        let caps = RULE_RE.captures(&line).expect("must match");
        let window = window(caps.get(1).expect("valid capture group").as_str());
        let is_alive = caps.get(2).expect("valid capture group").as_str();
        if to_bool(is_alive.chars().next().unwrap()) {
            set.insert(window);
        }
    }
    assert!(!set.contains(&[false;5]), "Cannot create life from nothing");
    set
}

mod pots {
    use super::to_bool;
    use std::cmp;
    use std::collections::HashSet;
    use std::fmt;

    pub struct Pots {
        stable_shift: Option<i64>,
        state: Vec<bool>,
        rules: HashSet<[bool;5]>,
        offset: i64,
        generation: u64,
    }

    impl Pots {
        pub fn new(initial: &str, rules: &[String]) -> Pots {
            let state = initial.chars().map(to_bool).collect();
            let rules = super::rules(rules);
            Pots { stable_shift: None, state, rules, offset: 0, generation: 0 }
        }

        fn shift_offsets(&mut self) {
            let first_true = self.state.iter().position(|&x| x == true)
                .expect("Expected a true element");
            let shift_by = 4 - cmp::min(4, first_true); // no need to shift by more than 4
            if shift_by > 0 {
                let mut new_state = Vec::new();
                new_state.resize(shift_by, false);
                new_state.extend_from_slice(&self.state);
                self.state = new_state;
                self.offset += shift_by as i64;
            }

            let last_true = self.state.iter().rposition(|&x| x == true)
                .expect("Expected a true element");
            let extend_to = cmp::max(self.state.len(), last_true + 4);
            self.state.resize(extend_to, false);
        }

        fn check_stability(&mut self, new_state: &Vec<bool>) {
            assert!(self.stable_shift.is_none());
            let bounds = |v: &Vec<bool>| (
                v.iter().position(|&x| x == true).unwrap(),
                v.iter().rposition(|&x| x == true).unwrap());

            let cur_bounds = bounds(&self.state);
            let new_bounds = bounds(new_state);

            if self.state[cur_bounds.0..cur_bounds.1] == new_state[new_bounds.0..new_bounds.1] {
                self.stable_shift = Some(new_bounds.0 as i64 - cur_bounds.0 as i64);
            }
        }

        fn advance_impl(&mut self) {
            assert!(self.stable_shift.is_none());
            self.shift_offsets();

            let mut new_state = Vec::new();
            new_state.resize(self.state.len(), false);

            // We can ignore the first two and last two pots, becuase shift_offsets ensures they're
            // [FFFF*]/[*FFFF] which means (since further pots are also false) the first two and
            // last two pots cannot germinate in this generation. Note that `..... => #` is banned.
            for i in 2..self.state.len() - 2 {
                let mut array = [false; 5];
                array.copy_from_slice(&self.state[i - 2..i + 3]);
                if self.rules.contains(&array) {
                    new_state[i] = true;
                }
            }

            self.check_stability(&new_state);

            self.state = new_state;
        }

        pub fn advance(&mut self, generations: u64) {
            let mut i = generations;
            while i > 0 && self.stable_shift.is_none() {
                self.advance_impl();
                i -= 1;
            }
            self.offset -= self.stable_shift.unwrap_or(0) * i as i64;
            self.generation += generations;
        }

        pub fn score(&self) -> i64 {
            let mut sum = 0;
            for i in { 0..self.state.len() } {
                if self.state[i] {
                    sum += i as i64 - self.offset;
                }
            }
            sum
        }

        pub fn generation(&self) -> u64 {
            self.generation
        }

        pub fn is_stable(&self) -> bool {
            self.stable_shift.is_some()
        }
    }

    impl fmt::Display for Pots {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let state = self.state.iter().map(|&x| if x { "#" } else { "." })
                .collect::<String>();
            let offset = std::iter::repeat(".").take(cmp::max(0, 0-self.offset) as usize)
                .collect::<String>();
            write!(f, "{:3}: {}{}", self.generation, offset, state)
        }
    }

    impl fmt::Debug for Pots {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let offset_mark = std::iter::repeat(" ").take(cmp::max(0, self.offset) as usize)
                .collect::<String>();
            write!(f, "{}\n     {}^  Score:{}{}",
                   self, offset_mark, self.score(), if self.is_stable() { " STABLE" } else { "" })
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn one_generation() {
            let rules=
                &vec!("..#.. => #").iter().map(|s| s.to_string()).collect::<Vec<_>>();

            let mut pots = Pots::new("#..#.#..##......###...###", rules);
            assert_eq!(format!("{:?}", pots), "  0: #..#.#..##......###...###\n     ^  Score:145");
            assert_eq!(pots.score(), 145); // value isn't documented

            pots.advance(1);
            assert_eq!(format!("{:?}", pots),
                       "  1: ....#...........................\n         ^  Score:0");
            assert!(!pots.is_stable()); // first stable generation isn't stable *yet*

            pots.advance(1);
            assert!(pots.is_stable());
            assert_eq!(format!("{:?}", pots),
                       "  2: ....#...........................\n         ^  Score:0 STABLE");
        }
    }
}
pub use self::pots::Pots;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn read_file() {
        assert_eq!(read_data().score(), 2434);
    }

    #[test]
    fn windows() {
        assert_eq!(window("#.#.#"), [true, false, true, false, true]);
    }

    fn example_rules() -> Vec<String> {
        vec!(
            "...## => #",
            "..#.. => #",
            ".#... => #",
            ".#.#. => #",
            ".#.## => #",
            ".##.. => #",
            ".#### => #",
            "#.#.# => #",
            "#.### => #",
            "##.#. => #",
            "##.## => #",
            "###.. => #",
            "###.# => #",
            "####. => #")
            .iter().map(|s| s.to_string()).collect::<Vec<_>>()
    }

    #[test]
    fn check_example() {
        let rules = rules(&example_rules());
        assert_eq!(rules.len(), 14);
    }

    #[test]
    fn example() {
        let mut pots = Pots::new("#..#.#..##......###...###", &example_rules());
        for _ in 0..20 {
            pots.advance(1);
        }
        assert_eq!(pots.score(), 325);
        pots.advance(66);
        assert!(!pots.is_stable());
        pots.advance(1);
        assert!(pots.is_stable());
    }
}