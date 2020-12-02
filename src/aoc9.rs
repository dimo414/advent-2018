use std::collections::HashMap;
use std::fmt::Write;

pub fn advent(args: &[String]) {
    println!("High Score: {}", simulate(432, 71019));

    // Manual microbenchmarking with --release suggests ~10000 is optimal for this many marbles
    let max_capacity: usize = args.get(0).unwrap_or(&"10000".to_string()).parse().unwrap();
    println!("High Score x100 with capacity {}: {}", max_capacity,
             simulate_generic(432, 71019 * 100, &mut MultiVecBacked::for_capacity(max_capacity) ));
}

fn simulate(players: u32, high_marble: u32) -> u32 {
    let mut scores: HashMap<_, _> = {1..players+1}.map(|n| (n, 0)).collect();
    let mut cur_pos = 0;
    let mut ring = Vec::with_capacity(high_marble as usize);
    ring.push(0);

    //print_ring(0, cur_pos, &ring);
    for marble in 1..high_marble+1 {
        let player = marble % players;
        if marble % 23 == 0 {
            let seven_counterclockwise_of_cur = (cur_pos + ring.len() - 8) % (ring.len()) + 1; // NEGATIVES???
            let removed_marble = ring.remove(seven_counterclockwise_of_cur);
            cur_pos = seven_counterclockwise_of_cur;
            let score = marble + removed_marble;
            scores.entry(player).and_modify(|s| *s += score).or_insert(score);
        } else {
            let two_clockwise_of_cur = (cur_pos + 1) % (ring.len()) + 1;
            cur_pos = two_clockwise_of_cur;
            if cur_pos == 0 {
                cur_pos = ring.len();
            }
            ring.insert(cur_pos, marble);
        }
        //print_ring(player, cur_pos, &ring);
    }

    *scores.values().max().expect("expected at least one player")
}

trait RingBuf {
    fn len(&self) -> usize;
    fn insert(&mut self, index: usize, value: u32);
    fn remove(&mut self, index: usize) -> u32;
    fn clear(&mut self);
}

fn simulate_generic<R>(players: u32, high_marble: u32, ring: &mut R) -> u32 where R: RingBuf {
    let mut scores: HashMap<_, _> = {1..players+1}.map(|n| (n, 0)).collect();
    let mut cur_pos = 0;
    ring.insert(0, 0);

    //print_ring(0, cur_pos, &ring);
    for marble in 1..high_marble+1 {
        let player = marble % players;
        if marble % 23 == 0 {
            let seven_counterclockwise_of_cur = (cur_pos + ring.len() - 8) % (ring.len()) + 1; // NEGATIVES???
            let removed_marble = ring.remove(seven_counterclockwise_of_cur);
            cur_pos = seven_counterclockwise_of_cur;
            let score = marble + removed_marble;
            scores.entry(player).and_modify(|s| *s += score).or_insert(score);
        } else {
            let two_clockwise_of_cur = (cur_pos + 1) % (ring.len()) + 1;
            cur_pos = two_clockwise_of_cur;
            if cur_pos == 0 {
                cur_pos = ring.len();
            }
            ring.insert(cur_pos, marble);
        }
        //print_ring(player, cur_pos, &ring);
    }

    *scores.values().max().expect("expected at least one player")
}

// Just a wrapper around a Vec, essentially the same behavior as simulate()
#[allow(dead_code)]
struct VecBacked { vec: Vec<u32> }
impl VecBacked {
    #[allow(dead_code)]
    fn new() -> Self { VecBacked { vec: Vec::with_capacity(10000) } }
}
impl RingBuf for VecBacked {
    fn len(&self) -> usize { self.vec.len() }
    fn insert(&mut self, index: usize, value: u32) { self.vec.insert(index, value); }
    fn remove(&mut self, index: usize) -> u32 { self.vec.remove(index) }
    fn clear(&mut self) { self.vec.clear(); }
}

// A sequence of vecs to support more efficient insertions; should be O(n) on the number of inner
// Vecs (since the size of each vec is capped it's effectively a constant). Requires tuning, but
// *much* more performant than the naive approach.
struct MultiVecBacked { vecs: Vec<Vec<u32>>, max_capacity: usize }
impl MultiVecBacked {
    #[allow(dead_code)]
    fn new() -> Self { MultiVecBacked::for_capacity(10000) }
    fn for_capacity(max_capacity: usize) -> Self {
        let vecs: Vec<Vec<u32>> = vec!(Vec::with_capacity(max_capacity));
        MultiVecBacked { vecs, max_capacity }
    }

    fn reshard(&mut self, index: usize) {
        // rough heuristic for when to reshard - would be better to keep a modification count and
        // use that, but this is fine for typical usages
        if index % (self.max_capacity / 10) != 0 {
            return
        }

        let mut i = 0;
        while i < self.vecs.len() {
            if self.vecs[i].is_empty() && self.vecs.len() > 1 {
                self.vecs.remove(i);
            }
            if self.vecs[i].len() > self.max_capacity {
                // vecs[i] may still be larger than max_capacity, but the extra churn of moving
                // more elements (potentially more than once, if vecs[i] is really large) isn't
                // worth it. With enough reshard()'s vecs[i] should be trimmed down to size.
                let truncate_at = self.vecs[i].len() - (self.max_capacity / 2);
                let mut next: Vec<_> = self.vecs[i].split_off(truncate_at);
                next.reserve(self.max_capacity);
                self.vecs.insert(i+1,next);
            }
            i += 1;
        }
    }
}
impl RingBuf for MultiVecBacked {
    fn len(&self) -> usize { self.vecs.iter().map(Vec::len).sum() }
    fn insert(&mut self, index: usize, value: u32) {
        self.reshard(index);
        let mut offset = 0;
        for vec in self.vecs.iter_mut() {
            if offset + vec.len() < index {
                offset += vec.len();
            } else {
                vec.insert(index - offset, value);
                return
            }
        }
        panic!("Didn't find a place to insert?")
    }
    fn remove(&mut self, index: usize) -> u32 {
        self.reshard(index);
        let mut offset = 0;
        for vec in self.vecs.iter_mut() {
            if offset + vec.len() <= index {
                offset += vec.len();
            } else {
                return vec.remove(index - offset);
            }
        }
        panic!("Didn't find a place to remove?")
    }

    fn clear(&mut self) {
        self.vecs.clear();
        self.vecs.push(Vec::with_capacity(self.max_capacity));
    }
}


#[allow(dead_code)]
fn print_ring(player: u32, cur_pos: usize, ring: &Vec<u32>) {
    let mut out = String::new();
    write!(out, "[{:2}]", player).unwrap();
    for i in 0..ring.len() {
        if i == cur_pos {
            write!(out, "({:2})", ring[i]).unwrap();
        } else {
            write!(out, " {:2} ", ring[i]).unwrap();
        }
    }

    println!("{}", out);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn examples() {
        assert_eq!(simulate(9, 25), 32);
        assert_eq!(simulate(10, 1618), 8317);
        assert_eq!(simulate(13, 7999), 146373);
        assert_eq!(simulate(17, 1104), 2764);
        assert_eq!(simulate(21, 6111), 54718);
        assert_eq!(simulate(30, 5807), 37305);
    }

    #[test]
    fn examples_vecbacked() {
        examples_generic(&mut VecBacked::new());
    }

    #[test]
    fn examples_multivecbacked() {
        examples_generic(&mut MultiVecBacked::new());
    }

    // would be preferable to pass in a Fn() -> RingBuf or similar, but I haven't figured out how
    fn examples_generic<T>(ring: &mut T) where T: RingBuf {
        ring.clear();
        assert_eq!(simulate_generic(9, 25, ring), 32);
        ring.clear();
        assert_eq!(simulate_generic(10, 1618, ring), 8317);
        ring.clear();
        assert_eq!(simulate_generic(13, 7999, ring), 146373);
        ring.clear();
        assert_eq!(simulate_generic(17, 1104, ring), 2764);
        ring.clear();
        assert_eq!(simulate_generic(21, 6111, ring), 54718);
        ring.clear();
        assert_eq!(simulate_generic(30, 5807, ring), 37305);
    }
}
