use std::fs;
use regex::Regex;

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
        if count == 1 {
            // println!("{:?}", finder);
        }
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

//#[allow(non_camel_case_types)]
mod opcode {
    use std::slice::Iter;
    use self::Opcode::*;

    #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
    pub enum Opcode {
        ADDR,
        ADDI,
        MULR,
        MULI,
        BANR,
        BANI,
        BORR,
        BORI,
        SETR,
        SETI,
        GTIR,
        GTRI,
        GTRR,
        EQIR,
        EQRI,
        EQRR,
    }

    impl Opcode {
        pub fn iter() -> Iter<'static, Opcode> {
            static OPCODES: [Opcode; 16] = [
                ADDR,
                ADDI,
                MULR,
                MULI,
                BANR,
                BANI,
                BORR,
                BORI,
                SETR,
                SETI,
                GTIR,
                GTRI,
                GTRR,
                EQIR,
                EQRI,
                EQRR];
            OPCODES.into_iter()
        }
    }
}
pub use self::opcode::Opcode;

mod device {
    use super::*;

    pub struct Device {
        regs: [usize; 4],
    }

    impl Device {
        pub fn new(regs: [usize; 4]) -> Device {
            Device { regs }
        }

        pub fn get_registers(&self) -> [usize; 4] {
            self.regs
        }

        pub fn exec(&mut self, instruction: Opcode, input_a: usize, input_b: usize, output: usize) {
            match instruction {
                Opcode::ADDR => self.addr(input_a, input_b, output),
                Opcode::ADDI => self.addi(input_a, input_b, output),
                Opcode::MULR => self.mulr(input_a, input_b, output),
                Opcode::MULI => self.muli(input_a, input_b, output),
                Opcode::BANR => self.banr(input_a, input_b, output),
                Opcode::BANI => self.bani(input_a, input_b, output),
                Opcode::BORR => self.borr(input_a, input_b, output),
                Opcode::BORI => self.bori(input_a, input_b, output),
                Opcode::SETR => self.setr(input_a, input_b, output),
                Opcode::SETI => self.seti(input_a, input_b, output),
                Opcode::GTIR => self.gtir(input_a, input_b, output),
                Opcode::GTRI => self.gtri(input_a, input_b, output),
                Opcode::GTRR => self.gtrr(input_a, input_b, output),
                Opcode::EQIR => self.eqir(input_a, input_b, output),
                Opcode::EQRI => self.eqri(input_a, input_b, output),
                Opcode::EQRR => self.eqrr(input_a, input_b, output),
            }
        }

        // Addition

        // (add register) stores into register C the result of adding register A and register B.
        fn addr(&mut self, reg_a: usize, reg_b: usize, reg_c: usize) {
            self.regs[reg_c] = self.regs[reg_a] + self.regs[reg_b];
        }

        // (add immediate) stores into register C the result of adding register A and value B.
        fn addi(&mut self, reg_a: usize, value_b: usize, reg_c: usize) {
            self.regs[reg_c] = self.regs[reg_a] + value_b;
        }

        // Multiplication

        // (multiply register) stores into register C the result of multiplying register A and
        // register B.
        fn mulr(&mut self, reg_a: usize, reg_b: usize, reg_c: usize) {
            self.regs[reg_c] = self.regs[reg_a] * self.regs[reg_b];
        }

        // (multiply immediate) stores into register C the result of multiplying register A and
        // value B.
        fn muli(&mut self, reg_a: usize, value_b: usize, reg_c: usize) {
            self.regs[reg_c] = self.regs[reg_a] * value_b;
        }

        // Bitwise AND

        // (bitwise AND register) stores into register C the result of the bitwise AND of register A
        // and register B.
        fn banr(&mut self, reg_a: usize, reg_b: usize, reg_c: usize) {
            self.regs[reg_c] = self.regs[reg_a] & self.regs[reg_b];
        }

        // (bitwise AND immediate) stores into register C the result of the bitwise AND of
        // register A and value B.
        fn bani(&mut self, reg_a: usize, value_b: usize, reg_c: usize) {
            self.regs[reg_c] = self.regs[reg_a] & value_b;
        }

        // Bitwise OR

        // (bitwise OR register) stores into register C the result of the bitwise OR of register A
        // and register B.
        fn borr(&mut self, reg_a: usize, reg_b: usize, reg_c: usize) {
            self.regs[reg_c] = self.regs[reg_a] | self.regs[reg_b];
        }

        // (bitwise OR immediate) stores into register C the result of the bitwise OR of register A
        // and value B.
        fn bori(&mut self, reg_a: usize, value_b: usize, reg_c: usize) {
            self.regs[reg_c] = self.regs[reg_a] | value_b;
        }

        // Assignment

        // (set register) copies the contents of register A into register C. (Input B is ignored.)
        fn setr(&mut self, reg_a: usize, _unused_b: usize, reg_c: usize) {
            self.regs[reg_c] = self.regs[reg_a];
        }

        // (set immediate) stores value A into register C. (Input B is ignored.)
        fn seti(&mut self, value_a: usize, _unused_b: usize, reg_c: usize) {
            self.regs[reg_c] = value_a;
        }

        // Greater-than testing

        // (greater-than immediate/register) sets register C to 1 if value A is greater than
        // register B. Otherwise, register C is set to 0.
        fn gtir(&mut self, value_a: usize, reg_b: usize, reg_c: usize) {
            self.regs[reg_c] = if value_a > self.regs[reg_b] { 1 } else { 0 };
        }

        // (greater-than register/immediate) sets register C to 1 if register A is greater than
        // value B. Otherwise, register C is set to 0.
        fn gtri(&mut self, reg_a: usize, value_b: usize, reg_c: usize) {
            self.regs[reg_c] = if self.regs[reg_a] > value_b { 1 } else { 0 };
        }

        // (greater-than register/register) sets register C to 1 if register A is greater than
        // register B. Otherwise, register C is set to 0.
        fn gtrr(&mut self, reg_a: usize, reg_b: usize, reg_c: usize) {
            self.regs[reg_c] = if self.regs[reg_a] > self.regs[reg_b] { 1 } else { 0 };
        }

        // Equality testing

        // (equal immediate/register) sets register C to 1 if value A is equal to register B.
        // Otherwise, register C is set to 0.
        fn eqir(&mut self, value_a: usize, reg_b: usize, reg_c: usize) {
            self.regs[reg_c] = if value_a == self.regs[reg_b] { 1 } else { 0 };
        }

        // (equal register/immediate) sets register C to 1 if register A is equal to value B.
        // Otherwise, register C is set to 0.
        fn eqri(&mut self, reg_a: usize, value_b: usize, reg_c: usize) {
            self.regs[reg_c] = if self.regs[reg_a] == value_b { 1 } else { 0 };
        }

        // (equal register/register) sets register C to 1 if register A is equal to register B.
        // Otherwise, register C is set to 0.
        fn eqrr(&mut self, reg_a: usize, reg_b: usize, reg_c: usize) {
            self.regs[reg_c] = if self.regs[reg_a] == self.regs[reg_b] { 1 } else { 0 };
        }
    }
}
pub use self::device::Device;

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