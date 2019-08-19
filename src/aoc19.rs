use std::fs;
use crate::device::{Device, Program, Debugger};

pub fn advent() {
    let program: Program = read_file();
    let mut device = Device::new([0, 0, 0, 0, 0, 0]);
    device.run_program(&program);
    let final_registers = device.get_registers();
    println!("Starting with A=0, F is initialized to {} and A is now: {}",
             final_registers[5], final_registers[0]);

    // The program is a naive sum-of-factors program that first computes a large number and then
    // finds all factors of that number. This initial setup happens at the end of the program,
    // before GOTO-ing to the start of the loop logic. Therefore we can stop at that instruction in
    // order to see the computed value without running the rest of the program.
    //
    // See the data file for more notes and comments

    let mut device = Device::new([1, 0, 0, 0, 0, 0]);
    device.debug_program(&program, &mut StopAtInstruction(35));
    let f = device.get_registers()[5];
    println!("Starting with A=1, F is initialized to: {}", f);
    println!("Sum of factors of {}: {}", f, sum_of_factors(f));
}

fn read_file() -> Program {
    fs::read_to_string("data/day19.txt").expect("Cannot open").parse().expect("invalid program")
}

struct StopAtInstruction(usize);
impl Debugger for StopAtInstruction {
    fn on_exec(&mut self, ip: usize) -> bool { self.0 != ip }
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
    fn read_program() {
        read_file();
    }

    #[test]
    fn sum_factors() {
        assert_eq!(sum_of_factors(950), 1860);
    }
}