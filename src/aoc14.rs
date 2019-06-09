use itertools::Itertools;

pub fn advent() {
    let input = 890691;
    let mut r = Recipies::new();
    println!("After {} recipies, next 10 scores: {}", input, r.ten_after(input));
    // step size manually tuned to be large enough
    println!("{} is first found at recipe: {}", input, r.find_score(&input.to_string(), 10000000));
}

#[derive(Debug)]
pub struct Recipies {
    scores: Vec<usize>,
    elf1: usize,
    elf2: usize,
}

impl Recipies {
    pub fn new() -> Recipies {
        let mut scores = Vec::new();
        scores.push(3);
        scores.push(7);
        Recipies { scores, elf1: 0, elf2: 1 }
    }

    pub fn advance(&mut self) {
        let sum = self.scores.get(self.elf1).unwrap() + self.scores.get(self.elf2).unwrap();
        assert!(sum < 100);
        if sum >= 10 {
            self.scores.push(sum / 10);
        }
        self.scores.push(sum % 10);
        self.elf1 = (self.elf1 + self.scores.get(self.elf1).unwrap() + 1) % self.scores.len();
        self.elf2 = (self.elf2 + self.scores.get(self.elf2).unwrap() + 1) % self.scores.len();
    }

    pub fn advance_til(&mut self, count: usize) {
        while self.scores.len() < count {
            self.advance();
        }
    }

    pub fn ten_after(&mut self, count: usize) -> String {
        self.advance_til(count + 10);
        self.scores[count..].iter().take(10).join("")
    }

    pub fn find_score(&mut self, search: &str, step: usize) -> usize {
        let mut i = 1;
        loop {
            self.advance_til(step * i);
            if let Some(idx) = self.scores.iter().join("").find(search) {
                return idx;
            }
            i += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn last_tens() {
        let mut r = Recipies::new();
        assert_eq!(r.ten_after(9), "5158916779");
        assert_eq!(r.ten_after(5), "0124515891");
        assert_eq!(r.ten_after(18), "9251071085");
        assert_eq!(r.ten_after(2018), "5941429882");
    }

    #[test]
    fn find_scores() {
        let mut r = Recipies::new();
        assert_eq!(r.find_score("51589", 10), 9);
        assert_eq!(r.find_score("01245", 10), 5);
        assert_eq!(r.find_score("92510", 20), 18);
        assert_eq!(r.find_score("59414", 2100), 2018);
    }
}