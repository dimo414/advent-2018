use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn advent() {
    let data = read_data();
    println!("Checksum:\t{}", checksum(&data));
    println!("Common Chars:\t{}", find_one_char_diff(&data));

}

fn checksum(ids: &Vec<String>) -> i32 {
    let sums = ids.iter()
        .map(|id| dupe_chars(&id))
        .fold((0,0), |s, i| (s.0 + i.0, s.1 + i.1));
    return sums.0 * sums.1;
}

fn dupe_chars(id: &str) -> (i32, i32) {
    let mut chars = HashMap::new();
    for c in id.chars() {
        *chars.entry(c).or_insert(0) += 1;
    }
    let mut result = (0, 0);
    for v in chars.values() {
        if *v == 2 {
            result.0 = 1;
        }
        if *v == 3 {
            result.1 = 1;
        }
    }
    return result;
}

fn strip_noncommon(id1: &str, id2: &str) -> String {
    return id1.chars().zip(id2.chars())
        .filter(|t| t.0 == t.1)
        .map(|t| t.0)
        .collect::<String>()
}

// O(n^2) is a bit sad, but it's more than fast enough for the given input
fn find_one_char_diff(ids: &Vec<String>) -> String {
    for id1 in ids.iter() {
        for id2 in ids.iter() {
            assert!(id1.len() == id2.len());

            let common = strip_noncommon(id1, id2);
            if common.len() == id1.len() - 1 {
                return common;
            }
        }
    }
    panic!("No IDs found with one character difference");
}

fn read_data() -> Vec<String> {
    let reader = BufReader::new(File::open("data/day2.txt").expect("Cannot open"));

    return reader.lines().map(|l| l.unwrap()).collect();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn copy(vec: Vec<&str>) -> Vec<String> {
        return vec.iter().map(|&s| s.to_string()).collect();
    }

    #[test]
    fn examples_part1() {
        let ids: Vec<String> =
            copy(vec!("abcdef", "bababc", "abbcde", "abcccd", "aabcdd", "abcdee", "ababab"));

        assert_eq!(dupe_chars(&ids[0]), (0, 0));
        assert_eq!(dupe_chars(&ids[1]), (1, 1));
        assert_eq!(dupe_chars(&ids[2]), (1, 0));
        assert_eq!(dupe_chars(&ids[3]), (0, 1));
        assert_eq!(dupe_chars(&ids[4]), (1, 0));
        assert_eq!(dupe_chars(&ids[5]), (1, 0));
        assert_eq!(dupe_chars(&ids[6]), (0, 1));

        assert_eq!(checksum(&ids), 12);
    }

    #[test]
    fn strip_chars() {
        assert_eq!(strip_noncommon("abcde", "axcye"), "ace");
        assert_eq!(strip_noncommon("fghij", "fguij"), "fgij");
    }

    #[test]
    fn find_one_distant() {
        let ids: Vec<String> =
            copy(vec!("abcde", "fghij", "klmno", "pqrst", "fguij", "axcye", "wvxyz"));
        assert_eq!(find_one_char_diff(&ids), "fgij");
    }

    #[test]
    fn read_file() {
        assert!(read_data().len() > 0);
    }
}