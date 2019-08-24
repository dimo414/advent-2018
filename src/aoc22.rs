use crate::euclid::point;

pub fn advent() {
    let cave = Cave::new(5355, point(14, 796));
    println!("Risk to target: {}", cave.risk_to_target());
    let explorer = Explorer { cave };
    println!("Time to target: {}", explorer.time_to_target().expect("No path found"));

}

mod cave {
    use crate::euclid::{Point, point};
    use std::collections::HashMap;
    use std::cell::RefCell;

    pub enum Region {
        ROCKY,
        WET,
        NARROW,
    }

    pub struct Cave {
        depth: u32,
        target: Point,
        erosion: RefCell<HashMap<Point, u32>>,
    }

    impl Cave {
        pub fn new(depth: u32, target: Point) -> Cave {
            Cave { depth, target, erosion: RefCell::new(HashMap::new()) }
        }

        pub fn target(&self) -> Point { self.target }

        fn geo_index(&self, coord: Point) -> u32 {
            if coord == self.target { return 0; }
            match coord {
                Point { x: 0, y: 0 } => 0,
                Point { x, y: 0 } if x > 0 => x as u32 * 16807,
                Point { x: 0, y } if y > 0 => y as u32 * 48271,
                Point { x, y } if x > 0 && y > 0 =>
                    self.erosion_level(point(x-1, y)) * self.erosion_level(point(x, y-1)),
                _ => panic!("Unexpected negative point: {}", coord),
            }
        }

        pub fn erosion_level(&self, coord: Point) -> u32 {
            // Not sure if there's a reasonable way to use Entry::or_insert_with here
            // https://stackoverflow.com/q/40209552/113632
            let erosion = self.erosion.borrow();
            if let Some(level) = erosion.get(&coord) {
                return *level;
            }
            drop(erosion);
            let level = (self.geo_index(coord) + self.depth) % 20183;
            let mut erosion = self.erosion.borrow_mut();
            erosion.insert(coord, level);
            level
        }

        pub fn region_type(&self, coord: Point) -> Region {
            match self.erosion_level(coord) % 3 {
                0 => Region::ROCKY,
                1 => Region::WET,
                2 => Region::NARROW,
                _ => panic!(),
            }
        }

        pub fn risk(&self, coord: Point) -> u32 {
            self.erosion_level(coord) % 3
        }

        pub fn risk_to_target(&self) -> u32 {
            let target = self.target;
            let mut sum = 0;
            for x in {0..target.x+1} {
                for y in {0..target.y+1} {
                    sum += self.risk(point(x, y));
                }
            }
            sum
        }
    }
}
pub use self::cave::{Cave, Region};

mod explorer {
    use super::*;
    use crate::euclid::{Point, vector};
    use crate::pathfinding::{Graph, Edge};

    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
    pub enum Tool {
        TORCH,
        GEAR,
        NONE,
    }

    impl Tool {
        fn available_tools(region: Region) -> Vec<Tool> {
            match region {
                Region::ROCKY => vec!(Tool::TORCH, Tool::GEAR),
                Region::WET => vec!(Tool::GEAR, Tool::NONE),
                Region::NARROW => vec!(Tool::TORCH, Tool::NONE),
            }
        }

        fn is_usable(&self, region: Region) -> bool {
            match region {
                Region::ROCKY => self != &Tool::NONE,
                Region::WET => self != &Tool::TORCH,
                Region::NARROW => self != &Tool::GEAR,
            }
        }
    }

    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
    pub struct Status {
        region: Point,
        tool: Tool,
    }

    impl Status {
        pub fn new(region: Point, tool: Tool) -> Status {
            Status { region, tool }
        }
    }

    pub struct Explorer {
        pub cave: Cave,
    }

    impl Explorer {
        pub fn time_to_target(&self) -> Option<i32> {
            let path = self.dijkstras(
                &Status::new(point(0, 0), Tool::TORCH),
                &Status::new(self.cave.target(), Tool::TORCH));
            path.map(|path| path.iter().map(|e| e.weight()).sum())
        }
    }

    impl Graph for Explorer {
        type Node = Status;

        fn neighbors(&self, source: &Self::Node) -> Vec<Edge<Self::Node>> {
            let Status { region, tool } = source;

            let mut neighbors: Vec<_> = vec!(vector(0, 1), vector(1, 0), vector(0, -1), vector(-1, 0)).iter()
                .map(|v| region + v)
                .filter(|p| p.x >= 0 && p.y >= 0)
                .filter(|p| tool.is_usable(self.cave.region_type(*p)))
                .map(|p| Edge::new(1, source.clone(),
                                            Status::new(p, tool.clone())))
                .collect();

            // In every region we can also swap to the other usable tool for that region, which may
            // open up new paths.
            for alt_tool in Tool::available_tools(self.cave.region_type(*region)) {
                if &alt_tool != tool {
                    neighbors.push(
                        Edge::new(7, source.clone(), Status::new(region.clone(), alt_tool)));
                }
            }

            neighbors
        }
    }
}
pub use self::explorer::{Explorer,Status,Tool};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn erosion() {
        let cave = Cave::new(510, point(10, 10));
        assert_eq!(cave.erosion_level(point(0, 0)), 510);
        assert_eq!(cave.erosion_level(point(1, 0)), 17317);
        assert_eq!(cave.erosion_level(point(0, 1)), 8415);
        assert_eq!(cave.erosion_level(point(1, 1)), 1805);
        assert_eq!(cave.erosion_level(point(10, 10)), 510);
    }

    #[test]
    fn example() {
        let cave = Cave::new(510, point(10, 10));
        assert_eq!(cave.risk_to_target(), 114);
        let explorer = Explorer { cave };
        assert_eq!(explorer.time_to_target(), Some(45));
    }
}