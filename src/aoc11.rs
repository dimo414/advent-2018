pub fn advent() {
    let grid_serial = 6303;
    let power_grid = populate_power_grid(grid_serial);
    let region_grid = populate_power_regions(&power_grid, 3);
    println!("Best 3x3 square for serial {}: {:?}", grid_serial, find_high_power_region(&region_grid));
    let max_region = 20; // 300 still takes too time
    println!("Best NxN square for serial {}: {:?}", grid_serial, find_high_power_region_size(&power_grid, max_region));
}

fn power_level(x: isize, y: isize, grid_serial: u32) -> i32 {
    let rack_id = x + 10;
    let power_level = rack_id * y + grid_serial as isize;
    let power_level = rack_id * power_level;
    power_level as i32 / 100 % 10 - 5
}

fn populate_power_grid(grid_serial: u32) -> Grid {
    let mut grid = Grid::new(301, 301);
    for x in {1..grid.width() as isize} {
        for y in {1..grid.height() as isize} {
            *grid.get_mut(x, y).unwrap() = power_level(x, y, grid_serial);
        }
    }
    grid
}

fn populate_power_regions(power_grid: &Grid, region_size: usize) -> Grid {
    let mut region_grid = Grid::new(power_grid.width(), power_grid.height());
    for x in {1..region_grid.width() as isize} {
        for y in {1..region_grid.height() as isize} {
            let power = power_grid[(x, y)];
            for xshift in {0..region_size as i32} {
                for yshift in {0..region_size as i32} {
                    let x = x as i32 - xshift;
                    let y = y as i32 - yshift;
                    if let Some(v) = region_grid.get_mut(x as isize, y as isize) {
                        *v += power;
                    }
                }
            }
        }
    }
    region_grid
}

fn find_high_power_region(region_grid: &Grid) -> (isize, isize) {
    let mut max_cords = (0,0);
    let mut max_power = -1000;

    for x in {1..region_grid.width() as isize} {
        for y in { 1..region_grid.width() as isize } {
            let power = region_grid[(x, y)];
            if max_power < power {
                max_cords = (x, y);
                max_power = power;
            }
        }
    }

    max_cords
}

fn widen_square(values: &Grid, result: &mut Grid, square_length: isize) {
    for x in {1..values.width() as isize} {
        for y in {1..values.height() as isize} {
            let value = values[(x, y)];
            for x_ring in {x-square_length+1..x+1} {
                if let Some(v) = result.get_mut(x_ring, y-square_length+1) {
                    *v += value;
                }
            }

            for y_ring in {y-square_length+2..y+1} { // don't double-count
                if let Some(v) = result.get_mut(x-square_length+1, y_ring) {
                    *v += value;
                }
            }
        }
    }
}

fn find_high_power_region_size(power_grid: &Grid, max: isize) -> (isize, isize, isize) {
    let mut max_cords_size = (0,0,0);
    let mut max_power = -1000;

    let mut region_grid = Grid::new(power_grid.width(), power_grid.height());
    for region_size in {1..max+1} {
        widen_square(&power_grid, &mut region_grid, region_size);
        for x in { 1..region_grid.width() as isize } {
            for y in { 1..region_grid.height() as isize } {
                let power = region_grid[(x, y)];
                if max_power < power {
                    max_cords_size = (x, y, region_size);
                    max_power = power;
                }
            }
        }
    }

    max_cords_size
}

// Same idea as aoc3
// Uses isize in several places for ease of use, where usize would be more appropriate
// maybe there's a better way that still uses usize
mod grid {
    use std::fmt;
    use std::fmt::Write;
    use std::ops::Index;

    #[derive(Debug, Eq, PartialEq)]
    pub struct Grid {
        grid: Vec<i32>,
        width: usize,
    }

    impl Grid {
        pub fn new(width: usize, height: usize) -> Grid {
            Grid { grid: vec![0; width * height], width }
        }

        pub fn width(&self) -> usize { self.width }
        pub fn height(&self) -> usize { self.grid.len() / self.width }

        pub fn get(&self, x: isize, y: isize) -> Option<&i32> {
            let index = x + self.width as isize * y;
            if index < 0 || index >= self.grid.len() as isize { return None }
            self.grid.get(index as usize)
        }

        pub fn get_mut(&mut self, x: isize, y: isize) -> Option<&mut i32> {
            let index = x + self.width as isize * y;
            if index < 0 || index >= self.grid.len() as isize { return None }
            self.grid.get_mut(index as usize)
        }
    }

    impl Index<(isize, isize)> for Grid {
        type Output = i32;

        fn index(&self, coordinate: (isize, isize)) -> &i32 {
            self.get(coordinate.0, coordinate.1).unwrap()
        }
    }

    impl fmt::Display for Grid {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            // See also https://stackoverflow.com/q/30320083#comment92146674_30320443
            let mut out = String::new();
            for i in 0..self.grid.len() {
                match self.grid[i] {
                    v if v > -100 && v < 100 => {
                        write!(&mut out, "{:4}", v).expect("writing to string");
                    },
                    _ => out.push_str("   *"),
                }
                if i % self.width == self.width - 1 {
                    out.push('\n');
                }
            }
            write!(f, "{}", out)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn grid_str() {
            let grid = Grid::new(2, 3);
            assert_eq!(grid.to_string(), "   0   0\n   0   0\n   0   0\n");
        }

        #[test]
        fn grid_bounds() {
            let mut grid = Grid::new(3, 2);
            *grid.get_mut(0, 0).unwrap() = 1;
            *grid.get_mut(2, 0).unwrap() = 2;
            *grid.get_mut(0, 1).unwrap() = 3;
            *grid.get_mut(2, 1).unwrap() = 4;
            assert_eq!(grid.to_string(), "   1   0   2\n   3   0   4\n");
        }
    }
}
pub use self::grid::Grid;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_level_examples() {
        assert_eq!(power_level(3, 5, 8), 4);
        assert_eq!(power_level(122, 79, 57), -5);
        assert_eq!(power_level(217, 196, 39), 0);
        assert_eq!(power_level(101, 153, 71), 4);
    }

    #[test]
    fn cordinate_examples() {
        let power_grid = populate_power_grid(18);
        let region_grid = populate_power_regions(&power_grid, 3);
        assert_eq!(find_high_power_region(&region_grid), (33, 45));

        let power_grid = populate_power_grid(42);
        let region_grid = populate_power_regions(&power_grid, 3);
        assert_eq!(find_high_power_region(&region_grid), (21, 61));
    }

    fn copy_region(source: &Grid, min: (isize, isize), max: (isize, isize)) -> Grid {
        let mut result = Grid::new((max.0 - min.0 + 1) as usize, (max.1 - min.1 + 1) as usize);
        for x in {min.0..max.0+1} {
            for y in {min.1..max.1+1} {
                *result.get_mut(x-min.0, y-min.1).expect("should exist") = source[(x,y)];
            }
        }
        result
    }

    #[test]
    fn widen_squares() {
        let values = copy_region(&populate_power_grid(18), (32,44), (36, 48));
        let mut results = Grid::new(values.width(), values.height());
        widen_square(&values, &mut results, 1);
        widen_square(&values, &mut results, 2);
        widen_square(&values, &mut results, 3);
        assert_eq!(results[(1,1)], 29);

        let values = copy_region(&populate_power_grid(42), (20,60), (24, 64));
        let mut results = Grid::new(values.width(), values.height());
        widen_square(&values, &mut results, 1);
        widen_square(&values, &mut results, 2);
        widen_square(&values, &mut results, 3);
        assert_eq!(results[(1,1)], 30);
    }

    #[test]
    fn coordinate_size_examples() {
        // heuristically, going beyond size 20 seemed unnecessary
        let max_region = 20;
        let power_grid = populate_power_grid(18);
        assert_eq!(find_high_power_region_size(&power_grid, max_region), (90, 269, 16));

        let power_grid = populate_power_grid(42);
        assert_eq!(find_high_power_region_size(&power_grid, max_region), (232, 251, 12));
    }
}