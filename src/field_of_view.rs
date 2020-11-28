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
    /// `min_x`, `min_y`, `max_x`, `max_y`.  Note that the latter two are inclusive.
    fn bounds(&self) -> (i32, i32, i32, i32);

    /// Returns `true` if the tile at the given coordinates is opaque.
    fn is_opaque(&self, x: i32, y: i32) -> bool;

    /// Returns `true` if the tile should be visible even if its center point is not within the field
    /// of view.
    fn is_asymetrically_visible(&self, x: i32, y: i32) -> bool;
}

/// Scan a column of tiles for visibility according to `current` sights, populating `next` sights
/// for subsequent column scans.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
fn field_of_view_scan<T, U, V>(
    map: &T,
    start_pos: (i32, i32),
    fov_shape: &FovShape,
    visit: &mut U,
    in_bounds: V,
    max_dist2: i32,
    x: i32,
    (real_x_from_x, real_x_from_y, real_y_from_x, real_y_from_y): (i32, i32, i32, i32),
    include_edges: bool,
    current: &mut Vec<((i32, i32), (i32, i32))>,
    next: &mut Vec<((i32, i32), (i32, i32))>,
) where
    T: ViewableField,
    U: FnMut(i32, i32, bool),
    V: Fn(i32, i32) -> bool,
{
    let angle_lt_or_eq = |(a_n, a_d), (b_n, b_d)| a_n * b_d <= b_n * a_d;

    for (low_angle, high_angle) in current.iter() {
        // Calculate the low and high tiles whose middle lines are cut by the angles.
        let low_y = (2 * x * low_angle.0 / low_angle.1 + 1) / 2;
        let high_y = (2 * x * high_angle.0 / high_angle.1 + 1) / 2;

        // Sight to populate the next sequence of sights with.
        let mut low_sight_angle = None;

        for y in low_y..=high_y {
            // Distance culling.
            match fov_shape {
                FovShape::Circle | FovShape::CirclePlus => {
                    if x * x + y * y > max_dist2 {
                        continue;
                    }
                }
                FovShape::Square => {}
            }

            let real_x = start_pos.0 + real_x_from_x * x + real_x_from_y * y;
            let real_y = start_pos.1 + real_y_from_x * x + real_y_from_y * y;

            // The slope of the center of the bottom edge.
            let low_mid_angle = (y * 2 - 1, x * 2);

            if in_bounds(real_x, real_y) && map.is_opaque(real_x, real_y) {
                // Finish the current sight when hitting an opaque tile.
                if low_sight_angle.is_some() {
                    next.push((low_sight_angle.unwrap(), low_mid_angle));
                    low_sight_angle = None;
                }
            } else if low_sight_angle.is_none() {
                // Begin a new sight with the higher of the bottom center of the current tile and
                // the low angle.
                low_sight_angle = if angle_lt_or_eq(*low_angle, low_mid_angle) {
                    Some(low_mid_angle)
                } else {
                    Some(*low_angle)
                };
            }

            // Visit the tile.
            if (include_edges || (y > 0 && y < x)) && in_bounds(real_x, real_y) {
                let symmetric =
                    angle_lt_or_eq(*low_angle, (y, x)) && angle_lt_or_eq((y, x), *high_angle);

                if symmetric || map.is_asymetrically_visible(real_x, real_y) {
                    visit(real_x, real_y, symmetric);
                }
            }
        }

        // Finish any sight left dangling.
        if let Some(low_sight_angle) = low_sight_angle {
            next.push((low_sight_angle, *high_angle));
        }
    }
}

/// Calculate field of view with center-to-center visibility and diamond-shaped walls.  This
/// function uses fixed-point iterative shadow-casting approach, so it should be pretty fast.
///
/// `start_pos` are the (x, y) coordinates to calculate field of view from.  `range` must be
/// non-negative.  `visit` is a callback that takes `x`, `y` and `symmetric`, the last of which is
/// `true` when vision is symmetric at that tile, meaning that `start_pos` would be visible if
/// field of view was calculated from that tile.
pub fn field_of_view<T, U>(
    map: &T,
    start_pos: (i32, i32),
    range: i32,
    fov_shape: FovShape,
    mut visit: U,
) -> Result<(), &'static str>
where
    T: ViewableField,
    U: FnMut(i32, i32, bool),
{
    if range < 0 {
        return Err("invalid range");
    }

    let bounds = map.bounds();

    if start_pos.0 + range < bounds.0
        || start_pos.0 - range > bounds.2
        || start_pos.1 + range < bounds.1
        || start_pos.1 - range > bounds.3
    {
        // Exit early if field of view doesn't intersect map.
        return Ok(());
    }

    let in_bounds = |x, y| x >= bounds.0 && x <= bounds.2 && y >= bounds.1 && y <= bounds.3;

    // Squared maximum range for distance comparisons.
    let max_dist2 = match fov_shape {
        FovShape::Square => 0, // unused
        FovShape::Circle => range * range,
        FovShape::CirclePlus => range * (range + 1),
    };

    // real_x_from_x, real_x_from_y, real_y_from_x, real_y_from_y, include_edges
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

    // Low and high sight angles for the current scan and next scan using page flipping.
    // Angles are y change over x change.
    let mut sights_even: Vec<((i32, i32), (i32, i32))> = Vec::with_capacity(range as usize);
    let mut sights_odd: Vec<((i32, i32), (i32, i32))> = Vec::with_capacity(range as usize);

    // Visit the starting position.
    if in_bounds(start_pos.0, start_pos.1) {
        visit(start_pos.0, start_pos.1, true);
    }

    for (real_x_from_x, real_x_from_y, real_y_from_x, real_y_from_y, include_edges) in
        octant_data.iter()
    {
        // Kick off with sight of the full octant.
        sights_odd.clear();
        sights_odd.push(((0, 1), (1, 1)));

        for x in 1..=range {
            // Flip between using even and odd sights for current and next.
            if x % 2 == 0 {
                sights_odd.clear();
                field_of_view_scan(
                    map,
                    start_pos,
                    &fov_shape,
                    &mut visit,
                    in_bounds,
                    max_dist2,
                    x,
                    (
                        *real_x_from_x,
                        *real_x_from_y,
                        *real_y_from_x,
                        *real_y_from_y,
                    ),
                    *include_edges,
                    &mut sights_even,
                    &mut sights_odd,
                );
            } else {
                sights_even.clear();
                field_of_view_scan(
                    map,
                    start_pos,
                    &fov_shape,
                    &mut visit,
                    in_bounds,
                    max_dist2,
                    x,
                    (
                        *real_x_from_x,
                        *real_x_from_y,
                        *real_y_from_x,
                        *real_y_from_y,
                    ),
                    *include_edges,
                    &mut sights_odd,
                    &mut sights_even,
                );
            }
        }
    }

    Ok(())
}
