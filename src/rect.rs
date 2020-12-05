pub struct Rect {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Rect {
        assert!(w > 0);
        assert!(h > 0);

        Rect {
            x1: x,
            y1: y,
            x2: x + w - 1,
            y2: y + h - 1,
        }
    }

    /// Returns true if `other` (plus `margin`) overlaps this Rect.
    pub fn intersects(&self, other: &Rect, margin: i32) -> bool {
        other.x2 + margin >= self.x1
            && other.x1 - margin <= self.x2
            && other.y2 + margin >= self.y1
            && other.y1 - margin <= self.y2
    }

    pub fn center(&self) -> (i32, i32) {
        (
            (self.x2 - self.x1) / 2 + self.x1,
            (self.y2 - self.y1) / 2 + self.y1,
        )
    }
}
