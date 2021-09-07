use serde::{Deserialize, Serialize};
use shipyard::EntityId;

use crate::{bitgrid::BitGrid, gamesym::GameSym, player::AutoRun};
use ruggrogue::util::{Color, Position};

#[derive(Deserialize, Serialize)]
pub struct AreaOfEffect {
    pub radius: i32,
}

#[derive(Deserialize, Serialize)]
pub struct Asleep {
    pub sleepiness: i32,
    pub last_hp: i32,
}

#[derive(Deserialize, Serialize)]
pub struct BlocksTile;

#[derive(Deserialize, Serialize)]
pub struct CombatBonus {
    pub attack: f32,
    pub defense: f32,
}

#[derive(Deserialize, Serialize)]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub attack: f32,
    pub defense: f32,
}

#[derive(Deserialize, Serialize)]
pub struct Consumable;

#[derive(Deserialize, Serialize)]
pub struct Coord(pub Position);

impl Coord {
    pub fn dist(&self, other: &Coord) -> i32 {
        std::cmp::max((other.0.x - self.0.x).abs(), (other.0.y - self.0.y).abs())
    }
}

#[derive(Deserialize, Serialize)]
pub enum EquipSlot {
    Weapon,
    Armor,
}

#[derive(Deserialize, Serialize)]
pub struct Equipment {
    pub weapon: Option<EntityId>,
    pub armor: Option<EntityId>,
}

#[derive(Deserialize, Serialize)]
pub struct Experience {
    pub level: i32,
    pub exp: u64,
    pub next: u64,
    pub base: u64,
}

#[derive(Deserialize, Serialize)]
pub struct FieldOfView {
    pub tiles: BitGrid,
    pub range: i32,
    pub center: (i32, i32),
    pub dirty: bool,
}

impl FieldOfView {
    pub fn new(range: i32) -> FieldOfView {
        assert!(range >= 0);

        let span = 2 * range + 1;

        FieldOfView {
            tiles: BitGrid::new(span, span),
            range,
            center: (0, 0),
            dirty: true,
        }
    }

    #[inline]
    fn offset_xy(&self, (x, y): (i32, i32)) -> (i32, i32) {
        (
            x - self.center.0 + self.range,
            y - self.center.1 + self.range,
        )
    }

    #[inline]
    pub fn set(&mut self, pos: (i32, i32), value: bool) {
        let offset_pos = self.offset_xy(pos);
        self.tiles.set_bit(offset_pos.0, offset_pos.1, value);
    }

    #[inline]
    pub fn get(&self, pos: (i32, i32)) -> bool {
        let offset_pos = self.offset_xy(pos);
        self.tiles.get_bit(offset_pos.0, offset_pos.1)
    }

    pub fn iter(&self) -> impl Iterator<Item = (i32, i32)> + '_ {
        let ys = (self.center.1 - self.range)..=(self.center.1 + self.range);

        ys.flat_map(move |y| {
            let xs = (self.center.0 - self.range)..=(self.center.0 + self.range);

            std::iter::repeat(y).zip(xs)
        })
        .filter(move |(y, x)| self.get((*x, *y)))
        .map(move |(y, x)| (x, y))
    }

    pub fn mark_seen(&self, seen: &mut BitGrid) {
        self.tiles
            .apply_bits_onto(seen, self.center.0 - self.range, self.center.1 - self.range);
    }
}

#[derive(Deserialize, Serialize)]
pub struct GivesExperience(pub u64);

pub enum HurtBy {
    Someone(EntityId),
    Starvation,
}

#[derive(Deserialize, Serialize)]
pub struct InflictsDamage {
    pub damage: i32,
}

#[derive(Deserialize, Serialize)]
pub struct InflictsSleep {
    pub sleepiness: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Inventory {
    pub items: Vec<EntityId>,
}

#[derive(Deserialize, Serialize)]
pub struct Item;

#[derive(Deserialize, Serialize)]
pub struct Monster;

#[derive(Deserialize, Serialize)]
pub struct Name(pub String);

#[derive(Deserialize, Serialize)]
pub struct Nutrition(pub i32);

#[derive(Deserialize, Serialize)]
pub struct Player {
    #[serde(skip)]
    pub auto_run: Option<AutoRun>,
}

#[derive(Deserialize, Serialize)]
pub struct ProvidesHealing {
    pub heal_amount: i32,
}

#[derive(Deserialize, Serialize)]
pub struct Ranged {
    pub range: i32,
}

#[derive(Deserialize, Serialize)]
pub struct RenderOnFloor;

#[derive(Deserialize, Serialize)]
pub struct RenderOnMap;

#[derive(Deserialize, Serialize)]
pub struct Renderable {
    pub sym: GameSym,
    pub fg: Color,
    pub bg: Color,
}

#[derive(Deserialize, Serialize)]
pub struct Stomach {
    pub fullness: i32,
    pub max_fullness: i32,
    pub sub_hp: i32,
}

#[derive(Deserialize, Serialize)]
pub struct Tally {
    pub damage_dealt: u64,
    pub damage_taken: u64,
    pub kills: u64,
}

#[derive(Deserialize, Serialize)]
pub struct Victory;
