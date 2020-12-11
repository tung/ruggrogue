use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
};

/// A trait for a map that paths can be found in using [AStarIter].
pub trait PathableMap {
    /// `min_x`, `min_y`, `max_x`, `max_y`.  Note that the latter two are inclusive.
    fn bounds(&self) -> (i32, i32, i32, i32);

    /// Returns `true` if the tile at the given coordinates is blocked.
    fn is_blocked(&self, x: i32, y: i32) -> bool;
}

/// Iterator that steps through each tile in a path found by [AStarIter::new].  Use this to
/// calculate the shortest path between two positions on a map.
pub struct AStarIter {
    came_from: HashMap<(i32, i32), (i32, i32)>,
    current_pos: Option<(i32, i32)>,
}

impl AStarIter {
    /// Calculate the shortest path between `start` and `dest` on the given map using the A*
    /// algorithm.  If a path is not found, this iterator will return [None] when calling
    /// [Iterator::next].
    ///
    /// If `circle_dist` is true, diagonal steps will be considered longer than cardinal steps.
    ///
    /// `explore_limit` sets the maximum number of tiles to explore before giving up; set this to
    /// zero to be limited only by the bounds of the map.
    pub fn new<T: PathableMap>(
        map: &T,
        start: (i32, i32),
        dest: (i32, i32),
        circle_dist: bool,
        explore_limit: i32,
    ) -> AStarIter {
        // (priority, (x, y))
        let mut frontier: BinaryHeap<(Reverse<i32>, (i32, i32))> = BinaryHeap::new();
        // ((x, y), (from_x, from_y))
        let mut came_from: HashMap<(i32, i32), (i32, i32)> = HashMap::new();
        // ((x, y), cost)
        let mut cost_so_far: HashMap<(i32, i32), i32> = HashMap::new();

        // Find the path starting from dest back to the start.
        frontier.push((Reverse(0), dest));
        cost_so_far.insert(dest, 0);

        let adjacent: [(i32, i32); 8] = [
            (-1, 0), // cardinals
            (1, 0),
            (0, -1),
            (0, 1),
            (-1, -1), // diagonals
            (-1, 1),
            (1, -1),
            (1, 1),
        ];
        let (min_x, min_y, max_x, max_y) = map.bounds();
        let mut steps = 0;
        let mut path_found = false;

        while (explore_limit <= 0 || steps < explore_limit) && !frontier.is_empty() {
            let (_, current) = frontier.pop().unwrap();
            let current_cost = *cost_so_far.get(&current).unwrap();

            if current == start {
                path_found = true;
                break;
            }

            for (i, (dx, dy)) in adjacent.iter().enumerate() {
                let next_x = current.0 + dx;
                let next_y = current.1 + dy;

                if next_x >= min_x && next_x <= max_x && next_y >= min_y && next_y <= max_y {
                    let next = (next_x, next_y);

                    if next == start || !map.is_blocked(next_x, next_y) {
                        let next_cost = current_cost
                            + if i >= 4 {
                                if circle_dist {
                                    141
                                } else {
                                    101
                                }
                            } else {
                                100
                            };

                        if next_cost < *cost_so_far.get(&next).unwrap_or(&i32::MAX) {
                            let priority = next_cost
                                + 100
                                    * if circle_dist {
                                        let x_diff = (next.0 - start.0).abs() as f32;
                                        let y_diff = (next.1 - start.1).abs() as f32;

                                        (x_diff * x_diff + y_diff * y_diff).sqrt() as i32
                                    } else {
                                        std::cmp::max(
                                            (next.0 - start.0).abs(),
                                            (next.1 - start.1).abs(),
                                        )
                                    };

                            frontier.push((Reverse(priority), next));
                            came_from.insert(next, current);
                            cost_so_far.insert(next, next_cost);
                        }
                    }
                }
            }

            steps += 1;
        }

        AStarIter {
            came_from,
            current_pos: if path_found { Some(start) } else { None },
        }
    }
}

impl Iterator for AStarIter {
    type Item = (i32, i32);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current_pos) = self.current_pos {
            self.current_pos = self.came_from.get(&current_pos).copied();
            Some(current_pos)
        } else {
            None
        }
    }
}
