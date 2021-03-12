#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl From<(u8, u8, u8)> for Color {
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        Self { r, g, b }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl From<(i32, i32)> for Position {
    fn from((x, y): (i32, i32)) -> Self {
        Self { x, y }
    }
}

impl From<Position> for (i32, i32) {
    fn from(pos: Position) -> Self {
        (pos.x, pos.y)
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Size {
    pub w: u32,
    pub h: u32,
}

impl From<(u32, u32)> for Size {
    fn from((w, h): (u32, u32)) -> Self {
        Self { w, h }
    }
}
