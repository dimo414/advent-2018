use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn advent() {
    let deps = read_data();
    println!("Task Order: {}", ordering(&deps));
    println!("Task Duration: {}", scheduling(&deps, 5, 60));
}

fn read_data() -> Vec<Dep> {
    let reader = BufReader::new(File::open("data/day7.txt").expect("Cannot open"));

    reader.lines().map(|l| l.unwrap().parse().unwrap()).collect()
}

fn topo(deps: &Vec<Dep>) -> Topology<char> {
    let mut t = Topology::new();
    for Dep(first, then) in deps {
        t.register(*first, *then);
    }
    t
}

fn ordering(deps: &Vec<Dep>) -> String {
    let mut t = topo(deps);
    let mut out = String::new();
    loop {
        match t.pop() {
            Some(c) => out.push(c),
            None => return out,
        }
    }
}

fn scheduling(deps: &Vec<Dep>, concurrency: u32, modifier: u32) -> u32 {
    let mut s = Scheduler::new(topo(deps), concurrency,
                               move |c: char| (c as u32) - ('A' as u32) + 1 + modifier);
    let mut out = 0;
    loop {
        if let None = s.tick() {
            return out;
        }
        out += 1;
    }
}

mod dep {
    use regex::{Match, Regex};
    use std::str::FromStr;

    #[derive(Debug, Eq, PartialEq)]
    pub struct Dep(pub char, pub char);

    impl FromStr for Dep {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, String> {
            lazy_static! {
                static ref RE: Regex =
                    Regex::new(r"^Step (.) must be finished before step (.) can begin.$").unwrap();
            }

            // https://stackoverflow.com/a/30811312/113632
            let char_from_group = |group: Option<Match>|
                group.expect("valid capture group").as_str().chars().next()
                    .expect("group matches one char");

            let caps = RE.captures(s).ok_or("no match")?;
            let first: char = char_from_group(caps.get(1));
            let then: char = char_from_group(caps.get(2));
            return Ok(Dep(first, then));
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn parse() {
            assert_eq!("Step C must be finished before step A can begin.".parse::<Dep>(),
                       Ok(Dep('C', 'A')));
        }
    }
}
pub use self::dep::Dep;

mod topology {
    use std::collections::{BTreeMap, HashSet};
    use std::collections::btree_map;
    use std::hash::Hash;

    // TODO remove the Clone constraint
    // See also https://stackoverflow.com/q/32401857/113632
    #[derive(Debug)]
    pub struct Topology<T: Clone + Hash + Ord> {
        deps: BTreeMap<T, HashSet<T>>,
    }

    impl <T: Clone + Hash + Ord> Topology<T> {
        pub fn new() -> Topology<T> {
            Topology { deps: BTreeMap::new() }
        }

        pub fn register(&mut self, first: T, then: T) {
            if first != then {
                self.deps.entry(then).or_insert_with(HashSet::new).insert(first.clone());
            }
            self.deps.entry(first).or_insert_with(HashSet::new);
        }

        pub fn is_empty(&self) -> bool {
            self.deps.is_empty()
        }

        pub fn peek_all(&self) -> impl Iterator<Item=&T> {
            self.deps.iter()
                .filter(|&(_, v)| v.is_empty())
                .map(|(k, _)| k)
        }

        pub fn pop(&mut self) -> Option<T> {
            if self.deps.is_empty() {
                return None;
            }

            let first_empty_key = self.peek_all()
                .next()
                .expect("Cycle detected");
            // Not sure how to avoid this .clone()
            Some(self.pop_exact(&first_empty_key.clone()))
        }

        pub fn pop_exact(&mut self, c: &T) -> T {
            assert!(self.peek_all().find(|k| *k == c).is_some());
            match self.deps.entry(c.clone()) {
                btree_map::Entry::Occupied(o) => {
                    let (k, _) = o.remove_entry();
                    for (_, v) in self.deps.iter_mut() {
                        v.remove(&k);
                    }
                    k
                },
                _ => panic!("impossible"),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn example_pt1() {
            let deps =
                vec!(('C', 'A'), ('C', 'F'), ('A', 'B'), ('A', 'D'), ('B', 'E'), ('D', 'E'),
                     ('F', 'E'));
            let mut t = Topology::new();
            for (first, then) in deps {
                t.register(first, then);
            }
            assert_eq!(t.pop(), Some('C'));
            assert_eq!(t.pop(), Some('A'));
            assert_eq!(t.pop(), Some('B'));
            assert_eq!(t.pop(), Some('D'));
            assert_eq!(t.pop(), Some('F'));
            assert_eq!(t.pop(), Some('E'));
            assert_eq!(t.pop(), None);
        }
    }
}
pub use self::topology::Topology;

mod scheduler {
    use super::Topology;
    use std::collections::HashMap;
    use std::fmt;
    use std::hash::Hash;

    pub struct Scheduler<T: Clone + Hash + Ord> {
        topo: Topology<T>,
        concurrency: u32,
        progress: HashMap<T, u32>,
        cost_fn: Box<Fn(T) -> u32>,
    }

    impl <T: Clone + Hash + Ord> Scheduler<T> {
        pub fn new<F>(topo: Topology<T>, concurrency: u32, cost_fn: F) -> Scheduler<T>
            where F: 'static + Fn(T) -> u32 {
            Scheduler { topo, concurrency, progress: HashMap::new(), cost_fn: Box::new(cost_fn) }
        }

        pub fn tick(&mut self) -> Option<Vec<T>> {
            if self.topo.is_empty() {
                return None;
            }

            // add new tasks
            let idle_workers = self.concurrency as usize - self.progress.len();
            let new_tasks: Vec<_> = self.topo.peek_all()
                .filter(|c| !self.progress.contains_key(*c))
                .take(idle_workers)
                .collect();
            for k in new_tasks {
                self.progress.insert(k.clone(), (self.cost_fn)(k.clone()));
            }
            assert!(self.progress.len() <= self.concurrency as usize);

            // pop finished tasks
            let mut done = Vec::new();
            for (k, v) in self.progress.iter_mut() {
                *v -= 1;
                if *v == 0 {
                    done.push(k.clone());
                }
            }
            done.iter().for_each(|k|
                {
                    self.progress.remove(k);
                    self.topo.pop_exact(k);
                });

            Some(done)
        }
    }

    impl <T: Clone + fmt::Debug + Hash + Ord> fmt::Debug for Scheduler<T> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Scheduler {{ topo: {:?}, concurrency: {:?}, progress: {:?} }}",
                   self.topo, self.concurrency, self.progress)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn example_pt2() {
            let deps =
                vec!(('C', 'A'), ('C', 'F'), ('A', 'B'), ('A', 'D'), ('B', 'E'), ('D', 'E'),
                     ('F', 'E'));
            // &|c| (c as u32) - ('A' as u32) + 1
            let mut t = Topology::new();
            for (first, then) in deps {
                t.register(first, then);
            }
            let mut s = Scheduler::new(t, 2, |c| (c as u32) - ('A' as u32) + 1);
            let empty: Option<Vec<char>> = Some(vec!());
            assert_eq!(s.tick(), empty);
            assert_eq!(s.tick(), empty);
            assert_eq!(s.tick(), Some(vec!('C')));
            assert_eq!(s.tick(), Some(vec!('A')));
            assert_eq!(s.tick(), empty);
            assert_eq!(s.tick(), Some(vec!('B')));
            assert_eq!(s.tick(), empty);
            assert_eq!(s.tick(), empty);
            assert_eq!(s.tick(), Some(vec!('F')));
            assert_eq!(s.tick(), Some(vec!('D')));
            assert_eq!(s.tick(), empty);
            assert_eq!(s.tick(), empty);
            assert_eq!(s.tick(), empty);
            assert_eq!(s.tick(), empty);
            assert_eq!(s.tick(), Some(vec!('E')));
            assert_eq!(s.tick(), None);
        }
    }
}
pub use self::scheduler::Scheduler;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_file() {
        assert!(read_data().len() > 0);
    }

    #[test]
    fn example_1() {
        let deps =
            vec!(Dep('C', 'A'), Dep('C', 'F'), Dep('A', 'B'), Dep('A', 'D'), Dep('B', 'E'),
                 Dep('D', 'E'), Dep('F', 'E'));
        assert_eq!(ordering(&deps), "CABDFE");
    }

    #[test]
    fn example_2() {
        let deps =
            vec!(Dep('C', 'A'), Dep('C', 'F'), Dep('A', 'B'), Dep('A', 'D'), Dep('B', 'E'),
                 Dep('D', 'E'), Dep('F', 'E'));
        assert_eq!(scheduling(&deps, 2, 0), 15);
    }
}



