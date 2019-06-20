use std::fs;

pub fn advent() {
    let input = read_data();
    let mut cave: Cave = input.parse().expect("Invalid file");
    let (winner, rounds, _, outcome) = cave.combat();
    println!("Victor: {:?} - after {} rounds outcome: {}", winner, rounds, outcome);

    let (attack_power, winner, rounds, _, outcome) = increase_attack_power(&input);
    println!("With {} attack power, victor: {:?} - after {} rounds outcome: {}",
             attack_power, winner, rounds, outcome);
}

fn read_data() -> String {
    fs::read_to_string("data/day15.txt").expect("Cannot open")
}

fn increase_attack_power(input: &str) -> (u32, Race, u32, u32, u32){
    let mut attack_power = 3;
    loop {
        attack_power += 1;
        let mut cave: Cave = input.parse().expect("IMPOSSIBLE");
        let (_, elves, _) = cave.counts();
        cave.set_elf_attack_power(attack_power);
        let outcome = cave.combat();
        let (_, elves_left, _) = cave.counts();
        let (winner, rounds, health, score) = outcome;
        if winner == Race::Elf && elves == elves_left {
            return (attack_power, winner, rounds, health, score);
        }
    }
}

mod cave {
    use std::collections::{HashSet, HashMap};
    use std::fmt;
    use std::fmt::Write;
    use std::str::FromStr;
    use crate::euclid::{point, Point, vector, Vector};
    use super::*;

    static DIRECTIONS: [Vector; 4] =
        [vector(0, -1), vector(-1, 0), vector(1, 0), vector(0, 1)];

    pub struct Cave {
        squares: HashSet<Point>,
        units: HashMap<Point, Unit>,
        killed: HashSet<Point>,
        rounds: u32,
    }

    impl Cave {
        pub fn counts(&self) -> (usize, usize, usize) {
            // TODO use https://crates.io/crates/enum-map?
            let mut elves = 0;
            let mut goblins = 0;
            for u in self.units.values() {
                match u.race() {
                    Race::Elf => elves += 1,
                    Race::Goblin => goblins += 1,
                }
            };
            (self.units.len(), elves, goblins)
        }

        #[cfg(test)]
        fn set_health(&mut self, unit: Point, health: u32) {
            self.units.get_mut(&unit).expect("Unit must exist").set_health(health);
        }

        pub fn set_elf_attack_power(&mut self, attack_power: u32) {
            for elf in self.units.values_mut().filter(|u| *u.race() == Race::Elf) {
                elf.set_attack_power(attack_power);
            }
        }

        pub fn combat(&mut self) -> (Race, u32, u32, u32) {
            while self.continue_combat() {
                self.move_all_units();
            }
            let winner = {
            let (_, elves, goblins) = self.counts();
                if goblins == 0 {
                    Race::Elf
                } else if elves == 0 {
                    Race::Goblin
                } else {
                    panic!("Shouldn't happen");
                }
            };
            let health = self.units.values().map(|u| u.health()).sum();
            (winner, self.rounds, health, self.rounds * health)
        }

        fn continue_combat(&self) -> bool {
            let mut found_bitmap = 0;
            for u in self.units.values() {
                found_bitmap |= *u.race() as u8;
                if found_bitmap == ALL_RACES_BITMASK {
                    return true;
                }
            }
            false
        }

        fn move_all_units(&mut self) {
            // Not sure why to_owned() doesn't work here
            let mut units: Vec<Point> = self.units.keys().map(|p| *p).collect();
                // TODO same sort_by used in aoc13, pull it out?
            units.sort_by(|p1, p2|
                p1.y.cmp(&p2.y).then_with(|| p1.x.cmp(&p2.x)));

            self.killed.clear();
            for unit in units.iter() {
                if ! self.continue_combat() { return; }
                if self.killed.contains(unit) { continue; }
                let new_loc = self.move_unit(*unit);
                self.unit_attacks(new_loc.unwrap_or(*unit));
            }

            self.rounds += 1;
        }

        fn unit_attacks(&mut self, coord: Point) {
            let unit = self.units.get(&coord).expect(&format!("No unit found at {}", coord));
            let unit_race = *unit.race();
            let unit_attack_power = unit.attack_power();
            let target = DIRECTIONS.iter().map(|v| coord + v)
                .filter(|p| self.squares.contains(p))
                .flat_map(|p| {
                    let p = p.clone();
                    self.units.get(&p).into_iter().map(move |u| (p, u))
                })
                .filter(|(_, u)| u.race() == &unit_race.enemy())
                .min_by_key(|(_, u)| u.health())
                .map(|(p, _)| p);

            if let Some(target) = target {
                let enemy = self.units.get_mut(&target).expect("Should have found unit");
                if enemy.take_damage(unit_attack_power) {
                    self.units.remove(&target).expect("Should have killed unit");
                    self.killed.insert(target);
                }
            }

        }

        fn move_unit(&mut self, coord: Point) -> Option<Point> {
            let next_move = self.find_move(coord);
            if let Some(next_coord) = next_move {
                let unit = self.units.remove(&coord).expect("Must be present");
                assert!(self.units.insert(next_coord, unit).is_none());
            }
            next_move
        }

        fn find_move(&mut self, coord: Point) -> Option<Point> {
            let unit = self.units.get(&coord).expect(&format!("No unit found at {}", coord));
            let mut seen: HashSet<Point> = HashSet::new();
            seen.insert(coord);
            let mut queued: Vec<(Point, Point)> = Vec::new();

            // Inspect neighbors and populate queued
            // TODO pull DIRECTIONS.iter().map(|v| coord + v) into a helper?
            for neighbor in DIRECTIONS.iter().map(|v| coord + v) {
                seen.insert(neighbor);
                if self.squares.contains(&neighbor) {
                    if let Some(u) = self.units.get(&neighbor) {
                        if u.race() == &unit.race().enemy() {
                            // Already next to an enemy, no need to go any further
                            return None;
                        } else {
                            // Next to an ally, can't go that way
                            continue;
                        }
                    }

                    for next_neighbor in DIRECTIONS.iter().map(|v| neighbor + v) {
                        if !seen.contains(&next_neighbor) {
                            queued.push((next_neighbor, neighbor));
                        }
                    }
                }
            }

            // Expand search radius one layer at a time
            while !queued.is_empty() {
                let mut current_queue = queued;
                queued = Vec::new();
                current_queue.sort_by(|(p1, _), (p2, _)|
                    p1.y.cmp(&p2.y).then_with(|| p1.x.cmp(&p2.x)));
                for (dest, first_step) in current_queue {
                    if seen.contains(&dest) {
                        continue;
                    }
                    seen.insert(dest);
                    if self.squares.contains(&dest) {
                        if let Some(u) = self.units.get(&dest) {
                            if u.race() == &unit.race().enemy() {
                                // Found an enemy! This should be our closest target
                                return Some(first_step);
                            } else {
                                // Found an ally, can't go that way
                                continue;
                            }
                        }

                        for next_dest in DIRECTIONS.iter().map(|v| dest + v) {
                            if !seen.contains(&next_dest) {
                                queued.push((next_dest, first_step));
                            }
                        }
                    }
                }
            }

            None
        }
    }

    impl FromStr for Cave {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, String> {
            let mut squares =  HashSet::new();
            let mut units = HashMap::new();
            let mut coord = point(0, 0);

            for c in s.chars() {
                let c = match c {
                    'E' => { units.insert(coord, Unit::new(Race::Elf)); '.' },
                    'G' => { units.insert(coord, Unit::new(Race::Goblin)); '.' },
                    c => c,
                };
                match c {
                    '.' => { squares.insert(coord); },
                    '#' => {},
                    '\n' => { coord = point(-1, coord.y + 1); },
                    _ => { return Err(format!("Unexpected char {} at {}", c, coord)); },
                };
                coord = point(coord.x + 1, coord.y);
            }

            Ok(Cave { squares, units, killed: HashSet::new(), rounds: 0 })
        }
    }

    impl fmt::Display for Cave {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            if self.squares.is_empty() {
                return write!(f, "");
            }

            let mut out = String::new();
            // Two searches isn't ideal, but it's fine
            let max_x = self.squares.iter().map(|p| p.x).max().expect("isn't empty");
            let max_y = self.squares.iter().map(|p| p.y).max().expect("isn't empty");
            // exceed max by 1 to show outer boundary walls
            for y in 0..max_y+2 {
                for x in 0..max_x+2 {
                    let coord = point(x, y);
                    if self.squares.contains(&coord) {
                        match self.units.get(&coord) {
                            Some(u) => { write!(&mut out, "{}", u.race())?; },
                            None => out.push('.'),
                        }
                    } else {
                        out.push('#')
                    }
                }
                out.push('\n');
            }
            write!(f, "{}", out)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn parse() {
            let input = "#######\n#.G.E.#\n#E.G.E#\n#.G.E.#\n#######";
            let cave: Cave = input.parse().unwrap();
            assert_eq!(cave.counts(), (7, 4, 3));
            assert_eq!(cave.to_string().trim(), input.trim());
        }

        #[test]
        fn move_unit() {
            let input = "#######\n#E..G.#\n#...#.#\n#.G.#G#\n#######\n";
            let mut cave: Cave = input.parse().unwrap();
            assert_eq!(cave.move_unit(point(1, 1)), Some(point(2, 1)));
            assert_eq!(cave.to_string(), "#######\n#.E.G.#\n#...#.#\n#.G.#G#\n#######\n");
        }

        #[test]
        fn move_unit_empty() {
            let mut cave: Cave = "#####\n#...#\n#.E.#\n#...#\n#####\n".parse().unwrap();
            assert_eq!(cave.move_unit(point(2, 2)), None);
        }

        #[test]
        fn move_unit_allies() {
            let mut cave: Cave = "#####\n#...#\n#.E.#\n#..E#\n#####\n".parse().unwrap();
            assert_eq!(cave.move_unit(point(2, 2)), None);
            assert_eq!(cave.move_unit(point(3, 3)), None);
        }

        #[test]
        fn move_unit_enemy_adjacent() {
            let mut cave: Cave = "#####\n#...#\n#.EG#\n#...#\n#####\n".parse().unwrap();
            assert_eq!(cave.move_unit(point(2, 2)), None);
            assert_eq!(cave.move_unit(point(3, 2)), None);
        }

        #[test]
        fn move_unit_enemy_far() {
            let mut cave: Cave = "#####\n#E..#\n#...#\n#..G#\n#####\n".parse().unwrap();
            assert_eq!(cave.move_unit(point(1, 1)), Some(point(2, 1)));
            assert_eq!(cave.move_unit(point(3, 3)), Some(point(3, 2)));
        }

        #[test]
        fn move_unit_enemy_surrounded() {
            let mut cave: Cave = "#####\n#.G.#\n#GEG#\n#.G.#\n#####\n".parse().unwrap();
            assert_eq!(cave.move_unit(point(2, 2)), None);
            assert_eq!(cave.move_unit(point(1, 2)), None);
            assert_eq!(cave.move_unit(point(2, 1)), None);
        }
        #[test]
        fn move_unit_surrounded() {
            let mut cave: Cave = "####\n#E.#\n####\n#G.#\n####\n".parse().unwrap();
            assert_eq!(cave.move_unit(point(1, 1)), None);
            assert_eq!(cave.move_unit(point(1, 3)), None);
        }

        #[test]
        fn move_unit_down() {
            let input = "#####\n#G.G\n#...\n#...\n#..E";
            let mut cave: Cave = input.parse().unwrap();
            cave.move_unit(point(3, 1));
            assert_eq!(cave.to_string(), "#####\n#G..#\n#..G#\n#...#\n#..E#\n#####\n");
        }

        #[test]
        fn basic_movement() {
            let input = "####\n#E..\n#...\n#..G";
            let mut cave: Cave = input.parse().unwrap();
            cave.move_all_units();
            cave.move_all_units();
            let result = "#####\n#..E#\n#..G#\n#...#\n#####\n";
            assert_eq!(cave.to_string(), result);
            cave.move_all_units(); // no change
            assert_eq!(cave.to_string(), result);
        }

        #[test]
        fn movement() {
            let input = "#########\n#G..G..G#\n#.......#\n#.......#\n#G..E..G#\n#.......#\n#.......#\n#G..G..G#\n#########\n";
            let mut cave: Cave = input.parse().unwrap();
            cave.move_all_units();
            assert_eq!(cave.to_string(),
                       "#########\n#.G...G.#\n#...G...#\n#...E..G#\n#.G.....#\n#.......#\n#G..G..G#\n#.......#\n#########\n");
            cave.move_all_units();
            assert_eq!(cave.to_string(),
                       "#########\n#..G.G..#\n#...G...#\n#.G.E.G.#\n#.......#\n#G..G..G#\n#.......#\n#.......#\n#########\n");
            cave.move_all_units();
            let result = "#########\n#.......#\n#..GGG..#\n#..GEG..#\n#G..G...#\n#......G#\n#.......#\n#.......#\n#########\n";
            assert_eq!(cave.to_string(), result);
            cave.move_all_units(); // no change
            assert_eq!(cave.to_string(), result);
        }

        #[test]
        fn movement_idempotent() {
            // Prompted due to bug in movement logic that wasn't iterating over units in-order
            let input = "#########\n#G..G..G#\n#.......#\n#.......#\n#G..E..G#\n#.......#\n#.......#\n#G..G..G#\n#########\n";
            let mut cave1: Cave = input.parse().unwrap();
            cave1.move_all_units();
            for _ in 0..10 {
                let mut cave2: Cave = input.parse().unwrap();
                cave2.move_all_units();
                assert_eq!(cave1.to_string(), cave2.to_string());
            }
        }

        #[test]
        fn attack() {
            let mut cave: Cave = "EG".parse().unwrap();
            for _ in 0..67 {
                let (total, _, _) = cave.counts();
                assert_eq!(total, 2);
                cave.move_all_units();
            }
            assert_eq!(cave.counts(), (1, 1, 0));
        }

        #[test]
        fn attack_weakest() {
            let mut cave: Cave = "G.\nEG\nG.".parse().unwrap();
            cave.set_health(point(0, 0), 4);
            cave.set_health(point(1, 1), 2);
            cave.set_health(point(0, 2), 2);
            cave.move_all_units();
            assert_eq!(cave.to_string(), "G.#\nE.#\nG.#\n###\n");
        }

        #[test]
        fn combat() {
            let input = "#######\n#.G...#\n#...EG#\n#.#.#G#\n#..G#E#\n#.....#\n#######";
            let mut cave: Cave = input.parse().unwrap();
            cave.move_all_units();
            assert_eq!(cave.to_string(), "#######\n#..G..#\n#...EG#\n#.#G#G#\n#...#E#\n#.....#\n#######\n");
            cave.move_all_units();
            let round_2 = "#######\n#...G.#\n#..GEG#\n#.#.#G#\n#...#E#\n#.....#\n#######\n";
            assert_eq!(cave.to_string(), round_2);
            for _ in 3..23 {
                cave.move_all_units();
                assert_eq!(cave.to_string(), round_2); // looks the same as round 2
            }
            cave.move_all_units();
            let round_23 = "#######\n#...G.#\n#..G.G#\n#.#.#G#\n#...#E#\n#.....#\n#######\n";
            assert_eq!(cave.to_string(), round_23);
            cave.move_all_units();
            assert_eq!(cave.to_string(), "#######\n#..G..#\n#...G.#\n#.#G#G#\n#...#E#\n#.....#\n#######\n");
            cave.move_all_units();
            assert_eq!(cave.to_string(), "#######\n#.G...#\n#..G..#\n#.#.#G#\n#..G#E#\n#.....#\n#######\n");
            cave.move_all_units();
            assert_eq!(cave.to_string(), "#######\n#G....#\n#.G...#\n#.#.#G#\n#...#E#\n#..G..#\n#######\n");
            cave.move_all_units();
            assert_eq!(cave.to_string(), "#######\n#G....#\n#.G...#\n#.#.#G#\n#...#E#\n#...G.#\n#######\n");            cave.move_all_units();
            cave.move_all_units();
            let round_28 = "#######\n#G....#\n#.G...#\n#.#.#G#\n#...#E#\n#....G#\n#######\n";
            assert_eq!(cave.to_string(), round_28);
            for _ in 29..46 {
                cave.move_all_units();
                assert_eq!(cave.to_string(), round_28); // looks the same as round 28
            }
            cave.move_all_units();
            let round_47 = "#######\n#G....#\n#.G...#\n#.#.#G#\n#...#.#\n#....G#\n#######\n";
            assert_eq!(cave.to_string(), round_47);
            cave.move_all_units();
            assert_eq!(cave.to_string(), round_47); // No more enemies, so no motion
            assert_eq!(cave.counts(), (4, 0, 4));
        }

        parameterized_test!{ combats, (input, result, winner, rounds, health), {
            let mut cave: Cave = input.parse().unwrap();
            let outcome = cave.combat();
            assert_eq!(cave.to_string(), result);
            assert_eq!(outcome, (winner, rounds, health, rounds*health));
        }}
        combats!{
            full_example: ("#######\n#.G...#\n#...EG#\n#.#.#G#\n#..G#E#\n#.....#\n#######\n",
                "#######\n#G....#\n#.G...#\n#.#.#G#\n#...#.#\n#....G#\n#######\n",
                Race::Goblin, 47, 590),
            example_1: ("#######\n#G..#E#\n#E#E.E#\n#G.##.#\n#...#E#\n#...E.#\n#######\n",
                "#######\n#...#E#\n#E#...#\n#.E##.#\n#E..#E#\n#.....#\n#######\n",
                Race::Elf, 37, 982),
            example_2: ("#######\n#E..EG#\n#.#G.E#\n#E.##E#\n#G..#.#\n#..E#.#\n#######\n",
                "#######\n#.E.E.#\n#.#E..#\n#E.##.#\n#.E.#.#\n#...#.#\n#######\n",
                Race::Elf, 46, 859),
            example_3: ("#######\n#E.G#.#\n#.#G..#\n#G.#.G#\n#G..#.#\n#...E.#\n#######\n",
                "#######\n#G.G#.#\n#.#G..#\n#..#..#\n#...#G#\n#...G.#\n#######\n",
                Race::Goblin, 35, 793),
            example_4: ("#######\n#.E...#\n#.#..G#\n#.###.#\n#E#G#G#\n#...#G#\n#######\n",
                "#######\n#.....#\n#.#G..#\n#.###.#\n#.#.#.#\n#G.G#G#\n#######\n",
                Race::Goblin, 54, 536),
            example_5: ("#########\n#G......#\n#.E.#...#\n#..##..G#\n#...##..#\n#...#...#\n#.G...G.#\n#.....G.#\n#########\n",
                "#########\n#.G.....#\n#G.G#...#\n#.G##...#\n#...##..#\n#.G.#...#\n#.......#\n#.......#\n#########\n",
                Race::Goblin, 20, 937),
        }

        parameterized_test!{ combats_with_power, (input, attack_power, result, rounds, health), {
            let mut cave: Cave = input.parse().unwrap();
            cave.set_elf_attack_power(attack_power);
            let outcome = cave.combat();
            assert_eq!(cave.to_string(), result);
            assert_eq!(outcome, (Race::Elf, rounds, health, rounds*health));
        }}
        combats_with_power! {
            full_example: ("#######\n#.G...#\n#...EG#\n#.#.#G#\n#..G#E#\n#.....#\n#######\n", 15,
                "#######\n#..E..#\n#...E.#\n#.#.#.#\n#...#.#\n#.....#\n#######\n",
                29, 172),
            // This example isn't actually documented for part two
            example_1: ("#######\n#G..#E#\n#E#E.E#\n#G.##.#\n#...#E#\n#...E.#\n#######\n", 4,
                "#######\n#.E.#E#\n#E#...#\n#.E##.#\n#E..#E#\n#.....#\n#######\n",
                28, 1038),
            example_2: ("#######\n#E..EG#\n#.#G.E#\n#E.##E#\n#G..#.#\n#..E#.#\n#######\n", 4,
                "#######\n#.E.E.#\n#.#E..#\n#E.##E#\n#.E.#.#\n#...#.#\n#######\n",
                33, 948),
            example_3: ("#######\n#E.G#.#\n#.#G..#\n#G.#.G#\n#G..#.#\n#...E.#\n#######\n", 15,
                "#######\n#.E.#.#\n#.#E..#\n#..#..#\n#...#.#\n#.....#\n#######\n",
                37, 94),
            example_4: ("#######\n#.E...#\n#.#..G#\n#.###.#\n#E#G#G#\n#...#G#\n#######\n", 12,
                "#######\n#...E.#\n#.#..E#\n#.###.#\n#.#.#.#\n#...#.#\n#######\n",
                39, 166),
            example_5: ("#########\n#G......#\n#.E.#...#\n#..##..G#\n#...##..#\n#...#...#\n#.G...G.#\n#.....G.#\n#########\n", 34,
                "#########\n#.......#\n#.E.#...#\n#..##...#\n#...##..#\n#...#...#\n#.......#\n#.......#\n#########\n",
                30, 38),
        }
    }
}
pub use self::cave::Cave;

mod race {
    use std::fmt;

    pub static ALL_RACES_BITMASK: u8 = (Race::Elf as u8) | (Race::Goblin as u8);

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub enum Race {
        // https://stackoverflow.com/q/31358826/113632
        Elf = 1,
        Goblin = 2,
    }

    impl Race {
        pub fn enemy(&self) -> Race {
            match self {
                Race::Elf => Race::Goblin,
                Race::Goblin => Race::Elf,
            }
        }
    }

    impl fmt::Display for Race {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Race::Elf => write!(f, "E"),
                Race::Goblin => write!(f, "G"),
            }
        }
    }
}
pub use self::race::{ALL_RACES_BITMASK, Race};

mod unit {
    use super::*;

    pub struct Unit {
        race: Race,
        attack_power: u32,
        health: Option<u32>,
    }

    impl Unit {
        pub fn new(race: Race) -> Unit {
            Unit { race, attack_power: 3, health: Some(200) }
        }

        pub fn race(&self) -> &Race {
            &self.race
        }

        pub fn health(&self) -> u32 {
            self.health.expect("Do not ask for the health of a dead unit")
        }

        #[cfg(test)]
        pub fn set_health(&mut self, health: u32) {
            self.health = Some(health);
        }

        pub fn set_attack_power(&mut self, attack_power: u32) {
            self.attack_power = attack_power;
        }

        pub fn attack_power(&self) -> u32 {
            self.attack_power
        }

        pub fn take_damage(&mut self, damage: u32) -> bool {
            self.health = match self.health {
                Some(health) if health <= damage => None,
                Some(health) => Some(health - damage),
                None => panic!("Cannot damage dead unit"),
            };
            self.health.is_none() // is dead
        }
    }
}
pub use self::unit::Unit;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_file() { read_data().parse::<Cave>().unwrap(); }

    parameterized_test!{ attack_power, (input, attack_power, rounds, health), {
        let outcome = increase_attack_power(input);
        assert_eq!(outcome, (attack_power, Race::Elf, rounds, health, rounds*health));
    }}
    attack_power! {
        full_example: ("#######\n#.G...#\n#...EG#\n#.#.#G#\n#..G#E#\n#.....#\n#######\n", 15, 29, 172),
        // This example isn't actually documented for part two
        example_1: ("#######\n#G..#E#\n#E#E.E#\n#G.##.#\n#...#E#\n#...E.#\n#######\n", 4, 28, 1038),
        example_2: ("#######\n#E..EG#\n#.#G.E#\n#E.##E#\n#G..#.#\n#..E#.#\n#######\n", 4, 33, 948),
        example_3: ("#######\n#E.G#.#\n#.#G..#\n#G.#.G#\n#G..#.#\n#...E.#\n#######\n", 15, 37, 94),
        example_4: ("#######\n#.E...#\n#.#..G#\n#.###.#\n#E#G#G#\n#...#G#\n#######\n", 12, 39, 166),
        example_5: ("#########\n#G......#\n#.E.#...#\n#..##..G#\n#...##..#\n#...#...#\n#.G...G.#\n#.....G.#\n#########\n",
            34, 30, 38),
    }
}