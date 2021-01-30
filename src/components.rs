use bitvec::prelude::*;
use shipyard::EntityId;

pub struct BlocksTile;

pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
}

pub struct FieldOfView {
    pub tiles: BitVec,
    pub range: i32,
    span: i32,
    pub center: (i32, i32),
    pub dirty: bool,
}

impl FieldOfView {
    pub fn new(range: i32) -> FieldOfView {
        assert!(range >= 0);

        let span = 2 * range + 1;

        FieldOfView {
            tiles: bitvec![0; (span * span) as usize],
            range,
            span,
            center: (0, 0),
            dirty: true,
        }
    }

    fn idx(&self, (x, y): (i32, i32)) -> usize {
        let tx = x - self.center.0 + self.range;
        let ty = y - self.center.1 + self.range;
        (ty * self.span + tx) as usize
    }

    pub fn set(&mut self, pos: (i32, i32), value: bool) {
        let idx = self.idx(pos);
        self.tiles.set(idx, value);
    }

    pub fn get(&self, pos: (i32, i32)) -> bool {
        if (pos.0 - self.center.0).abs() <= self.range
            && (pos.1 - self.center.1).abs() <= self.range
        {
            self.tiles[self.idx(pos)]
        } else {
            false
        }
    }
}

pub struct Item;

pub struct Monster;

pub struct Name(pub String);

pub struct Player;

pub struct PlayerId(pub EntityId);

pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn dist(&self, other: &Position) -> i32 {
        std::cmp::max((other.x - self.x).abs(), (other.y - self.y).abs())
    }
}

impl From<&Position> for (i32, i32) {
    fn from(pos: &Position) -> Self {
        (pos.x, pos.y)
    }
}

impl From<&mut Position> for (i32, i32) {
    fn from(pos: &mut Position) -> Self {
        (pos.x, pos.y)
    }
}

impl From<(i32, i32)> for Position {
    fn from((x, y): (i32, i32)) -> Self {
        Position { x, y }
    }
}

pub struct Potion {
    pub heal_amount: i32,
}

pub struct Renderable {
    pub ch: char,
    pub fg: [f32; 4],
    pub bg: [f32; 4],
}
