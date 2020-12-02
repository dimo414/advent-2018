use regex::Regex;
use std::slice::Iter;
use std::str::FromStr;
use self::Opcode::*;
use std::collections::BTreeMap;

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
        OPCODES.iter()
    }
}

impl FromStr for Opcode {
    type Err = String;

    fn from_str(s: &str) -> Result<Opcode, String> {
       // let s: &str = &s.to_uppercase();
        match &s.to_uppercase() as &str {
            "ADDR" => Ok(Opcode::ADDR),
            "ADDI" => Ok(Opcode::ADDI),
            "MULR" => Ok(Opcode::MULR),
            "MULI" => Ok(Opcode::MULI),
            "BANR" => Ok(Opcode::BANR),
            "BANI" => Ok(Opcode::BANI),
            "BORR" => Ok(Opcode::BORR),
            "BORI" => Ok(Opcode::BORI),
            "SETR" => Ok(Opcode::SETR),
            "SETI" => Ok(Opcode::SETI),
            "GTIR" => Ok(Opcode::GTIR),
            "GTRI" => Ok(Opcode::GTRI),
            "GTRR" => Ok(Opcode::GTRR),
            "EQIR" => Ok(Opcode::EQIR),
            "EQRI" => Ok(Opcode::EQRI),
            "EQRR" => Ok(Opcode::EQRR),
            _ => Err(format!("Unexpected: {}", s)),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Instruction {
    opcode: Opcode,
    input_a: usize,
    input_b: usize,
    output: usize,
}

impl Instruction {
    pub fn new(opcode: Opcode, input_a: usize, input_b: usize, output: usize) -> Instruction {
        Instruction { opcode, input_a, input_b, output }
    }
}

impl FromStr for Instruction {
    type Err = String;

    fn from_str(s: &str) -> Result<Instruction, String> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^([a-z]+)[^a-z\d]*(\d+)[^\d]*(\d+)[^\d]*(\d+)$").unwrap();
        }

        let caps = regex_captures!(RE, s)?;
        let code: Opcode = capture_group!(caps, 1).parse()?;
        let a: usize = capture_group!(caps, 2).parse().map_err(|_| "NOPE".to_string())?;
        let b: usize = capture_group!(caps, 3).parse().map_err(|_| "NOPE".to_string())?;
        let c: usize = capture_group!(caps, 4).parse().map_err(|_| "NOPE".to_string())?;
        Ok(Instruction::new(code, a, b, c))
    }
}

#[cfg(test)]
mod instruction_tests {
    use super::*;

    parameterized_test::create! { parse_instruction, (s, expected), {
        assert_eq!(s.parse::<Instruction>(), Ok(expected));
    }}
    parse_instruction! {
        seti: ("seti 5 0 1", Instruction::new(Opcode::SETI, 5, 0, 1)),
        addr: ("addr 1 2 3", Instruction::new(Opcode::ADDR, 1, 2, 3)),
    }
}

#[derive(Debug)]
pub struct Program {
    ip_register: Option<usize>,
    instructions: Vec<Instruction>,
}

impl Program {
    pub fn create(instructions: Vec<Instruction>) -> Program {
        Program::new(None, instructions)
    }

    #[allow(dead_code)]
    pub fn create_with_ip(register: usize, instructions: Vec<Instruction>) -> Program {
        Program::new(Some(register), instructions)
    }

    fn new(ip_register: Option<usize>, instructions: Vec<Instruction>) -> Program {
        Program { ip_register, instructions }
    }
}

// A more lenient parser than the spec described in AOC19, this permits blank lines and comments
// denoted by //. Comments can be on their own line or at the end of a "real" line.
impl FromStr for Program {
    type Err = String;

    fn from_str(s: &str) -> Result<Program, String> {
        lazy_static! {
            static ref IP_RE: Regex = Regex::new(r"^#ip[^\d]*(\d+)$").unwrap();
            static ref COMMENT_RE: Regex = Regex::new(r"\s*//.*$").unwrap();
        }

        let lines: Vec<_> = s.lines()
            .map(|l| COMMENT_RE.replace(l, ""))
            .filter(|l| !l.is_empty())
            .collect();

        if lines.is_empty() {
            return Err("Invalid program; no content".into());
        }

        let mut ip_register: Option<usize> = None;
        let mut remaining_lines = &lines[..];

        if let Ok(caps) = regex_captures!(IP_RE, &lines[0]) {
            remaining_lines = &lines[1..]; // skip line 0
            ip_register = Some(capture_group!(caps, 1).parse().expect("Invalid register"));
        }

        let mut instructions = Vec::new();
        for line in remaining_lines {
            let instruction = line.parse()?;
            instructions.push(instruction);
        }
        Ok(Program::new(ip_register, instructions))
    }
}

#[cfg(test)]
mod program_tests {
    use super::*;

    #[test]
    fn parse() {
        let sample = "\n// comment\n#ip 1\naddi 1 16 1 // EOL\nseti 1 8 2\nseti 1 5 4\n";
        let program: Program = sample.parse().unwrap();
        assert_eq!(program.ip_register, Some(1));
        assert_eq!(program.instructions.len(), 3);

    }
}

pub trait Debugger {
    fn on_exec(&mut self, ip: usize) -> bool { let _=ip; unimplemented!(); }

    fn on_exec_registers(&mut self, ip: usize, pre: [usize; 6], post: [usize; 6]) -> bool {
        let _=pre; let _=post;
        self.on_exec(ip)
    }

    fn on_halt(&mut self, ip: usize) { let _=ip; }
}

struct NoopDebugger;
impl Debugger for NoopDebugger {
    fn on_exec(&mut self, _: usize) -> bool { true }
}

pub struct ExecCounter {
    counts: BTreeMap<usize, usize>,
}

#[allow(dead_code)]
impl ExecCounter {
    pub fn new() -> ExecCounter {
        ExecCounter { counts: BTreeMap::new() }
    }

    pub fn counts(&self) -> &BTreeMap<usize, usize> {
        &self.counts
    }

    pub fn total(&self) -> usize {
        self.counts.values().sum()
    }
}

impl Debugger for ExecCounter {
    fn on_exec(&mut self, ip: usize) -> bool {
        let count = self.counts.entry(ip).or_insert(0);
        *count += 1;
        true
    }

    fn on_halt(&mut self, ip: usize) {
        self.counts.entry(ip).or_insert(0);
    }
}

pub struct ExecLogger {
    steps: usize,
    halt_after: usize,
    should_log: Box<dyn Fn(usize, usize) -> bool>,
}

#[allow(dead_code)]
impl ExecLogger {
    pub fn new(halt_after: usize, should_log: Box<dyn Fn(usize, usize) -> bool>) -> ExecLogger {
        ExecLogger{ steps: 0, halt_after, should_log }
    }

    pub fn halt_after(halt_after: usize) -> ExecLogger {
        ExecLogger::new(halt_after, Box::new(|_, _| true))
    }
}

impl Debugger for ExecLogger {
    fn on_exec_registers(&mut self, ip: usize, pre: [usize; 6], post: [usize; 6]) -> bool {
        self.steps += 1;
        if (self.should_log)(ip, self.steps) {
            println!("{}:{}\t{:?}\t->\t{:?}", ip, self.steps, pre, post);
        }
        self.steps <= self.halt_after
    }

    fn on_halt(&mut self, ip: usize) {
        println!("HALT: {}", ip);
    }
}

pub struct Device {
    regs: [usize; 6],
    ip: usize,
}

impl Device {
    pub fn new(regs: [usize; 6]) -> Device {
        Device { regs, ip: 0 }
    }

    pub fn get_registers(&self) -> [usize; 6] {
        self.regs
    }

    pub fn run_program(&mut self, program: &Program) {
        self.debug_program(program, &mut NoopDebugger);
    }

    pub fn debug_program(&mut self, program: &Program, debugger: &mut impl Debugger) {
        loop {
            if let Some(ip_reg) = program.ip_register {
                self.regs[ip_reg] = self.ip;
            }
            match program.instructions.get(self.ip) {
                Some(instruction) => {
                    let prior_state = self.get_registers();
                    self.exec(instruction);
                    let post_state = self.get_registers();
                    let proceed = debugger.on_exec_registers(self.ip, prior_state, post_state);
                    if !proceed { break; }
                },
                None => {
                    debugger.on_halt(self.ip);
                    break;
                },
            }
            if let Some(ip_reg) = program.ip_register {
                self.ip = self.regs[ip_reg];
            }
            self.ip += 1; // will be written back to the register in the next loop
        }
    }

    fn exec(&mut self, i: &Instruction) {
        match i.opcode {
            Opcode::ADDR => self.addr(i.input_a, i.input_b, i.output),
            Opcode::ADDI => self.addi(i.input_a, i.input_b, i.output),
            Opcode::MULR => self.mulr(i.input_a, i.input_b, i.output),
            Opcode::MULI => self.muli(i.input_a, i.input_b, i.output),
            Opcode::BANR => self.banr(i.input_a, i.input_b, i.output),
            Opcode::BANI => self.bani(i.input_a, i.input_b, i.output),
            Opcode::BORR => self.borr(i.input_a, i.input_b, i.output),
            Opcode::BORI => self.bori(i.input_a, i.input_b, i.output),
            Opcode::SETR => self.setr(i.input_a, i.input_b, i.output),
            Opcode::SETI => self.seti(i.input_a, i.input_b, i.output),
            Opcode::GTIR => self.gtir(i.input_a, i.input_b, i.output),
            Opcode::GTRI => self.gtri(i.input_a, i.input_b, i.output),
            Opcode::GTRR => self.gtrr(i.input_a, i.input_b, i.output),
            Opcode::EQIR => self.eqir(i.input_a, i.input_b, i.output),
            Opcode::EQRI => self.eqri(i.input_a, i.input_b, i.output),
            Opcode::EQRR => self.eqrr(i.input_a, i.input_b, i.output),
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
