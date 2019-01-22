use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn advent() {
    let data = read_data();
    println!("Final frequency:\t{}", frequency(&data));
    println!("First repeat frequency:\t{}", repeated_frequency(&data));
}

fn frequency(changes: &Vec<i32>) -> i32 {
    return changes.iter().sum();
}

fn repeated_frequency(changes: &Vec<i32>) -> i32 {
    let mut seen = HashSet::new();
    let mut rolling_sum = 0;
    seen.insert(rolling_sum);
    for change in changes.iter().cycle() {
        rolling_sum += change;
        //println!("Found: {:?}\tSum: {:?}\tSeen: {:?}", change, rolling_sum, seen);
        // if already inserted
        if ! seen.insert(rolling_sum) {
            return rolling_sum;
        }
    }
    panic!("IMPOSSIBLE");
}

fn parse(frequency: &str) -> i32 {
    return frequency.parse().unwrap();
}

fn read_data() -> Vec<i32> {
    let reader = BufReader::new(File::open("data/day1.txt").expect("Cannot open"));

    return reader.lines().map(|l| parse(&l.unwrap())).collect();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn examples_part1() {
        assert_eq!(frequency(&vec!(1, -2, 3, 1)), 3);
        assert_eq!(frequency(&vec!(1, 1, 1)), 3);
        assert_eq!(frequency(&vec!(1, 1, -2)), 0);
        assert_eq!(frequency(&vec!(-1, -2, -3)), -6);
    }

    #[test]
    fn examples_part2() {
        assert_eq!(repeated_frequency(&vec!(1, -1)), 0);
        assert_eq!(repeated_frequency(&vec!(3, 3, 4, -2, -4)), 10);
        assert_eq!(repeated_frequency(&vec!(-6, 3,8, 5, -6)), 5);
        assert_eq!(repeated_frequency(&vec!(7, 7, -2, -7, -4)), 14);
    }

    #[test]
    fn parsing() {
        assert_eq!(parse("+1"), 1);
        assert_eq!(parse("-2"), -2);
    }

    #[test]
    fn read_file() {
        assert!(read_data().len() > 0);
    }
}