use std::fs;
use regex::Regex;
use crate::device::{Device,Opcode};

pub fn advent() {
    let (samples, instructions) = read_data();

    let mut three_plus = 0;
    for Sample(before, instruction, after) in samples.iter() {
        let mut finder = Finder::new();
        let count = finder.consider(*before, *instruction, *after);
        if count >= 3 {
            three_plus += 1;
        }
    }
    println!("{} samples matched three or more opcodes", three_plus);

    let mut finder = Finder::new();
    for Sample(before, instruction, after) in samples.iter() {
        let count = finder.consider(*before, *instruction, *after);
        // Could break early if we're already solved, but (with the current structure) checking
        // isn't particularly cheap.
    }
    let opcodes = finder.solution();

    let mut device = Device::new([0,0,0,0]);
    for instr in instructions {
        let opcode = opcodes.get(&instr[0]).expect("Invalid opcode num");
        device.exec(*opcode, instr[1], instr[2], instr[3]);
    }
    println!("Device state: {:?}", device.get_registers());
}

fn read_data() -> (Vec<Sample>, Vec<[usize; 4]>) {
    let contents = fs::read_to_string("data/day16.txt").expect("Cannot open");
    let halves: Vec<_>  = contents.splitn(2, "\n\n\n\n").collect();
    match halves.as_slice() {
        [first, second] => {
            (first.split("\n\n").map(|s| parse_sample(s).expect("Invalid data")).collect(),
             second.lines().map(|s| to_arr(s).expect("Invalid data")).collect())
        },
        _ => panic!("Unexpected"),
    }
}

fn to_arr(input: &str) -> Result<[usize; 4], String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^(\d+)[^\d]*(\d+)[^\d]*(\d+)[^\d]*(\d+)$").unwrap();
    }

    let caps = regex_captures!(RE, input)?;
    let a: usize = capture_group!(caps, 1).parse().map_err(|_| "NOPE".to_string())?;
    let b: usize = capture_group!(caps, 2).parse().map_err(|_| "NOPE".to_string())?;
    let c: usize = capture_group!(caps, 3).parse().map_err(|_| "NOPE".to_string())?;
    let d: usize = capture_group!(caps, 4).parse().map_err(|_| "NOPE".to_string())?;
    Ok([a,b,c,d])
}

#[derive(Debug, Eq, PartialEq)]
struct Sample([usize; 4], [usize; 4], [usize; 4]);

fn parse_sample(input: &str) -> Result<Sample, String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"Before: \[([^\]]+)\]\n([^\n]+)\nAfter:  \[([^\]]+)\]").unwrap();
    }

    let caps = regex_captures!(RE, input)?;
    let before: [usize; 4] = to_arr(capture_group!(caps, 1))?;
    let instruction: [usize; 4] = to_arr(capture_group!(caps, 2))?;
    let after: [usize; 4] = to_arr(capture_group!(caps, 3))?;
    Ok(Sample(before, instruction, after))
}

mod opcode_finder {
    // Could use HashMap/Set, but BTree keeps things ordered, and for these small structures
    // might be faster. Who knows...
    use std::collections::{BTreeMap, BTreeSet};
    use super::*;

    #[derive(Debug)]
    pub struct Finder {
        candidates: BTreeMap<usize, BTreeSet<Opcode>>,
    }

    impl Finder {
        fn extract_only(s: &BTreeSet<Opcode>) -> Option<Opcode> {
            match s.len() {
                1 => Some(*s.iter().nth(0).expect("Must exist")),
                _ => None,
            }
        }

        pub fn new() -> Finder {
            let codes: BTreeSet<Opcode> = Opcode::iter().map(|o| *o).collect();
            let mut candidates = BTreeMap::new();
            for opcode in 0..16 {
                candidates.insert(opcode, codes.clone());
            }
            Finder { candidates }
        }

        fn solved(&self) -> BTreeMap<usize, Opcode> {
            self.candidates.iter()
                .flat_map(|(n, s)| Finder::extract_only(s).into_iter().map(move |o| (*n, o)))
                .collect()
        }

        pub fn solution(&self) -> BTreeMap<usize, Opcode> {
            // to check if solved:
            // !self.candidates.values().any(|m| m.len() != 1)
            let solved = self.solved();
            let codes: BTreeSet<Opcode> = Opcode::iter().map(|o| *o).collect();
            let unsolved: BTreeSet<Opcode> = codes
                .difference(&solved.values().map(|o| *o).collect())
                .map(|o| *o).collect();
            if ! unsolved.is_empty() {
                panic!("Still have multiple candidate solutions for {:?}:\n:{:?}", unsolved, self);
            }
            solved
        }

        #[cfg(test)]
        pub fn candidates_for(&self, code: usize) -> &BTreeSet<Opcode> {
            self.candidates.get(&code).expect("Invalid code")
        }

        // returns the number of still-valid opcodes for this opcode number
        pub fn consider(&mut self, input_state: [usize; 4], instr: [usize; 4], output_state: [usize; 4]) -> usize {
            // This is a bit gross, but I guess we can't pattern-match arrays directly
            let (opcode_num, input_a, input_b, output) =
                (instr[0], instr[1], instr[2], instr[3]);
            let codes = self.candidates.get_mut(&opcode_num).expect("Invalid code");

            if codes.len() > 1 {
                for opcode in codes.clone() {
                    let mut device = Device::new(input_state);
                    device.exec(opcode, input_a, input_b, output);
                    if device.get_registers() != output_state {
                        assert!(codes.remove(&opcode));
                    }
                }
            }

            let codes_len = codes.len();
            if codes_len == 1 {
                // if we've eliminated all-but-one code for one number, that code can't be valid for
                // any other number
                let opcode = *codes.iter().nth(0).expect("Must exist");
                for (code_num, candidates) in self.candidates.iter_mut() {
                    if code_num != &opcode_num {
                        candidates.remove(&opcode);
                    }
                }
            }
            codes_len
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn create_sample() -> Finder {
            let mut finder = Finder::new();
            let count = finder.consider([3, 2, 1, 1], [9, 2, 1, 2], [3, 2, 2, 1]);
            let expected: BTreeSet<Opcode> =
                vec!(Opcode::MULR, Opcode::ADDI, Opcode::SETI).into_iter().collect();
            assert_eq!(finder.candidates_for(9), &expected);
            assert_eq!(count, 3);
            finder
        }

        #[test]
        fn sample() {
            create_sample();
        }

        parameterized_test!{ compose, (input, output, expected_code), {
            let mut finder = create_sample();
            // consider a second operation that narrows us down to one
            let count = finder.consider(input, [9, 2, 1, 2], output);
            assert_eq!(count, 1);
            let expected: BTreeSet<Opcode> = vec!(expected_code).into_iter().collect();
            assert_eq!(finder.candidates_for(9), &expected);
        }}
        compose! {
            mulr: ([3, 2, 2, 1], [3, 2, 4, 1], Opcode::MULR),
            addi: ([3, 2, 2, 1], [3, 2, 3, 1], Opcode::ADDI),
            seti: ([3, 2, 2, 1], [3, 2, 2, 1], Opcode::SETI),
        }
    }
}
pub use self::opcode_finder::Finder;

#[cfg(test)]
mod tests {
    use super::*;

    parameterized_test!{ parse_arr, (s, expected), {
        assert_eq!(to_arr(s), Ok(expected));
    }}
    parse_arr! {
        with_commas: ("1, 2, 3, 4", [1, 2, 3, 4]),
        whitespace: ("2 3 4 5", [2, 3, 4, 5]),
    }

    #[test]
    fn try_parse_sample() {
        let sample = "Before: [0, 2, 2, 2]\n4 2 3 2\nAfter:  [0, 2, 5, 2]";
        let result = parse_sample(sample);
        assert_eq!(result, Ok(Sample([0, 2, 2, 2], [4, 2, 3, 2], [0, 2, 5, 2])))
    }
}