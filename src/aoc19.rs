use std::fs;
use crate::device::{Device,Program};

pub fn advent() {
    // See the data file for more notes and comments
    let program_text = read_file();
    let program: Program = program_text.parse().expect("invalid program");
    let mut device = Device::new([0, 0, 0, 0, 0, 0]);
    device.run_program(&program);
    let final_registers = device.get_registers();
    println!("Starting with A=0, F is set to {} and A is now: {}", final_registers[5], final_registers[0]);

    // Here we assume that the program is a naive sum-of-factors program, with the value to compute
    // set at the end of the program, and the final instruction is a GOTO to the loop-init logic.
    // Therefore we strip that last instruction to run the program just long enough to get the
    // target value, then compute the sum-of-factors more efficiently.
    //
    // Note there are no safety checks that the program being run is actually of this form :)
    let mut device = Device::new([1, 0, 0, 0, 0, 0]);
    let program: Program = remove_last_line(&program_text).parse().expect("invalid program");
    device.run_program(&program);
    let f = device.get_registers()[5];
    println!("Starting with A=1, F is set to: {}", f);
    println!("Sum of factors of {}: {}", f, sum_of_factors(f));
}

fn read_file() -> String {
    fs::read_to_string("data/day19.txt").expect("Cannot open")
}

fn remove_last_line(s: &str) -> &str {
    match s.trim_end().rfind('\n') {
        Some(n) => &s[..n+1],
        None => s,
    }
}

fn sum_of_factors(n: usize) -> usize {
    let sqrt_n = (n as f64).sqrt() as usize;
    let mut sum = 0;
    for i in {1..sqrt_n} {
        if n % i == 0 {
            sum += i + n/i;
        }
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remove_lines() {
        let sample = "AAAA\nBBBB\nCCCC\nDDDD\n";
        assert_eq!(remove_last_line(sample), "AAAA\nBBBB\nCCCC\n");
        assert_eq!(remove_last_line(&sample[..sample.len()-1]), "AAAA\nBBBB\nCCCC\n");
    }

    #[test]
    fn sum_factors() {
        assert_eq!(sum_of_factors(950), 1860);
    }
}