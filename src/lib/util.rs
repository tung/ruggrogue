use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Eq, PartialEq, Deserialize, Serialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const WHITE: Color = Color {
        r: 255,
        g: 255,
        b: 255,
    };
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0 };
    pub const GRAY: Color = Color {
        r: 128,
        g: 128,
        b: 128,
    };
    pub const DARK_GRAY: Color = Color {
        r: 64,
        g: 64,
        b: 64,
    };
    pub const LIGHT_GRAY: Color = Color {
        r: 192,
        g: 192,
        b: 192,
    };
    pub const RED: Color = Color { r: 255, g: 0, b: 0 };
    pub const GREEN: Color = Color { r: 0, g: 255, b: 0 };
    pub const BLUE: Color = Color { r: 0, g: 0, b: 255 };
    pub const YELLOW: Color = Color {
        r: 255,
        g: 255,
        b: 0,
    };
    pub const MAGENTA: Color = Color {
        r: 255,
        g: 0,
        b: 255,
    };
    pub const BROWN: Color = Color {
        r: 191,
        g: 92,
        b: 0,
    };
    pub const CYAN: Color = Color {
        r: 0,
        g: 255,
        b: 255,
    };
    pub const ORANGE: Color = Color {
        r: 255,
        g: 166,
        b: 0,
    };
    pub const PURPLE: Color = Color {
        r: 128,
        g: 0,
        b: 128,
    };
    pub const PINK: Color = Color {
        r: 255,
        g: 191,
        b: 204,
    };
}

impl From<(u8, u8, u8)> for Color {
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        Self { r, g, b }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Deserialize, Serialize)]
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
