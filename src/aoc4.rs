use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use chrono::Timelike;
use typed_arena::Arena;

pub fn advent() {
    let data = read_data();
    let naps = to_naps(&data);
    let guard = sleepiest_guard(&naps);
    let minute = guards_sleepiest_minute(&naps, guard);
    println!("Strategy 1:\tGuard: {} Minute: {} -\tGuard*Minute: {}", guard, minute, guard * minute);

    let (guard, minute) = sleepiest_guardminute(&naps);
    println!("Strategy 2:\tGuard: {} Minute: {} -\tGuard*Minute: {}", guard, minute, guard * minute);
}

// https://www.reddit.com/r/rust/comments/31syce/
fn max_entry<K, V>(map: &HashMap<K, V>) -> Option<(&K, &V)> where K: std::hash::Hash + Eq, V: Ord {
    //let mut max: Option<(&K, &V)> = None;
    //for (key, value) in map {
    //    if max.is_none() || max.unwrap().1 < value {
    //        max = Some((key, value))
    //    }
    //};
    //max
    map.iter().max_by(|&(_, v1), &(_, v2)| v1.cmp(&v2))
}

fn read_data() -> Vec<event::Event> {
    let reader = BufReader::new(File::open("data/day4.txt").expect("Cannot open"));

    let mut data: Vec<event::Event> =
        reader.lines().map(|l| l.unwrap().parse().unwrap()).collect();
    data.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    data
}

fn to_naps(events: &Vec<event::Event>) -> Vec<nap::Nap> {
    let mut naps = Vec::new();
    let mut cur_id = None;
    let mut asleep_at = None;

    for event in events {
        match event.event {
            event::EventType::StartsShift(id) => {
                if asleep_at.is_some() {
                    panic!("Invalid state {:?} while processing {:?}", asleep_at, event);
                }
                cur_id = Some(id);
            },
            event::EventType::FallsAsleep => {
                cur_id.expect("No guard on duty");
                asleep_at = Some(event.timestamp);
            },
            event::EventType::WakesUp => {
                naps.push(nap::Nap {
                    id: cur_id.expect("No guard on duty"),
                    start: asleep_at.expect("Guard isn't napping"),
                    end: event.timestamp });
                asleep_at = None;
            },
        };
    };
    naps
}

fn sleepiest_guard(naps: &Vec<nap::Nap>) -> u32 {
    let mut nap_minutes: HashMap<_, i64> = HashMap::new();
    for nap in naps {
        *nap_minutes.entry(nap.id).or_insert(0) += (nap.end - nap.start).num_minutes();
    };

    *max_entry(&nap_minutes).unwrap().0
}

fn guards_sleepiest_minute(naps: &Vec<nap::Nap>, guard: u32) -> u32 {
    let mut nap_minute: HashMap<u32, _> = HashMap::new();
    for nap in naps.iter().filter(|n| n.id == guard) {
        let mut minute = nap.start;
        while minute < nap.end {
            *nap_minute.entry(minute.minute()).or_insert(0) += 1;
            minute += chrono::Duration::minutes(1);
        }
    };

    *max_entry(&nap_minute).unwrap().0
}

fn sleepiest_guardminute(naps: &Vec<nap::Nap>) -> (u32, u32) {
    let inner = Arena::new();
    let mut minutes_map = HashMap::new();
    for nap in naps {
        let mut minute = nap.start;
        while minute < nap.end {
            let guard_map = minutes_map.entry(minute.minute())
                .or_insert_with(|| inner.alloc(HashMap::new()));
            *guard_map.entry(nap.id).or_insert(0) += 1;
            minute += chrono::Duration::minutes(1);
        }
    };

    let mut max_guard_minute_map = HashMap::new();
    for (minute, map) in minutes_map {
        let max = max_entry(map).expect("Map shouldn't be empty");
        max_guard_minute_map.insert(minute, (*max.1, *max.0));
    };

    let (minute, (_, guard)) = max_entry(&max_guard_minute_map).unwrap();
    (*guard, *minute)
}

#[cfg(test)]
mod tests {
    use chrono::naive::NaiveDate;
    use super::*;

    #[test]
    fn read_file() {
        assert!(read_data().len() > 0);
    }

    #[test]
    fn max_entries() {
        let mut map = HashMap::new();
        assert_eq!(max_entry(&map), None);

        map.insert("A".to_string(), 5);
        assert_eq!(max_entry(&map), Some((&"A".to_string(), &5)));

        map.insert("B".to_string(), 1);
        assert_eq!(max_entry(&map), Some((&"A".to_string(), &5)));

        map.insert("C".to_string(), 10);
        assert_eq!(max_entry(&map), Some((&"C".to_string(), &10)));
    }

    fn example_naps() -> Vec<nap::Nap> {
        let events: Vec<event::Event> = vec!(
            "[1518-11-01 00:00] Guard #10 begins shift",
            "[1518-11-01 00:05] falls asleep",
            "[1518-11-01 00:25] wakes up",
            "[1518-11-01 00:30] falls asleep",
            "[1518-11-01 00:55] wakes up",
            "[1518-11-01 23:58] Guard #99 begins shift",
            "[1518-11-02 00:40] falls asleep",
            "[1518-11-02 00:50] wakes up",
            "[1518-11-03 00:05] Guard #10 begins shift",
            "[1518-11-03 00:24] falls asleep",
            "[1518-11-03 00:29] wakes up",
            "[1518-11-04 00:02] Guard #99 begins shift",
            "[1518-11-04 00:36] falls asleep",
            "[1518-11-04 00:46] wakes up",
            "[1518-11-05 00:03] Guard #99 begins shift",
            "[1518-11-05 00:45] falls asleep",
            "[1518-11-05 00:55] wakes up").iter().map(|l| l.parse().unwrap()).collect();
        to_naps(&events)
    }

    #[test]
    fn validate_naps() {
        let naps = example_naps();

        let nov_1 = NaiveDate::from_ymd(1518, 11, 1);
        let nov_2 = NaiveDate::from_ymd(1518, 11, 2);
        let nov_3 = NaiveDate::from_ymd(1518, 11, 3);
        let nov_4 = NaiveDate::from_ymd(1518, 11, 4);
        let nov_5 = NaiveDate::from_ymd(1518, 11, 5);

        //date.and_hms(0, 0, 0)
        let expected = vec!(
            nap::Nap { id: 10, start: nov_1.and_hms(0, 5, 0), end: nov_1.and_hms(0, 25, 0) },
            nap::Nap { id: 10, start: nov_1.and_hms(0, 30, 0), end: nov_1.and_hms(0, 55, 0) },
            nap::Nap { id: 99, start: nov_2.and_hms(0, 40, 0), end: nov_2.and_hms(0, 50, 0) },
            nap::Nap { id: 10, start: nov_3.and_hms(0, 24, 0), end: nov_3.and_hms(0, 29, 0) },
            nap::Nap { id: 99, start: nov_4.and_hms(0, 36, 0), end: nov_4.and_hms(0, 46, 0) },
            nap::Nap { id: 99, start: nov_5.and_hms(0, 45, 0), end: nov_5.and_hms(0, 55, 0) });
        assert_eq!(naps, expected);
    }

    #[test]
    fn example_sleepiest() {
        let naps = example_naps();
        assert_eq!(sleepiest_guard(&naps), 10);
    }

    #[test]
    fn example_sleepiest_minute() {
        let naps = example_naps();
        assert_eq!(guards_sleepiest_minute(&naps, 10), 24);
        assert_eq!(guards_sleepiest_minute(&naps, 99), 45);
    }

    #[test]
    fn example_sleepiest_guardminute() {
        assert_eq!(sleepiest_guardminute(&example_naps()), (99, 45));
    }
}

mod nap {
    use chrono::naive::NaiveDateTime;

    #[derive(Debug, Eq, PartialEq)]
    pub struct Nap {
        pub id: u32,
        pub start: NaiveDateTime,
        pub end: NaiveDateTime,
    }
}

// Structure largely copied from aoc3::claim; maybe there's a way to reuse this boilerplate?
mod event {
    use chrono::naive::NaiveDateTime;
    use regex::{Captures, Regex};
    use std::error;
    use std::fmt;
    use std::num;
    use std::str::FromStr;

    #[derive(Debug, Eq, PartialEq)]
    pub enum EventError {
        Malformed(String),
        InvalidInt(num::ParseIntError),
    }

    impl fmt::Display for EventError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                EventError::Malformed(ref str) => write!(f, "Malformed {}!", str),
                EventError::InvalidInt(ref err) => err.fmt(f),
            }
        }
    }

    impl error::Error for EventError {
        fn source(&self) -> Option<&(dyn error::Error + 'static)> {
            match *self {
                EventError::Malformed(_) => None,
                EventError::InvalidInt(ref err) => Some(err),
            }
        }
    }

    impl From<num::ParseIntError> for EventError {
        fn from(err: num::ParseIntError) -> EventError {
            EventError::InvalidInt(err)
        }
    }

    #[derive(Debug, Eq, PartialEq)]
    pub enum EventType {
        StartsShift(u32),
        FallsAsleep,
        WakesUp,
    }

    #[derive(Debug, Eq, PartialEq)]
    pub struct Event {
        pub timestamp: NaiveDateTime,
        pub event: EventType,
    }

    type Result<T> = std::result::Result<T, EventError>;

    impl FromStr for Event {
        type Err = EventError;

        fn from_str(s: &str) -> Result<Self> {
            lazy_static! {
                static ref RE_SHIFT: Regex =
                    Regex::new(r"^\[([^\]]+)\] Guard #([0-9]+) begins shift$").unwrap();
                static ref RE_ASLEEP: Regex =
                    Regex::new(r"^\[([^\]]+)\] falls asleep$").unwrap();
                static ref RE_AWAKE: Regex =
                    Regex::new(r"^\[([^\]]+)\] wakes up$").unwrap();
            }

            let parse_timestamp = |caps: &Captures|
                NaiveDateTime::parse_from_str(
                        caps.get(1).expect("valid capture group").as_str(), "%Y-%m-%d %H:%M")
                    .map_err(|_| EventError::Malformed(s.into()));

            if let Some(caps) = RE_SHIFT.captures(s) {
                let timestamp = parse_timestamp(&caps)?;
                let id: u32 = caps.get(2).expect("valid capture group").as_str().parse()?;
                return Ok(Event { timestamp, event: EventType::StartsShift(id) });
            }
            if let Some(caps) = RE_ASLEEP.captures(s) {
                let timestamp = parse_timestamp(&caps)?;
                return Ok(Event { timestamp, event: EventType::FallsAsleep });
            }
            if let Some(caps) = RE_AWAKE.captures(s) {
                let timestamp = parse_timestamp(&caps)?;
                return Ok(Event { timestamp, event: EventType::WakesUp });
            }

            Err(EventError::Malformed("No Match".into()))
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use chrono::naive::NaiveDate;

        #[test]
        fn date_parse() {
            let timestamp = NaiveDateTime::parse_from_str("1518-11-01 00:00", "%Y-%m-%d %H:%M");
            assert_eq!(timestamp, Ok(NaiveDate::from_ymd(1518, 11, 1).and_hms(0, 0, 0)));
        }

        #[test]
        fn event_parse() {
            let (shift, asleep, awake) = (
                "[1518-11-01 00:00] Guard #10 begins shift",
                "[1518-11-01 00:05] falls asleep",
                "[1518-11-01 00:25] wakes up");
            let date = NaiveDate::from_ymd(1518, 11, 1);

            assert_eq!(shift.parse::<Event>(),
                       Ok(Event {
                           timestamp: date.and_hms(0, 0, 0), event: EventType::StartsShift(10) }));
            assert_eq!(asleep.parse::<Event>(),
                       Ok(Event {
                           timestamp: date.and_hms(0, 5, 0), event: EventType::FallsAsleep }));
            assert_eq!(awake.parse::<Event>(),
                       Ok(Event {
                           timestamp: date.and_hms(0, 25, 0), event: EventType::WakesUp }));
        }
    }
}