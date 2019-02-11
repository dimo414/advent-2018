#[macro_use] extern crate itertools;
#[macro_use] extern crate lazy_static;
extern crate chrono;
extern crate regex;

use std::env;

mod aoc1;
mod aoc2;
mod aoc3;
mod aoc4;
mod aoc5;
mod aoc6;
mod aoc7;
mod aoc8;

fn main() {
    println!(); // split build output from runtime output
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} DAY_OF_ADVENT", args[0]);
        return;
    }
    let day: u32 = args[1].parse().unwrap();
    match day {
        1 => aoc1::advent(),
        2 => aoc2::advent(),
        3 => aoc3::advent(),
        4 => aoc4::advent(),
        5 => aoc5::advent(),
        6 => aoc6::advent(),
        7 => aoc7::advent(),
        8 => aoc8::advent(),
        x => {
            eprintln!("Day {} hasn't happened yet.", x);
            ::std::process::exit(1);
        },

    }
}
