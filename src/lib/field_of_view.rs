use super::BoundedMap;

/// Shape of field of view, to be used to [field_of_view].
pub enum FovShape {
    /// Square FOV.
    Square,
    /// Exact circular FOV.  Creates a bump of vision at the cardinal edges.
    Circle,
    /// Circular FOV extended by half a space to round out the cardinal bumps, though the
    /// additional tiles will not strictly be within range.
    CirclePlus,
}

/// A map-like trait that can be sent into [field_of_view] to calculate a field of view.
pub trait ViewableField {
    /// Returns `true` if the tile at the given coordinates is opaque.
    fn is_opaque(&self, x: i32, y: i32) -> bool;
}

/// Iterator returned by [field_of_view] that iterates over each tile in the field of view.
///
/// Each call to [FovIter::next] returns `x`, `y` and `symmetric`, the last of which is `true` if
/// the starting position and tile in question are in each other's fields of view.
pub struct FovIter<'a, T: BoundedMap + ViewableField> {
    map: &'a T,
    start_pos: (i32, i32),
    range: i32,
    fov_shape: FovShape,

    bounds: (i32, i32, i32, i32), // min_x, min_y, max_x, max_y
    max_dist2: i32,
    sights_even: Vec<((i32, i32), (i32, i32))>, // ((low_dy, low_dx), (high_dy, high_dx))
    sights_odd: Vec<((i32, i32), (i32, i32))>,  // ^
    low_y: i32,
    high_y: i32,
    low_sight_angle: Option<(i32, i32)>,

    octant: Option<i32>,
    x: Option<i32>,
    s: Option<usize>,
    y: Option<i32>,
}

impl<T: BoundedMap + ViewableField> FovIter<'_, T> {
    /// Advance one step through the field of view calculation.  Returns `x`, `y` and `symmetric` if
    /// this step contains a tile in the field of view.
    ///
    /// Iterates through the starting position, then each octant, x, sight and y; the last four are
    /// all nested iterations with pre-loop setups, but no actual looping allowed, so the logic is
    /// pretty complicated.  At least it's relatively fast.
    fn advance(&mut self) -> Option<(i32, i32, bool)> {
        let mut out_pos = None;
        let mut out_symmetric = false;

        if self.octant.is_none() {
            // Exit early if field of view doesn't intersect map.
            if self.start_pos.0 + self.range < self.bounds.0
                || self.start_pos.0 - self.range > self.bounds.2
                || self.start_pos.1 + self.range < self.bounds.1
                || self.start_pos.1 - self.range > self.bounds.3
            {
                self.octant = Some(8);
            } else {
                // Visit the starting position.
                self.octant = Some(-1);
            }
        }

        let octant = self.octant.unwrap();

        if octant == -1 {
            out_pos = Some(self.start_pos);
            out_symmetric = true;
            self.octant = Some(octant + 1);
        } else if octant < 8 {
            if self.x.is_none() {
                // Kick off with sight of the full octant.
                self.sights_odd.clear();
                self.sights_odd.push(((0, 1), (1, 1)));
                self.x = Some(1);
            }

            let x = self.x.unwrap();

            if x <= self.range {
                let (min_x, min_y, max_x, max_y) = self.bounds;

                // Flip between using even and odd sights for current and next.
                let (current, next) = if x % 2 == 0 {
                    (&mut self.sights_even, &mut self.sights_odd)
                } else {
                    (&mut self.sights_odd, &mut self.sights_even)
                };

                if self.s.is_none() {
                    next.clear();
                    self.s = Some(0);
                }

                let s = self.s.unwrap();

                if s < current.len() {
                    let (low_angle, high_angle) = current[s];

                    if self.y.is_none() {
                        // Calculate the low and high tiles whose middle lines are cut by the angles.
                        self.low_y = (2 * x * low_angle.0 / low_angle.1 + 1) / 2;
                        self.high_y = (2 * x * high_angle.0 / high_angle.1 + 1) / 2;

                        // Sight to populate the next sequence of sights with.
                        self.low_sight_angle = None;

                        self.y = Some(self.low_y);
                    }

                    let y = self.y.unwrap();

                    let in_shape = match self.fov_shape {
                        FovShape::Square => true,
                        FovShape::Circle | FovShape::CirclePlus => x * x + y * y <= self.max_dist2,
                    };

                    if in_shape && y <= self.high_y {
                        let octant_data = [
                            (1, 0, 0, 1, true),
                            (0, 1, 1, 0, false),
                            (0, -1, 1, 0, true),
                            (-1, 0, 0, 1, false),
                            (-1, 0, 0, -1, true),
                            (0, -1, -1, 0, false),
                            (0, 1, -1, 0, true),
                            (1, 0, 0, -1, false),
                        ];
                        let (
                            real_x_from_x,
                            real_x_from_y,
                            real_y_from_x,
                            real_y_from_y,
                            include_edges,
                        ) = octant_data[octant as usize];

                        let real_x = self.start_pos.0 + x * real_x_from_x + y * real_x_from_y;
                        let real_y = self.start_pos.1 + x * real_y_from_x + y * real_y_from_y;

                        // The slope of the center of the bottom edge.
                        let low_mid_angle = (y * 2 - 1, x * 2);

                        let in_bounds = |x, y| x >= min_x && x <= max_x && y >= min_y && y <= max_y;
                        let angle_lt_or_eq = |(a_n, a_d), (b_n, b_d)| a_n * b_d <= b_n * a_d;

                        if in_bounds(real_x, real_y) && self.map.is_opaque(real_x, real_y) {
                            // Finish the current sight when hitting an opaque tile.
                            if self.low_sight_angle.is_some() {
                                next.push((self.low_sight_angle.unwrap(), low_mid_angle));
                                self.low_sight_angle = None;
                            }
                        } else if self.low_sight_angle.is_none() {
                            // Begin a new sight with the higher of the bottom center of the
                            // current tile and the low angle.
                            self.low_sight_angle = if angle_lt_or_eq(low_angle, low_mid_angle) {
                                Some(low_mid_angle)
                            } else {
                                Some(low_angle)
                            };
                        }

                        // Visit the tile.
                        if (include_edges || (y > 0 && y < x)) && in_bounds(real_x, real_y) {
                            out_pos = Some((real_x, real_y));
                            out_symmetric = angle_lt_or_eq(low_angle, (y, x))
                                && angle_lt_or_eq((y, x), high_angle);
                        }

                        self.y = Some(y + 1);
                    } else {
                        // Finish any sight left dangling.
                        if let Some(low_sight_angle) = self.low_sight_angle {
                            next.push((low_sight_angle, high_angle));
                        }

                        self.s = Some(s + 1);
                        self.y = None;
                    }
                } else {
                    self.x = Some(x + 1);
                    self.s = None;
                }
            } else {
                self.octant = Some(octant + 1);
                self.x = None;
            }
        }

        out_pos.map(|pos| (pos.0, pos.1, out_symmetric))
    }
}

impl<T: BoundedMap + ViewableField> Iterator for FovIter<'_, T> {
    type Item = (i32, i32, bool);

    /// Returns the next `(x, y, symmetric)` tuple, where `symmetric` means that the starting
    /// position and tile in question are in each other's fields of view.
    fn next(&mut self) -> Option<Self::Item> {
        let mut item;

        loop {
            item = self.advance();

            if let Some((x, y, _)) = item {
                if x >= self.bounds.0
                    && x <= self.bounds.2
                    && y >= self.bounds.1
                    && y <= self.bounds.3
                {
                    // Valid position.
                    break;
                }
            } else if self.octant.unwrap() >= 8 {
                // The field of view has been completely iterated over.
                break;
            }
        }

        item
    }
}

/// Calculate field of view with center-to-center visibility and diamond-shaped walls.  This
/// function uses fixed-point iterative shadow-casting, so it should be pretty fast.
///
/// `start_pos` are the (x, y) coordinates to calculate field of view from.
///
/// `range` must be non-negative.
pub fn field_of_view<T>(
    map: &'_ T,
    start_pos: (i32, i32),
    range: i32,
    fov_shape: FovShape,
) -> FovIter<'_, T>
where
    T: BoundedMap + ViewableField,
{
    assert!(range >= 0);

    let max_dist2 = match fov_shape {
        FovShape::Square => 0, // unused
        FovShape::Circle => range * range,
        FovShape::CirclePlus => range * (range + 1),
    };

    FovIter {
        map,
        start_pos,
        range,
        fov_shape,

        bounds: map.bounds(),
        max_dist2,
        sights_even: Vec::with_capacity(range as usize),
        sights_odd: Vec::with_capacity(range as usize),
        low_y: 0,
        high_y: 0,
        low_sight_angle: None,

        octant: None,
        x: None,
        s: None,
        y: None,
    }
}
