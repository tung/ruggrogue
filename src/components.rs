use shipyard::EntityId;
use std::collections::HashSet;

pub struct FieldOfView(pub HashSet<(i32, i32)>);

pub struct Position {
    pub x: i32,
    pub y: i32,
}

pub struct Renderable {
    pub ch: char,
    pub fg: [f32; 4],
    pub bg: [f32; 4],
}

pub struct Player;

pub struct PlayerId(pub EntityId);
