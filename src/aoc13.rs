use std::fs;
use crate::euclid::{vector, Vector};

pub fn advent() {
    let mut tracks = read_data();
    loop {
        if let Some(coord) = tracks.advance() {
            println!("First collision at: {}", coord);
            break;
        }
    }
    let last_cart = tracks.advance_until_cleared();
    println!("Last remaining cart at {}", last_cart.expect("Should have one last cart"));
}

fn read_data() -> Tracks {
    fs::read_to_string("data/day13.txt").expect("Cannot open")
        .parse().expect("invalid file")
}

#[derive(Clone, Copy, Eq, Debug, PartialEq)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    fn as_vector(self) -> Vector {
        match self {
            Direction::North => vector(0, -1),
            Direction::South => vector(0, 1),
            Direction::East => vector(1, 0),
            Direction::West => vector(-1, 0),
        }
    }
}

mod track {
    use std::collections::HashMap;
    use std::fmt;
    use std::fmt::Write;
    use std::str::FromStr;
    use crate::euclid::{point, Point};
    use super::cart::Cart;
    use super::Direction;

    #[allow(non_camel_case_types)]
    #[derive(Clone, Copy, Debug)]
    pub enum TrackType {
        Vertical,
        Horizontal,
        WestSouth_EastNorth,
        WestNorth_EastSouth,
        Intersection,
    }

    #[derive(Debug)]
    pub struct Tracks {
        tracks: HashMap<Point, TrackType>,
        carts: HashMap<Point, Cart>,
    }

    impl Tracks {
        pub fn advance(&mut self) -> Option<Point> {
            let mut collision = None;
            // Ordered cart locations
            let mut coords: Vec<Point> = self.carts.keys().map(|&p| p).collect();
            coords.sort_by(|p1, p2|
                p1.y.cmp(&p2.y).then_with(|| p1.x.cmp(&p2.x)));
            let coords = coords;

            for coord in coords {
                // if not found, cart was already collided with
                if let Some(mut cart) = self.carts.remove(&coord) {
                    let track = self.tracks.get(&coord).expect("must be on track");
                    let new_coord = coord + cart.advance(track).as_vector();
                    // could also check that the cart is facing a valid direction if necessary
                    assert!(self.tracks.contains_key(&new_coord));

                    // Remove colliding carts
                    match self.carts.remove(&new_coord) {
                        Some(_) => {
                            // record the first collision
                            collision = collision.or(Some(new_coord));
                        },
                        None => {
                            assert!(self.carts.insert(new_coord, cart).is_none());
                        },
                    }
                }
            }
            collision
        }

        pub fn advance_until_cleared(&mut self) -> Option<Point> {
            while self.carts.len() > 1 {
                self.advance();
            }
            self.carts.keys().map(|&p| p).nth(0)
        }
    }

    impl FromStr for Tracks {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, String> {
            let mut tracks = HashMap::new();
            let mut carts = HashMap::new();
            let mut coord = point(0, 0);

            for c in s.chars() {
                let c = match c {
                    '<' => { carts.insert(coord, Cart::new(Direction::West)); '-'},
                    '>' => { carts.insert(coord, Cart::new(Direction::East)); '-'},
                    '^' => { carts.insert(coord, Cart::new(Direction::North)); '|'},
                    'v' => { carts.insert(coord, Cart::new(Direction::South)); '|'},
                    c => c,
                };
                match c {
                    '|' => { tracks.insert(coord, TrackType::Vertical); },
                    '-' => { tracks.insert(coord, TrackType::Horizontal); },
                    '/' => { tracks.insert(coord, TrackType::WestNorth_EastSouth); },
                    '\\' => { tracks.insert(coord, TrackType::WestSouth_EastNorth); },
                    '+' => { tracks.insert(coord, TrackType::Intersection); },
                    ' ' => {},
                    '\n' => { coord = point(-1, coord.y + 1); },
                    x => panic!("Unexpected char: {}", x),
                }
                coord = point(coord.x + 1, coord.y);
            }

            Ok(Tracks { tracks, carts })
        }
    }

    impl fmt::Display for Tracks {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            if self.tracks.is_empty() {
                return write!(f, "");
            }

            let mut out = String::new();
            // Two searches isn't ideal, but it's fine
            let max_x = self.tracks.keys().map(|p| p.x).max().expect("isn't empty");
            let max_y = self.tracks.keys().map(|p| p.y).max().expect("isn't empty");
            for y in 0..max_y+1 {
                for x in 0..max_x+1 {
                    let coord = point(x, y);
                    if let Some(c) = self.carts.get(&coord) {
                        assert!(self.tracks.contains_key(&coord));
                        write!(&mut out, "{}", c).expect("writing to string");
                        continue;
                    }

                    let c = match self.tracks.get(&coord) {
                        Some(t) => {
                            match t {
                                TrackType::Vertical => '|',
                                TrackType::Horizontal => '-',
                                TrackType::WestSouth_EastNorth => '\\',
                                TrackType::WestNorth_EastSouth => '/',
                                TrackType::Intersection => '+',
                            }
                        },
                        None => ' ',
                    };
                    out.push(c);
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
            let input = "/----\\\n|    |\n|    |\n\\----/\n";
            assert_eq!(input, input.parse::<Tracks>().unwrap().to_string());

            let input = "/--<--\\   \n|     |   \n|  /--+--\\\n|  ^  v  |\n".to_string()
                + "\\--+--/  |\n   |     |\n   \\-->--/\n";
            assert_eq!(input, input.parse::<Tracks>().unwrap().to_string());
        }

        #[test]
        fn advance() {
            let input = "->---<-\n";
            let mut tracks: Tracks = input.parse().unwrap();
            for _ in 0..4 {
                if let Some(coord) = tracks.advance() {
                    assert_eq!(coord, point(3, 0));
                    return;
                }
            }
            assert!(false, "{:?} should have crashed", tracks);
        }

        // https://www.reddit.com/r/adventofcode/comments/a5t7vx/x/ebp1hlz/
        #[test]
        fn back_to_back() {
            let input = "/>>-\\\n";
            let mut tracks: Tracks = input.parse().unwrap();
            assert_eq!(tracks.advance(), Some(point(2, 0)));
            assert_eq!(tracks.to_string(), "/---\\\n");
        }

        #[test]
        fn cart_ordering() {
            let input = "/<<-\\\n\\---/\n";
            let mut tracks: Tracks = input.parse().unwrap();
            assert_eq!(tracks.advance(), None);
            assert_eq!(tracks.to_string(), "<<--\\\n\\---/\n");
            assert_eq!(tracks.advance(), None);
            assert_eq!(tracks.to_string(), "<---\\\nv---/\n");
            assert_eq!(tracks.advance(), Some(point(0, 1)));
            assert_eq!(tracks.to_string(), "/---\\\n\\---/\n");
        }
    }
}
pub use self::track::Tracks;

mod cart {
    use std::fmt;
    use super::Direction;
    use super::track::*;

    #[derive(Clone, Copy, Debug)]
    enum TurnState {
        Left,
        Straight,
        Right,
    }

    #[derive(Debug)]
    pub struct Cart {
        direction: Direction,
        next_turn: TurnState,
    }

    impl Cart {
        pub fn new(direction: Direction) -> Cart {
            Cart { direction, next_turn: TurnState::Left }
        }


        pub fn advance(&mut self, track: &TrackType) -> Direction {
            self.direction = match (track, &self.direction) {
                (&TrackType::Vertical, &Direction::North) => Direction::North,
                (&TrackType::Vertical, &Direction::South) => Direction::South,

                (&TrackType::Horizontal, &Direction::East) => Direction::East,
                (&TrackType::Horizontal, &Direction::West) => Direction::West,

                (&TrackType::WestSouth_EastNorth, &Direction::North) => Direction::West,
                (&TrackType::WestSouth_EastNorth, &Direction::South) => Direction::East,
                (&TrackType::WestSouth_EastNorth, &Direction::East) => Direction::South,
                (&TrackType::WestSouth_EastNorth, &Direction::West) => Direction::North,

                (&TrackType::WestNorth_EastSouth, &Direction::North) => Direction::East,
                (&TrackType::WestNorth_EastSouth, &Direction::South) => Direction::West,
                (&TrackType::WestNorth_EastSouth, &Direction::East) => Direction::North,
                (&TrackType::WestNorth_EastSouth, &Direction::West) => Direction::South,

                (&TrackType::Intersection, _) => self.intersection(),

                x => panic!("Illegal arrangement {:?}", x),
            };

            self.direction
        }

        fn intersection(&mut self) -> Direction {
            let new_dir = Cart::turn(self.direction, self.next_turn);
            self.next_turn = match self.next_turn {
                TurnState::Left => TurnState::Straight,
                TurnState::Straight => TurnState::Right,
                TurnState::Right => TurnState::Left,
            };
            new_dir
        }

        fn turn(direction: Direction, turn: TurnState) -> Direction {
            match (direction, turn) {
                (Direction::North, TurnState::Left) => Direction::West,
                (Direction::West, TurnState::Left) => Direction::South,
                (Direction::South, TurnState::Left) => Direction::East,
                (Direction::East, TurnState::Left) => Direction::North,

                (Direction::North, TurnState::Right) => Direction::East,
                (Direction::West, TurnState::Right) => Direction::North,
                (Direction::South, TurnState::Right) => Direction::West,
                (Direction::East, TurnState::Right) => Direction::South,

                (d, TurnState::Straight) => d,
            }
        }
    }

    impl fmt::Display for Cart {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let c = match self.direction {
                Direction::North => '^',
                Direction::South => 'v',
                Direction::East => '>',
                Direction::West => '<',
            };
            write!(f, "{}", c)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn advances() {
            let mut cart = Cart::new(Direction::East);
            assert_eq!(cart.advance(&TrackType::Horizontal), Direction::East);
            assert_eq!(cart.advance(&TrackType::WestNorth_EastSouth), Direction::North);
            assert_eq!(cart.advance(&TrackType::WestSouth_EastNorth), Direction::West);
            assert_eq!(cart.advance(&TrackType::Intersection), Direction::South);
            assert_eq!(cart.advance(&TrackType::Intersection), Direction::South);
            assert_eq!(cart.advance(&TrackType::Intersection), Direction::West);

            let mut cart = Cart::new(Direction::South);
            assert_eq!(cart.advance(&TrackType::Vertical), Direction::South);
            assert_eq!(cart.advance(&TrackType::WestNorth_EastSouth), Direction::West);
            assert_eq!(cart.advance(&TrackType::WestSouth_EastNorth), Direction::North);
            assert_eq!(cart.advance(&TrackType::Intersection), Direction::West);
            assert_eq!(cart.advance(&TrackType::Intersection), Direction::West);
            assert_eq!(cart.advance(&TrackType::Intersection), Direction::North);
        }

        // TODO test that invalid cases panic
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::euclid::point;

    #[test]
    fn read_file() {
        read_data();
    }

    #[test]
    fn first_crash() {
        let input = "/->-\\        \n|   |  /----\\\n| /-+--+-\\  |\n| | |  | v  |\n\\-+-/  \\-+--/\n  \\------/   \n";
        let mut tracks: Tracks = input.parse().unwrap();

        for _ in 0..13 {
            assert_eq!(tracks.advance(), None);
        }
        assert_eq!(tracks.advance(), Some(point(7, 3)));
    }

    #[test]
    fn last_cart() {
        let input = "/>-<\\  \n|   |  \n| /<+-\\\n| | | v\n\\>+</ |\n  |   ^\n  \\<->/\n";
        let mut tracks: Tracks = input.parse().unwrap();
        assert_eq!(input, tracks.to_string());
        assert_eq!(tracks.advance_until_cleared(), Some(point(6, 4)));
    }
}