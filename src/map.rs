use rand::{seq::IteratorRandom, Rng, SeedableRng};
use rand_xoshiro::Xoshiro128PlusPlus as GameRng;
use serde::{Deserialize, Serialize};
use shipyard::{EntityId, Get, UniqueView, UniqueViewMut, View, ViewMut, World};
use std::{collections::HashMap, hash::Hasher};
use wyhash::WyHash;

use crate::{
    bitgrid::BitGrid,
    components::{Coord, Experience, FieldOfView, Item, Monster, Name, Player},
    experience::Difficulty,
    gamesym::GameSym,
    magicnum,
    player::PlayerId,
    GameSeed,
};
use ruggrogue::util::Color;

#[derive(Clone, Copy, Eq, PartialEq, Deserialize, Serialize)]
pub enum Tile {
    #[serde(rename = "F")]
    Floor,
    #[serde(rename = "W")]
    Wall,
    #[serde(rename = "D")]
    DownStairs,
}

impl std::fmt::Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Tile::Floor => "Floor",
                Tile::Wall => "Wall",
                Tile::DownStairs => "Down Stairs",
            }
        )
    }
}

#[derive(Clone, Copy, Deserialize, Serialize)]
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

    pub fn iter_xy(&self) -> impl Iterator<Item = (i32, i32)> + '_ {
        (self.y1..=self.y2)
            .flat_map(move |y| std::iter::repeat(y).zip(self.x1..=self.x2))
            .map(move |(y, x)| (x, y))
    }
}

#[derive(Deserialize, Serialize)]
pub struct Map {
    pub depth: i32,
    pub width: i32,
    pub height: i32,
    #[serde(with = "crate::saveload::run_length_encoded")]
    tiles: Vec<Tile>,
    pub rooms: Vec<Rect>,
    pub seen: BitGrid,

    // (x, y) -> (blocking_entity_count, entities_here)
    #[serde(skip)]
    tile_entities: HashMap<(i32, i32), (i32, Vec<EntityId>)>,

    // zero-length non-zero-capacity vectors for reuse in tile_entities
    #[serde(skip)]
    empty_entity_vecs: Vec<Vec<EntityId>>,
}

impl Map {
    pub fn new(width: i32, height: i32) -> Self {
        assert!(width > 0 && height > 0);

        Self {
            depth: 0,
            width,
            height,
            tiles: vec![Tile::Floor; (width * height) as usize],
            rooms: Vec::new(),
            seen: BitGrid::new(width, height),
            tile_entities: HashMap::new(),
            empty_entity_vecs: Vec::new(),
        }
    }

    pub fn replace(&mut self, replacement: Self) {
        self.depth = replacement.depth;
        self.width = replacement.width;
        self.height = replacement.height;
        self.tiles = replacement.tiles;
        self.rooms = replacement.rooms;
        self.seen = replacement.seen;
        self.tile_entities = replacement.tile_entities;
        self.empty_entity_vecs = replacement.empty_entity_vecs;
    }

    pub fn clear(&mut self) {
        self.tiles.clear();
        self.tiles
            .resize((self.width * self.height) as usize, Tile::Floor);
        self.rooms.clear();
        self.seen.zero_out_bits();
        self.tile_entities.clear();
    }

    #[inline]
    fn index(&self, x: i32, y: i32) -> usize {
        (y * self.width + x) as usize
    }

    #[inline]
    pub fn get_tile(&self, x: i32, y: i32) -> &Tile {
        &self.tiles[self.index(x, y)]
    }

    #[inline]
    pub fn set_tile(&mut self, x: i32, y: i32, tile: Tile) {
        let idx = self.index(x, y);
        self.tiles[idx] = tile;
    }

    pub fn set_rect(&mut self, rect: &Rect, tile: Tile) {
        assert!(rect.x1 >= 0 && rect.x2 < self.width);
        assert!(rect.y1 >= 0 && rect.y2 < self.height);

        for y in rect.y1..=rect.y2 {
            for x in rect.x1..=rect.x2 {
                self.set_tile(x, y, tile);
            }
        }
    }

    pub fn set_hline(&mut self, x1: i32, x2: i32, y: i32, tile: Tile) {
        assert!(x1 >= 0 && x1 < self.width);
        assert!(x2 >= 0 && x2 < self.width);
        assert!(y >= 0 && y < self.height);

        let (x1, x2) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };

        for x in x1..=x2 {
            self.set_tile(x, y, tile);
        }
    }

    pub fn set_vline(&mut self, y1: i32, y2: i32, x: i32, tile: Tile) {
        assert!(y1 >= 0 && y1 < self.height);
        assert!(y2 >= 0 && y2 < self.height);
        assert!(x >= 0 && x < self.width);

        let (y1, y2) = if y1 <= y2 { (y1, y2) } else { (y2, y1) };

        for y in y1..=y2 {
            self.set_tile(x, y, tile);
        }
    }

    #[inline]
    pub fn wall_or_oob(&self, x: i32, y: i32) -> bool {
        x < 0
            || y < 0
            || x >= self.width
            || y >= self.height
            || matches!(self.get_tile(x, y), Tile::Wall)
    }

    #[allow(clippy::many_single_char_names)]
    fn wall_sym(&self, x: i32, y: i32) -> GameSym {
        let n = self.wall_or_oob(x, y - 1);
        let s = self.wall_or_oob(x, y + 1);
        let e = self.wall_or_oob(x + 1, y);
        let w = self.wall_or_oob(x - 1, y);
        let ne = self.wall_or_oob(x + 1, y - 1);
        let nw = self.wall_or_oob(x - 1, y - 1);
        let se = self.wall_or_oob(x + 1, y + 1);
        let sw = self.wall_or_oob(x - 1, y + 1);

        // Extend wall stems in a direction if it has a wall,
        // and at least one of its cardinal/diagonal adjacent tiles is not a wall.
        let mut mask: u8 = 0;

        if n && (!ne || !nw || !e || !w) {
            mask += 1;
        }
        if s && (!se || !sw || !e || !w) {
            mask += 2;
        }
        if w && (!nw || !sw || !n || !s) {
            mask += 4;
        }
        if e && (!ne || !se || !n || !s) {
            mask += 8;
        }

        match mask {
            0 => GameSym::WallPillar,
            1 => GameSym::WallN,
            2 => GameSym::WallS,
            3 => GameSym::WallNs,
            4 => GameSym::WallW,
            5 => GameSym::WallNw,
            6 => GameSym::WallSw,
            7 => GameSym::WallNsw,
            8 => GameSym::WallE,
            9 => GameSym::WallNe,
            10 => GameSym::WallEs,
            11 => GameSym::WallNes,
            12 => GameSym::WallEw,
            13 => GameSym::WallNew,
            14 => GameSym::WallEsw,
            15 => GameSym::WallNesw,
            _ => GameSym::WallOther,
        }
    }

    pub fn iter_bounds(
        &self,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
    ) -> impl Iterator<Item = (i32, i32, Option<(GameSym, Color)>)> + '_ {
        let ys = if y1 <= y2 { y1..=y2 } else { y2..=y1 };

        ys.flat_map(move |y| {
            let xs = if x1 <= x2 { x1..=x2 } else { x2..=x1 };

            std::iter::repeat(y).zip(xs)
        })
        .map(move |(y, x)| {
            if self.seen.get_bit(x, y) {
                (
                    x,
                    y,
                    Some(match self.get_tile(x, y) {
                        Tile::Floor => (
                            GameSym::Floor,
                            Color {
                                r: 102,
                                g: 102,
                                b: 102,
                            },
                        ),
                        Tile::Wall => (
                            self.wall_sym(x, y),
                            Color {
                                r: 134,
                                g: 77,
                                b: 20,
                            },
                        ),
                        Tile::DownStairs => (
                            GameSym::DownStairs,
                            Color {
                                r: 255,
                                g: 255,
                                b: 0,
                            },
                        ),
                    }),
                )
            } else {
                (x, y, None)
            }
        })
    }

    pub fn place_entity(&mut self, entity: EntityId, pos: (i32, i32), blocks: bool) {
        if let Some((block_count, entities_here)) = self.tile_entities.get_mut(&pos) {
            if blocks {
                *block_count += 1;
            }
            entities_here.push(entity);
        } else {
            let mut entities_here = self.empty_entity_vecs.pop().unwrap_or_default();

            entities_here.push(entity);
            self.tile_entities
                .insert(pos, (if blocks { 1 } else { 0 }, entities_here));
        }
    }

    pub fn remove_entity(&mut self, entity: EntityId, pos: (i32, i32), blocks: bool) {
        if self.tile_entities.get(&pos).unwrap().1.len() == 1 {
            let (_, mut entities_here) = self.tile_entities.remove(&pos).unwrap();

            assert!(entities_here[0] == entity);
            entities_here.clear();
            self.empty_entity_vecs.push(entities_here);
        } else {
            let (block_count, entities_here) = self.tile_entities.get_mut(&pos).unwrap();
            let idx = entities_here.iter().position(|e| *e == entity).unwrap();

            entities_here.remove(idx);
            if blocks {
                *block_count -= 1;
            }
        }
    }

    pub fn move_entity(
        &mut self,
        entity: EntityId,
        old_pos: (i32, i32),
        new_pos: (i32, i32),
        blocks: bool,
    ) {
        if self.tile_entities.get(&old_pos).unwrap().1.len() == 1
            && !self.tile_entities.contains_key(&new_pos)
        {
            let data = self.tile_entities.remove(&old_pos).unwrap();

            assert!(data.1[0] == entity);
            self.tile_entities.insert(new_pos, data);
        } else {
            self.remove_entity(entity, old_pos, blocks);
            self.place_entity(entity, new_pos, blocks);
        }
    }

    pub fn iter_entities_at(&self, x: i32, y: i32) -> impl Iterator<Item = EntityId> + '_ {
        self.tile_entities
            .get(&(x, y))
            .map(|(_, es)| es.iter().copied())
            .into_iter()
            .flatten()
    }

    /// Describe a position on the map from the perspective of the player.
    ///
    /// Returns a description string and a bool that is true if the position is being recalled from
    /// the seen memory of the player.
    pub fn describe_pos(
        &self,
        world: &World,
        x: i32,
        y: i32,
        focus_on_target: bool,
        omit_player: bool,
        omit_boring_tiles: bool,
    ) -> (String, bool) {
        if self.seen.get_bit(x, y) {
            let in_player_fov = {
                let player_id = world.borrow::<UniqueView<PlayerId>>();
                let fovs = world.borrow::<View<FieldOfView>>();
                fovs.get(player_id.0).get((x, y))
            };

            if in_player_fov {
                let names = world.borrow::<View<Name>>();
                let mut desc_vec = Vec::new();

                if let Some(monster) = self
                    .iter_entities_at(x, y)
                    .find(|id| world.borrow::<View<Monster>>().contains(*id))
                {
                    desc_vec.push(names.get(monster).0.clone());
                }

                if !omit_player {
                    if let Some(player) = self
                        .iter_entities_at(x, y)
                        .find(|id| world.borrow::<View<Player>>().contains(*id))
                    {
                        desc_vec.push(names.get(player).0.clone());
                    }
                }

                if desc_vec.is_empty() || !focus_on_target {
                    let mut items_at_pos = self
                        .iter_entities_at(x, y)
                        .filter(|id| world.borrow::<View<Item>>().contains(*id));

                    if let Some(item) = items_at_pos.next() {
                        let more_items_count = items_at_pos.count();

                        if more_items_count > 0 {
                            desc_vec.push(format!("{} items", more_items_count + 1));
                        } else {
                            desc_vec.push(names.get(item).0.clone());
                        }
                    }

                    let tile = self.get_tile(x, y);

                    if desc_vec.is_empty()
                        || !omit_boring_tiles
                        || !matches!(tile, Tile::Floor | Tile::Wall)
                    {
                        desc_vec.push(tile.to_string());
                    }
                }

                (desc_vec.join(", "), false)
            } else {
                (self.get_tile(x, y).to_string(), true)
            }
        } else {
            ("nothing".to_string(), true)
        }
    }
}

impl ruggrogue::BoundedMap for Map {
    fn bounds(&self) -> (i32, i32, i32, i32) {
        (0, 0, self.width - 1, self.height - 1)
    }
}

impl ruggrogue::ViewableField for Map {
    fn is_opaque(&self, x: i32, y: i32) -> bool {
        matches!(self.get_tile(x, y), Tile::Wall)
    }
}

impl ruggrogue::PathableMap for Map {
    fn is_blocked(&self, x: i32, y: i32) -> bool {
        matches!(self.get_tile(x, y), &Tile::Wall)
            || self
                .tile_entities
                .get(&(x, y))
                .map_or(false, |(block_count, _)| *block_count > 0)
    }
}

/// Returns the position to spawn the victory item if the game has progressed far enough.
pub fn generate_rooms_and_corridors(
    difficulty: UniqueView<Difficulty>,
    game_seed: UniqueView<GameSeed>,
    mut map: UniqueViewMut<Map>,
    exps: View<Experience>,
) -> Option<(i32, i32)> {
    {
        let w = map.width;
        let h = map.height;
        map.set_rect(&Rect::new(0, 0, w, h), Tile::Wall);
    }

    let mut rng = {
        let mut hasher = WyHash::with_seed(magicnum::GENERATE_ROOMS_AND_CORRIDORS);
        hasher.write_u64(game_seed.0);
        hasher.write_i32(map.depth);
        GameRng::seed_from_u64(hasher.finish())
    };

    for _ in 0..30 {
        let w: i32 = rng.gen_range(6i32..15i32);
        let h: i32 = rng.gen_range(6i32..11i32);
        let x: i32 = rng.gen_range(1i32..map.width - w - 1);
        let y: i32 = rng.gen_range(1i32..map.height - h - 1);
        let new_room = Rect::new(x, y, w, h);

        if !map.rooms.iter().any(|r| new_room.intersects(r, 1)) {
            map.set_rect(&new_room, Tile::Floor);
            map.rooms.push(new_room);
        }
    }

    let connect_rooms = |map: &mut UniqueViewMut<Map>, r1: usize, r2: usize, h_then_v: bool| {
        let (r1x, r1y) = map.rooms[r1].center();
        let (r2x, r2y) = map.rooms[r2].center();
        if h_then_v {
            map.set_hline(r2x, r1x, r2y, Tile::Floor);
            map.set_vline(r2y, r1y, r1x, Tile::Floor);
        } else {
            map.set_vline(r2y, r1y, r2x, Tile::Floor);
            map.set_hline(r2x, r1x, r1y, Tile::Floor);
        }
    };
    let mut connected: Vec<usize> = Vec::new();
    let mut disconnected: Vec<usize> = Vec::new();

    // Consider the first room as the start of connectedness.
    connected.push(0);

    // All other rooms start disconnected.
    for i in 1..map.rooms.len() {
        disconnected.push(i);
    }

    // Connect all the disconnected rooms to the connected rooms based on closeness.
    while !disconnected.is_empty() {
        // Find the closest match between connected and disconnected.
        let (closest_connected, closest_disconnected) = connected
            .iter()
            .enumerate()
            .flat_map(|c| std::iter::repeat(c).zip(disconnected.iter().enumerate()))
            .min_by_key(|&((_, &croom), (_, &droom))| {
                let ccenter = map.rooms[croom].center();
                let dcenter = map.rooms[droom].center();
                (ccenter.0 - dcenter.0).abs() + (ccenter.1 - dcenter.1).abs()
            })
            .map(|((ci, _), (di, _))| (ci, di))
            .unwrap();

        // Connect the closest connected and disconnected rooms together.
        connect_rooms(
            &mut map,
            connected[closest_connected],
            disconnected[closest_disconnected],
            rng.gen::<bool>(),
        );

        // Transfer newly-connected room index from disconnected to connected.
        connected.push(disconnected.remove(closest_disconnected));
    }

    // Decide corridor styles to connect random extra rooms.
    let mut extra_corridors = [false; 3];
    for extra_corridor in extra_corridors.iter_mut() {
        *extra_corridor = rng.gen::<bool>();
    }

    // Connect random extra rooms.
    for (extra_rooms, extra_corridor) in (0..map.rooms.len())
        .choose_multiple(&mut rng, extra_corridors.len() * 2)
        .chunks_exact(2)
        .zip(&extra_corridors)
    {
        connect_rooms(&mut map, extra_rooms[0], extra_rooms[1], *extra_corridor);
    }

    if let Some(last_room) = map.rooms.last() {
        let (center_x, center_y) = last_room.center();

        if exps.get(difficulty.id).level < 25 {
            map.set_tile(center_x, center_y, Tile::DownStairs);
            None
        } else {
            Some((center_x, center_y))
        }
    } else {
        None
    }
}

pub fn place_player_in_first_room(
    mut map: UniqueViewMut<Map>,
    player_id: UniqueView<PlayerId>,
    mut coords: ViewMut<Coord>,
) {
    let room_center = map.rooms.first().unwrap().center();
    let mut player_coord = (&mut coords).get(player_id.0);

    map.place_entity(player_id.0, room_center, false);
    player_coord.0 = room_center.into();
}
