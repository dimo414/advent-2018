use std::fs;
use crate::device::{Device, Program, Debugger};
use std::collections::HashSet;
use std::time::Instant;

pub fn advent() {
    // See the data file for more notes and comments
    let program: Program = read_file();
    println!("First F: {}", find_first_f_value(&program));

    let start = Instant::now();
    println!("Computed last F: {}", compute_last_f_value());
    let stop = Instant::now() - start;
    println!("  Took: {:?}", stop);

    if cfg!(debug_assertions) {
        println!("WARNING: without --release this will take a while...");
    }
    let start = Instant::now();
    println!("Actual last F:   {}", find_last_f_value(&program));
    let stop = Instant::now() - start;
    println!("  Took: {:?}", stop);
}

fn read_file() -> Program {
    fs::read_to_string("data/day21.txt").expect("Cannot open")
        .parse().expect("invalid program")
}

struct FindFirstFValue;
impl Debugger for FindFirstFValue {
    fn on_exec(&mut self, ip: usize) -> bool {
        // stop once we hit instruction 28
        ip != 28
    }
}

fn find_first_f_value(program: &Program) -> usize {
    let mut finder = FindFirstFValue;
    let mut device = Device::new([0, 0, 0, 0, 0, 0]);
    device.debug_program(program, &mut finder);
    device.get_registers()[5] // F holds the target value
}

struct FindLastFValue {
    last_f_value: Option<usize>,
    seen: HashSet<usize>,
}
impl FindLastFValue {
    fn new() -> FindLastFValue {
        FindLastFValue { last_f_value: None, seen: HashSet::new() }
    }

    fn last_value(&self) -> Option<usize> {
        self.last_f_value
    }
}
impl Debugger for FindLastFValue {
    fn on_exec_registers(&mut self, ip: usize, pre: [usize; 6], _post: [usize; 6]) -> bool {
        if ip == 28 {
            // stop once we hit a value we've seen before
            if ! self.seen.insert(pre[5]) { return false; }
            self.last_f_value = Some(pre[5]);

            if cfg!(debug_assertions) && self.seen.len() % 1000 == 0 {
                println!("  Seen {} values...", self.seen.len());
            }
        }
        true
    }
}

fn find_last_f_value(program: &Program) -> usize {
    let mut finder = FindLastFValue::new();
    let mut device = Device::new([0, 0, 0, 0, 0, 0]);
    device.debug_program(program, &mut finder);
    finder.last_value().expect("Should have computed at least one F value")
}

fn compute_last_f_value() -> usize {
    let mut seen = HashSet::new();
    let mut last_f = None;

    let mut b;
    let mut d;
    let mut e;
    let mut f = 0;

    loop { // GOTO 6
        d = f | 0x10000;
        f = 7586220;
        loop { // GOTO 8
            f = ((((d & 0xFF) + f) & 0xFFFFFF) * 65899) & 0xFFFFFF;
            if 0x100 > d { break; }
            // the innermost loop is a linear-time division operation
            let next_d = d / 0x100;
            if cfg!(debug_assertions) {
                // https://users.rust-lang.org/t/conditional-compilation-for-debug-release/1098/8
                // https://doc.rust-lang.org/reference/conditional-compilation.html#the-cfg-macro
                b = 0;
                loop { // GOTO 18
                    e = (b + 1) * 0x100;
                    if e > d { break; }
                    b += 1;
                }
                assert_eq!(next_d, b); // d = b;
            }
            d = next_d;
        }

        if ! seen.insert(f) { // if seen before
            return last_f.expect("Must be set");
        } else {
            last_f = Some(f);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // These are basically change-detector sanity checks, but better than nothing
    #[test]
    fn validate_first_f() {
        let program: Program = read_file(); // also confirms program is syntactically valid
        assert_eq!(find_first_f_value(&program), 11050031);
    }

    #[test]
    fn validate_last_f() {
        assert_eq!(compute_last_f_value(), 11341721);
    }
}