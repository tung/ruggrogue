use rand::{seq::IteratorRandom, Rng, SeedableRng};
use rand_pcg::Pcg32;
use shipyard::{EntityId, Get, UniqueView, UniqueViewMut, ViewMut};
use std::{collections::HashMap, hash::Hasher};
use wyhash::WyHash;

use crate::{
    bitgrid::BitGrid, chunked, components::Coord, gamesym::GameSym, magicnum, player::PlayerId,
    GameSeed,
};
use ruggle::util::Color;

#[derive(Clone, Copy)]
pub enum Tile {
    Floor,
    Wall,
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

#[derive(Clone, Copy)]
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

pub struct Map {
    pub depth: i32,
    pub width: i32,
    pub height: i32,
    tiles: Vec<Tile>,
    pub rooms: Vec<Rect>,
    pub seen: BitGrid,
    // (x, y) -> (blocking_entity_count, entities_here)
    tile_entities: HashMap<(i32, i32), (i32, Vec<EntityId>)>,
    // zero-length non-zero-capacity vectors for reuse in tile_entities
    empty_entity_vecs: Vec<Vec<EntityId>>,
}

impl Map {
    pub fn new(width: i32, height: i32) -> Self {
        assert!(width > 0 && height > 0);

        let padded_width = (width + chunked::CHUNK_TILE_WIDTH - 1) / chunked::CHUNK_TILE_WIDTH
            * chunked::CHUNK_TILE_WIDTH;
        let padded_height = (height + chunked::CHUNK_TILE_HEIGHT - 1) / chunked::CHUNK_TILE_HEIGHT
            * chunked::CHUNK_TILE_HEIGHT;

        Self {
            depth: 0,
            width,
            height,
            tiles: vec![Tile::Floor; (padded_width * padded_height) as usize],
            rooms: Vec::new(),
            seen: BitGrid::new(width, height),
            tile_entities: HashMap::new(),
            empty_entity_vecs: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        let padded_width = (self.width + chunked::CHUNK_TILE_WIDTH - 1) / chunked::CHUNK_TILE_WIDTH
            * chunked::CHUNK_TILE_WIDTH;
        let padded_height = (self.height + chunked::CHUNK_TILE_HEIGHT - 1)
            / chunked::CHUNK_TILE_HEIGHT
            * chunked::CHUNK_TILE_HEIGHT;

        self.tiles.clear();
        self.tiles
            .resize((padded_width * padded_height) as usize, Tile::Floor);
        self.rooms.clear();
        self.seen.zero_out_bits();
        self.tile_entities.clear();
    }

    #[inline]
    fn index(&self, x: i32, y: i32) -> usize {
        let chunks_across =
            (self.width + chunked::CHUNK_TILE_WIDTH - 1) / chunked::CHUNK_TILE_WIDTH;
        let chunk_x = x / chunked::CHUNK_TILE_WIDTH;
        let chunk_y = y / chunked::CHUNK_TILE_HEIGHT;
        let sub_x = x % chunked::CHUNK_TILE_WIDTH;
        let sub_y = y % chunked::CHUNK_TILE_HEIGHT;

        ((chunk_y * chunks_across + chunk_x)
            * chunked::CHUNK_TILE_WIDTH
            * chunked::CHUNK_TILE_HEIGHT
            + sub_y * chunked::CHUNK_TILE_WIDTH
            + sub_x) as usize
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
}

impl ruggle::BoundedMap for Map {
    fn bounds(&self) -> (i32, i32, i32, i32) {
        (0, 0, self.width - 1, self.height - 1)
    }
}

impl ruggle::ViewableField for Map {
    fn is_opaque(&self, x: i32, y: i32) -> bool {
        matches!(self.get_tile(x, y), Tile::Wall)
    }
}

impl ruggle::PathableMap for Map {
    fn is_blocked(&self, x: i32, y: i32) -> bool {
        matches!(self.get_tile(x, y), &Tile::Wall)
            || self
                .tile_entities
                .get(&(x, y))
                .map_or(false, |(block_count, _)| *block_count > 0)
    }
}

pub fn generate_rooms_and_corridors(game_seed: UniqueView<GameSeed>, mut map: UniqueViewMut<Map>) {
    {
        let w = map.width;
        let h = map.height;
        map.set_rect(&Rect::new(0, 0, w, h), Tile::Wall);
    }

    let mut rng = {
        let mut hasher = WyHash::with_seed(magicnum::GENERATE_ROOMS_AND_CORRIDORS);
        hasher.write_u64(game_seed.0);
        hasher.write_i32(map.depth);
        Pcg32::seed_from_u64(hasher.finish())
    };

    for _ in 0..30 {
        let w: i32 = rng.gen_range(6, 15);
        let h: i32 = rng.gen_range(6, 11);
        let x: i32 = rng.gen_range(1, map.width - w - 1);
        let y: i32 = rng.gen_range(1, map.height - h - 1);
        let new_room = Rect::new(x, y, w, h);

        if !map.rooms.iter().any(|r| new_room.intersects(&r, 1)) {
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
            rng.gen(),
        );

        // Transfer newly-connected room index from disconnected to connected.
        connected.push(disconnected.remove(closest_disconnected));
    }

    // Decide corridor styles to connect random extra rooms.
    let mut extra_corridors = [false; 3];
    for extra_corridor in extra_corridors.iter_mut() {
        *extra_corridor = rng.gen();
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
        map.set_tile(center_x, center_y, Tile::DownStairs);
    }
}

#[allow(dead_code)]
pub fn generate_test_pattern(mut map: UniqueViewMut<Map>) {
    let width = map.width;
    let height = map.height;

    // Border walls.
    for x in 0..width {
        map.set_tile(x, 0, Tile::Wall);
    }
    for y in 1..height - 1 {
        map.set_tile(0, y, Tile::Wall);
        map.set_tile(width - 1, y, Tile::Wall);
    }
    for x in 0..width {
        map.set_tile(x, height - 1, Tile::Wall);
    }

    // Pillars next to center.
    map.set_tile(39, 18, Tile::Wall);
    map.set_tile(41, 18, Tile::Wall);

    // Oblique pillars down-right.
    map.set_tile(42, 20, Tile::Wall);
    map.set_tile(44, 20, Tile::Wall);
    map.set_tile(45, 22, Tile::Wall);
    map.set_tile(47, 22, Tile::Wall);
    map.set_tile(48, 24, Tile::Wall);
    map.set_tile(50, 24, Tile::Wall);

    // Oblique pillars down-left.
    map.set_tile(36, 20, Tile::Wall);
    map.set_tile(38, 20, Tile::Wall);
    map.set_tile(33, 22, Tile::Wall);
    map.set_tile(35, 22, Tile::Wall);
    map.set_tile(30, 24, Tile::Wall);
    map.set_tile(32, 24, Tile::Wall);

    // Oblique pillars up-right.
    map.set_tile(42, 16, Tile::Wall);
    map.set_tile(44, 16, Tile::Wall);
    map.set_tile(45, 14, Tile::Wall);
    map.set_tile(47, 14, Tile::Wall);
    map.set_tile(48, 12, Tile::Wall);
    map.set_tile(50, 12, Tile::Wall);

    // Oblique pillars up-left.
    map.set_tile(36, 16, Tile::Wall);
    map.set_tile(38, 16, Tile::Wall);
    map.set_tile(33, 14, Tile::Wall);
    map.set_tile(35, 14, Tile::Wall);
    map.set_tile(30, 12, Tile::Wall);
    map.set_tile(32, 12, Tile::Wall);

    // Intersecting corridors.
    for y in 2..6 {
        for x in 30..40 {
            map.set_tile(x, y, Tile::Wall);
        }
        for x in 41..51 {
            map.set_tile(x, y, Tile::Wall);
        }
    }
    for y in 7..11 {
        for x in 30..40 {
            map.set_tile(x, y, Tile::Wall);
        }
        for x in 41..51 {
            map.set_tile(x, y, Tile::Wall);
        }
    }

    // Room with corners.
    for x in 30..40 {
        map.set_tile(x, 26, Tile::Wall);
    }
    for y in 27..30 {
        map.set_tile(30, y, Tile::Wall);
        map.set_tile(39, y, Tile::Wall);
    }
    map.set_tile(30, 30, Tile::Wall);
    for y in 31..34 {
        map.set_tile(30, y, Tile::Wall);
        map.set_tile(39, y, Tile::Wall);
    }
    for x in 31..39 {
        map.set_tile(x, 33, Tile::Wall);
    }

    // Room without corners.
    for x in 42..50 {
        map.set_tile(x, 26, Tile::Wall);
    }
    for y in 27..30 {
        map.set_tile(41, y, Tile::Wall);
        map.set_tile(50, y, Tile::Wall);
    }
    map.set_tile(50, 30, Tile::Wall);
    for y in 31..33 {
        map.set_tile(41, y, Tile::Wall);
        map.set_tile(50, y, Tile::Wall);
    }
    for x in 42..50 {
        map.set_tile(x, 33, Tile::Wall);
    }

    // 25% pillars.
    for y in 1..=8 {
        for x in 1..=7 {
            map.set_tile(x * 2, y * 2, Tile::Wall);
        }
    }

    // Diagonal grid.
    for y in 2..17 {
        for x in 16..29 {
            if (x + y) % 2 == 0 {
                map.set_tile(x, y, Tile::Wall);
            }
        }
    }

    // 2-wide diagonal walls.
    for i in 1..8 {
        map.set_tile(15 - i, 26 - i, Tile::Wall);
        map.set_tile(15 - (i + 1), 26 - i, Tile::Wall);
        map.set_tile(15 + i, 26 - i, Tile::Wall);
        map.set_tile(15 + (i + 1), 26 - i, Tile::Wall);
        map.set_tile(15 - i, 26 + i, Tile::Wall);
        map.set_tile(15 - (i + 1), 26 + i, Tile::Wall);
        map.set_tile(15 + i, 26 + i, Tile::Wall);
        map.set_tile(15 + (i + 1), 26 + i, Tile::Wall);
    }

    // Oblique intersecting corridors.
    for y in 2..15 {
        for x in 53..78 {
            map.set_tile(x, y, Tile::Wall);
        }
    }
    for y in 2..15 {
        map.set_tile(65, y, Tile::Floor);
    }
    for x in 53..78 {
        map.set_tile(x, 8, Tile::Floor);
    }
    for i in 0..5 {
        let dx = i * 2 + 1;
        map.set_tile(65 - dx, 8 - (i + 1), Tile::Floor);
        map.set_tile(65 - (dx + 1), 8 - (i + 1), Tile::Floor);
        map.set_tile(65 + dx, 8 - (i + 1), Tile::Floor);
        map.set_tile(65 + (dx + 1), 8 - (i + 1), Tile::Floor);
        map.set_tile(65 - dx, 8 + (i + 1), Tile::Floor);
        map.set_tile(65 - (dx + 1), 8 + (i + 1), Tile::Floor);
        map.set_tile(65 + dx, 8 + (i + 1), Tile::Floor);
        map.set_tile(65 + (dx + 1), 8 + (i + 1), Tile::Floor);
    }

    // A pillar in an otherwise open space.
    map.set_tile(65, 24, Tile::Wall);
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
