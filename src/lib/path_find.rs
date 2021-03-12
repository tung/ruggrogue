use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
};

use super::BoundedMap;

/// A trait for a map that paths can be found in using [AStarIter].
pub trait PathableMap {
    /// Returns `true` if the tile at the given coordinates is blocked.
    fn is_blocked(&self, x: i32, y: i32) -> bool;
}

/// Iterator that steps through each tile in a path found by [find_path].
pub struct AStarIter {
    came_from: HashMap<(i32, i32), (i32, i32)>,
    current_pos: Option<(i32, i32)>,
    pub fallback: bool,
}

impl AStarIter {
    /// Returns true if this iterator represents a path to the closest reachable point to the
    /// destination, rather than the destination itself.
    pub fn is_fallback(&self) -> bool {
        self.fallback
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

const ADJACENT_TILES: [(i32, i32); 8] = [
    (-1, 0), // cardinals
    (1, 0),
    (0, -1),
    (0, 1),
    (-1, -1), // diagonals
    (-1, 1),
    (1, -1),
    (1, 1),
];

/// Calculate the shortest path from `start` to `dest` using the A* algorithm, returning the
/// closest point to `dest` that is reachable from `start`, which will be equal to `dest` if a path
/// was found.
///
/// The path data are stored in `came_from` where the keys are positions and the values are the
/// position that they came from; this means that the path is stored in reverse.
fn a_star<T: BoundedMap + PathableMap>(
    map: &T,
    start: (i32, i32),
    dest: (i32, i32),
    bound_pad: i32,
    came_from: &mut HashMap<(i32, i32), (i32, i32)>,
) -> (i32, i32) {
    // (priority, (x, y))
    let mut frontier: BinaryHeap<(Reverse<i32>, (i32, i32))> = BinaryHeap::new();
    // ((x, y), cost)
    let mut cost_so_far: HashMap<(i32, i32), i32> = HashMap::new();
    let (min_x, min_y, max_x, max_y) = if bound_pad == 0 {
        map.bounds()
    } else {
        let bounds = map.bounds();

        (
            std::cmp::max(bounds.0, std::cmp::min(start.0, dest.0) - bound_pad),
            std::cmp::max(bounds.1, std::cmp::min(start.1, dest.1) - bound_pad),
            std::cmp::min(bounds.2, std::cmp::max(start.0, dest.0) + bound_pad),
            std::cmp::min(bounds.3, std::cmp::max(start.1, dest.1) + bound_pad),
        )
    };
    let dist100 = |(x1, y1), (x2, y2)| {
        let x_diff = if x1 < x2 { x2 - x1 } else { x1 - x2 };
        let y_diff = if y1 < y2 { y2 - y1 } else { y1 - y2 };
        let (low_diff, high_diff) = if x_diff < y_diff {
            (x_diff, y_diff)
        } else {
            (y_diff, x_diff)
        };

        // Prefer axis-aligning with (x2, y2).
        low_diff * 141 + (high_diff - low_diff) * 99
    };
    let mut closest = start;
    let mut closest_cost = 0;
    let mut closest_dist = dist100(start, dest);

    frontier.push((Reverse(0), start));
    cost_so_far.insert(start, 0);

    while let Some((_, current)) = frontier.pop() {
        let current_cost = *cost_so_far.get(&current).unwrap();
        let current_dist = dist100(current, dest);

        if current_dist < closest_dist
            || (current_dist == closest_dist && current_cost < closest_cost)
        {
            closest = current;
            closest_cost = current_cost;
            closest_dist = current_dist;
        }

        if current == dest {
            break;
        }

        for (i, (dx, dy)) in ADJACENT_TILES.iter().enumerate() {
            let next_x = current.0 + dx;
            let next_y = current.1 + dy;

            if next_x >= min_x && next_x <= max_x && next_y >= min_y && next_y <= max_y {
                let next = (next_x, next_y);

                if next == dest || !map.is_blocked(next_x, next_y) {
                    let next_cost = current_cost + if i >= 4 { 141 } else { 100 };

                    if next_cost < *cost_so_far.get(&next).unwrap_or(&i32::MAX) {
                        frontier.push((Reverse(next_cost + dist100(next, dest)), next));
                        came_from.insert(next, current);
                        cost_so_far.insert(next, next_cost);
                    }
                }
            }
        }
    }

    closest
}

/// Find the shortest path from `start` to `dest` on the given map.
///
/// If `bound_pad` is non-zero, confine the search for the path to the rectangle created by the
/// `start` and `dest` points plus a padding of `bound_pad` positions, otherwise search the whole
/// map.
///
/// If a path cannot be found and `fallback_closest` is set, find the closest point to `dest`
/// reachable from `start` (within the `bound_pad` if given) and calculate the path towards that
/// point instead.
pub fn find_path<T: BoundedMap + PathableMap>(
    map: &T,
    start: (i32, i32),
    dest: (i32, i32),
    bound_pad: i32,
    fallback_closest: bool,
) -> AStarIter {
    let mut came_from: HashMap<(i32, i32), (i32, i32)> = HashMap::new();
    let closest = a_star(map, start, dest, bound_pad, &mut came_from);

    if closest == dest || fallback_closest {
        // Reverse the path from closest to start.
        let mut current = came_from.get(&closest).copied();
        let mut prev = closest;

        came_from.remove(&closest);

        while let Some(c) = current {
            let next = came_from.get(&c).copied();

            came_from.insert(c, prev);
            prev = c;
            current = next;
        }

        AStarIter {
            came_from,
            current_pos: Some(start),
            fallback: true,
        }
    } else {
        AStarIter {
            came_from,
            current_pos: None,
            fallback: false,
        }
    }
}
