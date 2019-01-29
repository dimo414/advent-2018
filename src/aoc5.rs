use std::collections::HashSet;
use std::fs;

pub fn advent() {
    let original = read_data().trim().to_string();
    let trimmed = trim_all_pairs(&original);
    println!("Count: {}", trimmed.len());

    let units: HashSet<char> = original.chars().map(|c| c.to_ascii_lowercase()).collect();
    let min = units.iter()
        .map(|c| trim_all_pairs(&remove_pair(&original, *c)).len())
        .min().expect("Should be a min");
    println!("Count with unit removed: {}", min);
}

fn read_data() -> String {
    fs::read_to_string("data/day5.txt").expect("Cannot open")
}

fn is_pair(a: char, b: char) -> bool {
    a != b && a.eq_ignore_ascii_case(&b)
}

fn trim_pairs(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    let mut last_char = chars.next();

    loop {
        let cur_char = chars.next();
        match cur_char {
            Some(c) => {
                if let Some(lc) = last_char {
                    if !is_pair(lc, c) {
                        out.push(lc);
                        last_char = cur_char;
                    } else {
                        last_char = chars.next(); // skip cur_char
                    }
                }
            },
            None => {
                if let Some(c) = last_char {
                    out.push(c);
                }
                return out;
            },
        }
    }
}

fn trim_all_pairs(s: &str) -> String {
    let mut last = s.to_string();
    loop {
        let stripped = trim_pairs(&last);
        if last == stripped {
            return last;
        }
        last = stripped;
    }
}

fn remove_pair(s: &str, p: char) -> String {
    s.chars().filter(|c| !c.eq_ignore_ascii_case(&p)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_pairs() {
        assert!(is_pair('a', 'A'));
        assert!(!is_pair('A', 'A'));
        assert!(!is_pair('b', 'A'));
    }

    #[test]
    fn remove_pairs() {
        let s = "dabAcCaCBAcCcaDA";
        assert_eq!(remove_pair(&s, 'd'), "abAcCaCBAcCcaA");
        assert_eq!(remove_pair(&s, 'D'), "abAcCaCBAcCcaA");
        assert_eq!(remove_pair(&s, 'a'), "dbcCCBcCcD");
        assert_eq!(remove_pair(&s, 'A'), "dbcCCBcCcD");
    }

    #[test]
    fn basic_examples() {
        assert_eq!(trim_pairs("aA"), "");
        assert_eq!(trim_pairs("abBA"), "aA");
        assert_eq!(trim_pairs("abAB"), "abAB");
        assert_eq!(trim_pairs("aabAAB"), "aabAAB");

        assert_eq!(trim_all_pairs("aA"), "");
        assert_eq!(trim_all_pairs("abBA"), "");
        assert_eq!(trim_all_pairs("abAB"), "abAB");
        assert_eq!(trim_all_pairs("aabAAB"), "aabAAB");
    }

    #[test]
    fn larger_examples() {
        // our algorithm does multiple 'reactions' in a single pass, so the examples aren't the same
        //assert_eq!(trim_pairs("dabAcCaCBAcCcaDA"), "dabAaCBAcCcaDA");
        //assert_eq!(trim_pairs("dabAaCBAcCcaDA"), "dabCBAcCcaDA");
        //assert_eq!(trim_pairs("dabCBAcCcaDA"), "dabCBAcaDA");
        assert_eq!(trim_pairs("dabAcCaCBAcCcaDA"), "dabAaCBAcaDA");
        assert_eq!(trim_pairs("dabAaCBAcaDA"), "dabCBAcaDA");

        assert_eq!(trim_all_pairs("dabAcCaCBAcCcaDA"), "dabCBAcaDA");
    }

    #[test]
    fn remove_examples() {
        let s = "dabAcCaCBAcCcaDA";
        assert_eq!(trim_all_pairs(&remove_pair(s, 'a')), "dbCBcD");
        assert_eq!(trim_all_pairs(&remove_pair(s, 'b')), "daCAcaDA");
        assert_eq!(trim_all_pairs(&remove_pair(s, 'c')), "daDA");
        assert_eq!(trim_all_pairs(&remove_pair(s, 'd')), "abCBAc");
    }
}