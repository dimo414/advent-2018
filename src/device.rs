use std::slice::Iter;
use self::Opcode::*;
use super::*;

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