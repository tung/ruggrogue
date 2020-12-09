use shipyard::EntityId;
use std::collections::HashMap;

pub struct FieldOfView {
    pub tiles: HashMap<(i32, i32), bool>,
    pub range: i32,
    pub dirty: bool,
    pub mark: bool,
}

impl FieldOfView {
    pub fn new(range: i32) -> FieldOfView {
        assert!(range > 0);

        FieldOfView {
            tiles: HashMap::new(),
            range,
            dirty: true,
            mark: false,
        }
    }
}

pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl From<&Position> for (i32, i32) {
    fn from(pos: &Position) -> Self {
        (pos.x, pos.y)
    }
}

impl From<(i32, i32)> for Position {
    fn from((x, y): (i32, i32)) -> Self {
        Position { x, y }
    }
}

pub struct Renderable {
    pub ch: char,
    pub fg: [f32; 4],
    pub bg: [f32; 4],
}

pub struct Player;

pub struct PlayerId(pub EntityId);
