mod pathfinding {
    use std::collections::{VecDeque, HashMap, BinaryHeap};
    use std::cmp::Ordering;

    // References:
    // https://www.redblobgames.com/pathfinding/a-star/introduction.html
    // http://theory.stanford.edu/~amitp/GameProgramming/AStarComparison.html
    // https://doc.rust-lang.org/std/collections/binary_heap/
    pub trait Graph {
        type Node: Clone + std::fmt::Debug + Eq + core::hash::Hash;

        fn neighbors(&self, source: &Self::Node) -> Vec<Edge<Self::Node>>;

        fn bfs(&self, start: &Self::Node, goal: &Self::Node) -> Option<Vec<Self::Node>> {
            let mut frontier = VecDeque::new();
            frontier.push_back(start.clone());
            let mut routes = HashMap::new();
            routes.insert(start.clone(), start.clone()); // careful, potential infinite loop

            while ! frontier.is_empty() && !routes.contains_key(goal) {
                let current = frontier.pop_front().expect("frontier is not empty");
                for edge in self.neighbors(&current) {
                    assert_eq!(edge.weight(), 1, "BFS does not support weighted edges: {:?}", edge);
                    let next = edge.dest();
                    if !routes.contains_key(next) {
                        frontier.push_back(next.clone());
                        routes.insert(next.clone(), current.clone());
                    }
                }
            }

            let mut path = Vec::new();
            let mut current = goal.clone();
            while current != *start {
                if let Some(next) = routes.get(&current) {
                    path.push(current);
                    current = next.clone();
                } else {
                    return None;
                }
            }
            path.push(start.clone());
            path.reverse();
            Some(path)
        }

        fn dijkstras(&self, start: &Self::Node, goal: &Self::Node) -> Option<Vec<Edge<Self::Node>>> {
            let mut frontier = BinaryHeap::new();
            let mut costs = HashMap::new();
            let mut routes = HashMap::new();
            frontier.push(State { cost: 0, node: start.clone() });
            costs.insert(start.clone(), 0);
            routes.insert(start.clone(),
                          Edge::new(0, start.clone(), start.clone())); // careful, potential infinite loop

            while let Some(State { cost: _, node: current }) = frontier.pop() {
                if routes.contains_key(goal) { break; }
                let current_cost = *costs.get(&current).expect("Current cost should be known");
                for edge in self.neighbors(&current) {
                    let next = edge.dest();
                    let next_cost = current_cost + edge.weight();

                    let prior_next_cost = costs.get(&next);
                    if prior_next_cost.is_none() || *prior_next_cost.expect("Not-none") > next_cost {
                        costs.insert(next.clone(), next_cost);
                        frontier.push(State { cost: next_cost, node: next.clone() });
                        routes.insert(next.clone(), edge.clone());
                    }
                }
            }

            let mut path = Vec::new();
            let mut current = goal.clone();
            while current != *start {
                if let Some(next) = routes.get(&current) {
                    path.push(next.clone());
                    current = next.source().clone();
                } else {
                    return None;
                }
            }
            path.reverse();
            Some(path)
        }
    }

    #[derive(Clone, Debug)]
    pub struct Edge<N: Clone + std::fmt::Debug> {
        weight: i32,
        source: N,
        dest: N,
    }

    impl<N: Clone + std::fmt::Debug> Edge<N> {
        pub fn new(weight: i32, source: N, dest: N) -> Edge<N> {
            Edge { weight, source, dest }
        }

        pub fn weight(&self) -> i32 { self.weight }
        pub fn source(&self) -> &N { &self.source }
        pub fn dest(&self) -> &N { &self.dest }
    }

    impl<N: Copy + std::fmt::Debug> Copy for Edge<N> {}

    #[derive(Clone, Debug)]
    struct State<N: Clone + std::fmt::Debug> {
        cost: i32,
        node: N,
    }

    impl<N: Copy + std::fmt::Debug> Copy for State<N> {}

    // We don't implement Eq because it's not well defined, but Ord requires it exist
    impl<N: Clone + std::fmt::Debug> PartialEq for State<N> {
        fn eq(&self, _: &Self) -> bool {
            unimplemented!()
        }
    }

    impl<N: Clone + std::fmt::Debug> Eq for State<N> {}

    impl<N: Clone + std::fmt::Debug> Ord for State<N> {
        fn cmp(&self, other: &State<N>) -> Ordering {
            other.cost.cmp(&self.cost)
        }
    }

    impl<N: Clone + std::fmt::Debug> PartialOrd for State<N> {
        fn partial_cmp(&self, other: &State<N>) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }
}
pub use self::pathfinding::{Edge,Graph};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::euclid::{point,Point,vector};
    use std::collections::HashSet;

    struct BasicGraph {
        blocked: HashSet<Point>,
    }

    impl BasicGraph {
        fn new(blocked: &[Point]) -> BasicGraph {
            BasicGraph { blocked: blocked.iter().cloned().collect() }
        }
    }

    impl Graph for BasicGraph {
        type Node = Point;

        fn neighbors(&self, source: &Self::Node) -> Vec<Edge<Self::Node>> {
            if self.blocked.contains(source) { return vec!(); }

            vec!(vector(0, 1), vector(1, 0), vector(0, -1), vector(-1, 0)).iter()
                .map(|v| source + v)
                .filter(|p| !self.blocked.contains(p))
                .map(|d| Edge::new(1, source.clone(), d.clone()))
                .collect()
        }
    }

    #[test]
    fn direct() {
        let graph = BasicGraph::new(&vec!());
        let start = point(1, 1);
        let goal = point(3, 4);

        let bfs_route = graph.bfs(&start, &goal).unwrap();
        assert_eq!(bfs_route.len(), 6);
        assert_eq!(bfs_route[0], start);
        assert_eq!(bfs_route[bfs_route.len()-1], goal);

        let djk_route = graph.dijkstras(&start, &goal).unwrap();
        assert_eq!(djk_route.len(), 5);
        assert_eq!(djk_route[0].source(), &start);
        assert_eq!(djk_route[djk_route.len()-1].dest(), &goal);
    }

    #[test]
    fn wall() {
        let graph = BasicGraph::new(&vec!(
            point(0, 3), point(1, 3), point(2, 3), point(3, 3), point(4, 3)
        ));
        let start = point(1, 1);
        let goal = point(3, 4);

        let bfs_route = graph.bfs(&start, &goal).unwrap();
        assert_eq!(bfs_route.len(), 10);
        assert_eq!(bfs_route[0], start);
        assert_eq!(bfs_route[bfs_route.len()-1], goal);

        let djk_route = graph.dijkstras(&start, &goal).unwrap();
        assert_eq!(djk_route.len(), 9);
        assert_eq!(djk_route[0].source(), &start);
        assert_eq!(djk_route[djk_route.len()-1].dest(), &goal);
    }
}