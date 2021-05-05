use shipyard::EntityId;

use crate::{bitgrid::BitGrid, gamesym::GameSym, player::AutoRun};
use ruggle::util::{Color, Position};

pub struct AreaOfEffect {
    pub radius: i32,
}

pub struct Asleep {
    pub sleepiness: i32,
    pub last_hp: i32,
}

pub struct BlocksTile;

pub struct CombatBonus {
    pub attack: i32,
    pub defense: i32,
}

pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub attack: i32,
    pub defense: i32,
}

pub struct Consumable;

pub struct Coord(pub Position);

impl Coord {
    pub fn dist(&self, other: &Coord) -> i32 {
        std::cmp::max((other.0.x - self.0.x).abs(), (other.0.y - self.0.y).abs())
    }
}

pub enum EquipSlot {
    Weapon,
    Armor,
}

pub struct Equipment {
    pub weapon: Option<EntityId>,
    pub armor: Option<EntityId>,
}

pub struct Experience {
    pub level: i32,
    pub exp: u64,
    pub next: u64,
    pub base: u64,
}

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

pub struct GivesExperience(pub u64);

pub enum HurtBy {
    Someone(EntityId),
    Starvation,
}

pub struct InflictsDamage {
    pub damage: i32,
}

pub struct InflictsSleep {
    pub sleepiness: i32,
}

pub struct Inventory {
    pub items: Vec<EntityId>,
}

pub struct Item;

pub struct Monster;

pub struct Name(pub String);

pub struct Nutrition(pub i32);

pub struct Player {
    pub auto_run: Option<AutoRun>,
}

pub struct ProvidesHealing {
    pub heal_amount: i32,
}

pub struct Ranged {
    pub range: i32,
}

pub struct RenderOnFloor;

pub struct RenderOnMap;

pub struct Renderable {
    pub sym: GameSym,
    pub fg: Color,
    pub bg: Color,
}

pub struct Stomach {
    pub fullness: i32,
    pub max_fullness: i32,
    pub sub_hp: i32,
}
