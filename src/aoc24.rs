use std::fs;

pub fn advent() {
    let mut battle = read_data("data/day24.txt");
    let units_left = battle.fight();
    println!("Units remaining: {:?}", units_left.0 + units_left.1);
    let (boost, remaining) = increase_attack_damage("data/day24.txt");
    println!("Boost of {} lets {} immune system units survive", boost, remaining);
}

fn read_data(path: &str) -> Battle {
    fs::read_to_string(path).expect("Cannot open").parse().expect("Cannot parse")
}

fn increase_attack_damage(path: &str) -> (u32, u32) {
    let mut boost_max = 1;
    loop {
        let mut battle = read_data(path);
        battle.set_boost(boost_max);
        if battle.fight().1 == 0 { // infection is dead
            break;
        }
        boost_max *= 2;
    }
    let mut boost_range = (boost_max / 2 + 1, boost_max);
    loop {
        let mid_boost = {
            let (min, max) = boost_range;
            (max-min)/2 + min
        };
        let mut battle = read_data(path);
        battle.set_boost(mid_boost);
        let remaining_units = battle.fight();
        if remaining_units.1 == 0 { // infection is dead
            if mid_boost == boost_range.0 {
                return (mid_boost, remaining_units.0);
            }
            boost_range = (boost_range.0, mid_boost);
        } else {
            boost_range = (mid_boost+1, boost_range.1);
        }
    }
}

mod battle {
    use super::*;
    use std::str::FromStr;
    use crate::error::ParseError;
    use std::collections::HashMap;
    use itertools::Itertools;
    use std::cmp::Reverse;

    pub struct Battle {
        immune: Vec<Group>,
        infection: Vec<Group>,
    }

    impl Battle {
        // Each group, ordered by Effective Power (desc) selects a target group
        //   1. Would deal the most damage
        //   2. Most effective power
        //   3. Highest initiative
        // Skips if cannot deal damage to any group
        // Defending groups can only be targeted by one attacker per round
        fn target_selection(&self) -> HashMap<Army, HashMap<usize, usize>> {
            let mut targets = HashMap::new();
            targets.insert(Army::IMMUNE, Battle::target_selection_for(&self.immune, &self.infection));
            targets.insert(Army::INFECTION, Battle::target_selection_for(&self.infection, &self.immune));

            targets
        }

        fn target_selection_for(groups: &[Group], enemies: &[Group]) -> HashMap<usize, usize> {
            let mut team_targets = HashMap::new();
            let mut remaining_enemies: Vec<&Group> = enemies.iter().collect();

            for group in groups.iter().sorted_by_key(|g| Reverse(g.effective_power())) {
                let best_target = remaining_enemies.iter()
                    .map(|e| (e, e.expected_damage_from(group)))
                    .filter(|(_, d)| d > &0)
                    .sorted_by_key(|(e, d)| Reverse((*d, e.effective_power(), e.initiative())))
                    .map(|(e, _)| e.id())
                    .next();

                if let Some(best_id) = best_target {
                    let pos = remaining_enemies.iter()
                        .position(|e| e.id() == best_id).expect("Must be present");
                    remaining_enemies.swap_remove(pos);
                    team_targets.insert(group.id(), best_id);
                }
            }

            team_targets
        }

        // Each group, ordered by initiative, attacks their target
        // Deals Effective Power to target group, unless
        //   Immune: no damage
        //   Weak: 2x damage
        fn attack(&mut self, targets: HashMap<Army, HashMap<usize, usize>>) {
            let groups = {
                self.immune.iter().chain(self.infection.iter())
                    .sorted_by_key(|g| Reverse(g.initiative()))
                    .map(|g| (g.army(), g.id()))
                    .collect::<Vec<_>>()
            };
            for (army, id) in groups {
                if let Some(target) = targets[&army].get(&id) {
                    match army {
                        Army::IMMUNE => Battle::single_attack(&self.immune, id, &mut self.infection, *target),
                        Army::INFECTION => Battle::single_attack(&self.infection, id, &mut self.immune, *target),
                    }
                }
            }
        }

        fn single_attack(attackers: &Vec<Group>, attacker: usize, enemies: &mut Vec<Group>, target: usize) {
            if let Some(attacker) = attackers.iter().find(|g| g.id() == attacker) {
                if let Some(i) = enemies.iter().position(|g| g.id() == target) {
                    enemies[i].take_damage_from(attacker);
                    if enemies[i].units() == 0 {
                        enemies.swap_remove(i);
                    }
                }
            }
        }

        fn remaining_units(&self) -> (u32, u32) {
            (self.immune.iter().map(Group::units).sum(),
             self.infection.iter().map(Group::units).sum())
        }

        pub fn fight(&mut self) -> (u32, u32) {
            let mut units = self.remaining_units();
            loop {
                let old_units = units;
                let targets = self.target_selection();
                self.attack(targets);
                units = self.remaining_units();
                if units == old_units || self.immune.is_empty() || self.infection.is_empty() {
                    break;
                }
            }
            units
        }

        pub fn set_boost(&mut self, boost: u32) {
            for group in self.immune.iter_mut() {
                group.set_boost(boost);
            }
        }
    }

    impl FromStr for Battle {
        type Err = ParseError;

        fn from_str(s: &str) -> Result<Self, ParseError> {
            let mut combatants = HashMap::new();
            let mut cur_army: Option<Army> = None;

            for line in s.lines().filter(|l| !l.is_empty()) {
                if line.ends_with(':') {
                    cur_army = Some(line[..line.len()-1].parse().expect("Invalid"));
                } else {
                    let cur_army = cur_army.expect("Must first specify an army");
                    let groups = combatants.entry(cur_army).or_insert(Vec::new());
                    let mut group: Group = line.parse()?;
                    group.set_id(groups.len()+1);
                    group.set_army(cur_army);
                    groups.push(group);
                }
            }

            Ok(Battle {
                immune: combatants.remove(&Army::IMMUNE).expect("Must have immune system"),
                infection: combatants.remove(&Army::INFECTION).expect("Must have infection")
            })
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn parse_data() {
            read_data("data/day24.txt");
        }

        #[test]
        fn selection() {
            let battle = read_data("data/day24-example.txt");
            let selections = battle.target_selection();

            let mut expected_immune = HashMap::new();
            expected_immune.insert(1, 2);
            expected_immune.insert(2, 1);
            let mut expected_infect = HashMap::new();
            expected_infect.insert(1, 1);
            expected_infect.insert(2, 2);
            let mut expected = HashMap::new();
            expected.insert(Army::IMMUNE, expected_immune);
            expected.insert(Army::INFECTION, expected_infect);

            assert_eq!(selections, expected);
        }

        #[test]
        fn example() {
            let mut battle = read_data("data/day24-example.txt");
            assert_eq!(battle.fight(), (0, 5216));
        }

        #[test]
        fn example_boost() {
            let mut battle = read_data("data/day24-example.txt");
            battle.set_boost(1570);
            assert_eq!(battle.fight(), (51, 0));
        }
    }
}
pub use self::battle::Battle;

mod army {
    use std::str::FromStr;
    use crate::error::ParseError;

    #[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
    pub enum Army {
        IMMUNE,
        INFECTION,
    }

    impl FromStr for Army {
        type Err = ParseError;

        fn from_str(s: &str) -> Result<Self, ParseError> {
            match s {
                "Immune System" => Ok(Army::IMMUNE),
                "Infection" => Ok(Army::INFECTION),
                _ => Err(ParseError::Malformed(format!("Unsupported: {}", s))),
            }
        }
    }
}
pub use self::army::Army;

mod group {
    use std::fmt;
    use std::str::FromStr;
    use std::fmt::Write;
    use regex::{Regex, Captures};
    use crate::error::ParseError;
    use crate::aoc24::Army;
    use itertools::Itertools;

    #[derive(Eq, PartialEq)]
    pub struct Group {
        id: Option<usize>,
        army: Option<Army>,
        units: u32,
        hp: u32,
        immune: Vec<String>,
        weak: Vec<String>,
        attack: u32,
        boost: u32,
        attack_type: String,
        initiative: u32,
    }

    impl Group {
        fn new(units: u32, hp: u32, immune: Vec<String>, weak: Vec<String>, attack: u32, attack_type: &str, initiative: u32) -> Group {
            Group { id: None, army: None, units, hp, immune, weak, attack, boost: 0, attack_type: attack_type.to_string(), initiative }
        }

        pub fn id(&self) -> usize { self.id.expect("Must set ID first") }

        pub fn set_id(&mut self, id: usize) {
            assert!(self.id.is_none());
            self.id = Some(id);
        }

        pub fn army(&self) -> Army { self.army.expect("Must set army first") }

        pub fn set_army(&mut self, army: Army) {
            assert!(self.army.is_none());
            self.army = Some(army);
        }

        pub fn set_boost(&mut self, boost: u32) {
            self.boost = boost;
        }

        pub fn units(&self) -> u32 { self.units }
        pub fn initiative(&self) -> u32 { self.initiative }

        pub fn effective_power(&self) -> u32 { self.units * (self.attack + self.boost) }

        pub fn expected_damage_from(&self, other: &Group) -> u32 {
            assert!(self.units > 0, "No point computing damage for a dead group");
            if self.immune.contains(&other.attack_type) {
                return 0;
            }
            let mut damage = other.effective_power();
            if self.weak.contains(&other.attack_type) {
                damage *= 2;
            }
            damage
        }

        // Target group loses floor(Effective Power / Target Units) units
        pub fn take_damage_from(&mut self, other: &Group) {
            self.take_damage(self.expected_damage_from(other));
        }

        fn take_damage(&mut self, damage: u32) {
            let deaths = std::cmp::min(damage / self.hp, self.units);
            self.units -= deaths;
        }
    }

    impl fmt::Display for Group {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{:?}({})", self.army(), self.id())
        }
    }

    impl fmt::Debug for Group {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let mut out = String::new();
            write!(&mut out, "{{{}: ", self)?;
            write!(&mut out, "Init:{}\tEP:{}\tUnits:{}\tHP:{}\tAttack:{}\tAT:{}",
                   self.initiative, self.effective_power(), self.units, self.hp, self.attack, self.attack_type)?;
            if ! self.immune.is_empty() {
                write!(&mut out, "\tImmune:{}", self.immune.iter().join(","))?;
            }
            if ! self.weak.is_empty() {
                write!(&mut out, "\tWeak:{}", self.weak.iter().join(","))?;
            }
            write!(f, "{}}}", out)
        }
    }

    impl FromStr for Group {
        type Err = ParseError;

        fn from_str(s: &str) -> Result<Self, ParseError> {

            lazy_static! {
                static ref GROUP_RE: Regex = Regex::new(
                    r"^(\d+) units each with (\d+) hit points (\([^)]+\) )?with an attack that does (\d+) (.+) damage at initiative (\d+)$").unwrap();
                static ref IMMUNE_RE: Regex = Regex::new("immune to ([^;)]+)").unwrap();
                static ref WEAK_RE: Regex = Regex::new("weak to ([^;)]+)").unwrap();
            }

            let caps: Captures = regex_captures!(GROUP_RE, s)?;
            let units: u32 = capture_group!(caps, 1).parse()?;
            let hp: u32 = capture_group!(caps, 2).parse()?;
            let modifiers = caps.get(3).map(|c| c.as_str());
            let attack: u32 = capture_group!(caps, 4).parse()?;
            let attack_type = capture_group!(caps, 5).to_string();
            let initiative: u32 = capture_group!(caps, 6).parse()?;

            let (immune, weak) = match modifiers {
                Some(modifiers) => {
                    let immune = match IMMUNE_RE.captures(modifiers) {
                        Some(caps) => capture_group!(caps, 1).split(", ").map(|s| s.to_string()).collect(),
                        None => vec!(),
                    };
                    let weak = match WEAK_RE.captures(modifiers) {
                        Some(caps) => capture_group!(caps, 1).split(", ").map(|s| s.to_string()).collect(),
                        None => vec!(),
                    };
                    // Minimal error handling; should actually check that each ;-delimited block
                    // is a known modifier, but this is fine for now
                    if immune.is_empty() && weak.is_empty() {
                        return Err(ParseError::Malformed(format!("Unknown modifiers: {}", modifiers)));
                    }
                    (immune, weak)
                },
                None => (vec!(), vec!()),
            };

            Ok(Group::new(units, hp, immune, weak, attack, &attack_type, initiative))
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        parameterized_test!{ parse, (s, expected), {
            let group = s.parse::<Group>();
            assert_eq!(group, Ok(expected));
        }}
        parse!{
            example: ("18 units each with 729 hit points (weak to fire; immune to cold, \
                slashing) with an attack that does 8 radiation damage at initiative 10",
                Group::new(18, 729, vec!("cold".to_string(), "slashing".to_string()),
                    vec!("fire".to_string()), 8, "radiation", 10)),
            example_no_mods: ("18 units each with 729 hit points with an attack \
                that does 8 radiation damage at initiative 10",
                Group::new(18, 729, vec!(), vec!(), 8, "radiation", 10)),
        }
    }
}
pub use self::group::Group;