use std::cmp::min;
use std::fs;
use std::collections::HashMap;
use crate::euclid::{point, Point};

pub fn advent() {
    let ast = read_data();
    let distances = plot(&ast);
    println!("Furthest Room: {}", distances.values().max().unwrap());
    println!("Rooms 1000+ away: {}",
             distances.values().filter(|&&d| d >= 1000).count());
}

fn read_data() -> Ast {
    fs::read_to_string("data/day20.txt").expect("Cannot open").trim_end()
        .parse().expect("Cannot parse")
}

fn plot(ast: &Ast) -> HashMap<Point, u32> {
    let mut map = HashMap::new();
    map.insert(point(0, 0), 0);
    ast.visit(|p, q| {
        let to = *map.get(&p)
            .expect("First point should have a known distance") + 1;
        map.entry(q)
            .and_modify(|v| *v = min(*v, to))
            .or_insert(to);
    });
    map
}

mod ast {
    use std::fmt;
    use std::str::FromStr;
    use std::fmt::Write;
    use itertools::Itertools;
    use crate::euclid::{point, Point, vector, Vector};
    use std::collections::HashSet;

    #[derive(Eq, PartialEq, Debug)]
    pub enum Path {
        LITERAL(char),
        GROUP(Vec<Path>),
        SEQUENCE(Vec<Path>),
    }

    impl Path {
        fn dir_to_vec(dir: char) -> Vector {
            match dir {
                'N' => vector(0, -1),
                'S' => vector(0, 1),
                'E' => vector(1, 0),
                'W' => vector( -1, 0),
                x => panic!("Invalid dir: {}", x),
            }
        }

        fn visit<F>(&self, coord: Point, visitor: &mut F) -> HashSet<Point>
            where F: FnMut(Point, Point) {
            match self {
                Path::LITERAL(dir) => {
                    let next_coord = coord + Path::dir_to_vec(*dir);
                    visitor(coord, next_coord);
                    [next_coord].iter().cloned().collect()
                },
                Path::SEQUENCE(children) => {
                    let mut next_coords = HashSet::new();
                    next_coords.insert(coord);
                    for child in children {
                        let child_cords: HashSet<_> = next_coords.drain().collect();
                        for child_cord in child_cords {
                            let result = child.visit(child_cord, visitor);
                            result.iter().for_each(|&c| { next_coords.insert(c); });
                        }
                    }
                    next_coords
                },
                Path::GROUP(nodes) => {
                    let mut next_coords = HashSet::new();
                    for node in nodes {
                        node.visit(coord, visitor).iter()
                            .for_each(|&c| { next_coords.insert(c); });
                    }
                    next_coords
                },
            }
        }
    }

    impl fmt::Display for Path {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Path::LITERAL(c) => write!(f, "{}", c),
                Path::GROUP(group) => {
                    let mut out = String::new();
                    out.push('(');
                    write!(&mut out, "{}", group.iter().map(|g| g.to_string()).join("|"))?;
                    out.push(')');
                    write!(f, "{}", out)
                },
                Path::SEQUENCE(seq) => {
                    let mut out = String::new();
                    for p in seq {
                        write!(&mut out, "{}", p)?;
                    }
                    write!(f, "{}", out)
                }
            }
        }
    }

    // Helper for Ast.FromStr
    // Tuple is (ValidSubtree, IndexOfFirstUnexpectedChar)
    fn tokenize(s: &[char]) -> Result<(Path, usize), String> {
        let mut tokens = Vec::new();
        let mut pos = 0;

        while pos < s.len() {
            match s[pos] {
                'N'|'S'|'E'|'W' => {
                    tokens.push(Path::LITERAL(s[pos]));
                    pos += 1;
                },
                '(' => {
                    let mut group = Vec::new();
                    loop {
                        pos += 1;
                        let (path, sub_pos) = tokenize(&s[pos..])?;
                        group.push(path);
                        pos = pos + sub_pos;
                        if pos >= s.len() {
                            return Err(format!("Expected ), hit end of input"));
                        }
                        if s[pos] == ')' { pos += 1; break; }
                        if s[pos] != '|' {
                            return Err(format!("Expected grouping marker, found {}", s[pos]));
                        }
                    }
                    tokens.push(Path::GROUP(group));
                },
                '|'|')' => {
                    break; // return whatever we've collected so far, which might be nothing
                },
                c => { return Err(format!("Unexepected char {}", c)); },
            }
        }

        if tokens.len() == 1 {
            return Ok((tokens.pop().expect("Must exist"), pos));
        }
        Ok((Path::SEQUENCE(tokens), pos))
    }

    #[derive(Debug)]
    pub struct Ast {
        root: Path,
    }

    impl Ast {
        pub fn visit(&self, mut visitor: impl FnMut(Point, Point)) {
            self.root.visit(point(0, 0), &mut visitor);
        }
    }

    impl FromStr for Ast {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, String> {
            let chars: Vec<_> = s.chars().collect();
            if chars[0] != '^' {
                return Err(format!("Unexpected first char: {}", chars[0]));
            }
            if chars[chars.len()-1] != '$' {
                return Err(format!("Unexpected last char: {}", chars[chars.len()-1]));
            }
            let chars = &chars[1..chars.len()-1];
            let (path, len) = tokenize(chars)?;
            if len == chars.len() {
                Ok(Ast { root: path })
            } else {
                Err(format!("Incomplete parse; stopped at char {}", len))
            }
        }
    }
    impl fmt::Display for Ast {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "^{}$", self.root)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        parameterized_test::create!{ to_tokens, (regex, expected_pos), {
            let (path, pos) = tokenize(&regex.chars().collect::<Vec<_>>()).unwrap();
            println!("{}:{}", path, pos);
            assert_eq!(path.to_string(), regex[..pos]);
            assert_eq!(pos, expected_pos);
        }}
        to_tokens! {
            basic: ("WNE", 3),
            basic_group: ("(WNE)", 5),
            trailing_paren: ("ENW)NE", 3),
            trailing_pipe: ("ENW|NE", 3),
            two_group: ("(NW|NE)", 7),
            empty_group: ("(NW|)", 5),
            three_group: ("(N|W|NE)", 8),
            nested_group: ("(N|(W)|NE)", 10),
            deep_nested_group: ("(N|(W|(NS|E))|NE)", 17),
        }

        parameterized_test::create!{ invalid_tokens, regex, {
            tokenize(&regex.chars().collect::<Vec<_>>()).unwrap_err();
        }}
        invalid_tokens! {
            invalid_chars: "XYZ",
            bad_nesting: "N(S",
        }

        parameterized_test::create!{ to_ast, regex, {
            let ast = regex.parse::<Ast>();
            println!("{:?}", ast);
            assert_eq!(regex.parse::<Ast>().unwrap().to_string(), regex);
        }}
        to_ast! {
            basic: "^WNE$",
            basic_group: "^(WNE)$",
            far_last_room: "^ENWWW(NEEE|SSE(EE|N))$",
            long: "^ENNWSWW(NEWS|)SSSEEN(WNSE|)EE(SWEN|)NNN$",
            longer: "^ESSWWN(E|NNENN(EESS(WNSE|)SSS|WWWSSSSE(SW|NNNE)))$",
            longest: "^WSSEESWWWNW(S|NENNEEEENN(ESSSSW(NWSW|SSEN)|WSWWN(E|WWS(E|SS))))$",
        }

        #[test]
        fn visit() {
            let ast: Ast = "^(E|SE)ES(E|S)ES$".parse().unwrap();
            let mut from = HashSet::new();
            let mut to = HashSet::new();
            ast.visit(|p, q| {
                // verify that we're traversing in-order (all east or south)
                assert!(p.x <= q.x);
                assert!(p.y <= q.y);
                // verify the next node is one step away
                assert_eq!((p-q).grid_len(), 1);
                from.insert(p);
                to.insert(q);
            });

            let sources: HashSet<_> = [point(0, 0)].iter().cloned().collect();
            let dests: HashSet<_> = [point(3, 4), point(4, 3)].iter().cloned().collect();
            assert_eq!(from.difference(&to).cloned().collect::<HashSet<_>>(), sources);
            assert_eq!(to.difference(&from).cloned().collect::<HashSet<_>>(), dests);
        }
    }
}
pub use self::ast::Ast;

#[cfg(test)]
mod tests {
    use super::*;

    parameterized_test::create!{ min_paths, (regex, expected), {
            let ast: Ast = regex.parse().unwrap();
            let distances = plot(&ast);
            let max = *distances.values().max().unwrap();
            assert_eq!(max, expected);
        }}
    min_paths! {
            basic: ("^WNE$", 3),
            basic_group: ("^(WNE)$", 3),
            far_last_room: ("^ENWWW(NEEE|SSE(EE|N))$", 10),
            long: ("^ENNWSWW(NEWS|)SSSEEN(WNSE|)EE(SWEN|)NNN$", 18),
            longer: ("^ESSWWN(E|NNENN(EESS(WNSE|)SSS|WWWSSSSE(SW|NNNE)))$", 23),
            longest: ("^WSSEESWWWNW(S|NENNEEEENN(ESSSSW(NWSW|SSEN)|WSWWN(E|WWS(E|SS))))$", 31),
        }
}