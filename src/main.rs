use std::env;

mod aoc1;
mod aoc2;

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
        x => {
            eprintln!("Day {} hasn't happened yet.", x);
            ::std::process::exit(1);
        },

    }
}
