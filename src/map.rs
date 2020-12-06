use rand::Rng;
use shipyard::{Get, UniqueView, UniqueViewMut, View, ViewMut, World};

use crate::{
    components::{FieldOfView, PlayerId, Position},
    player::get_player_position,
    rect::Rect,
    RuggleRng,
};
use ruggle::CharGrid;

#[derive(Clone, Copy)]
pub enum Tile {
    Floor,
    Wall,
}

impl Tile {
    pub fn appearance(&self) -> (char, [f32; 4]) {
        match *self {
            Tile::Floor => ('∙', [0.3, 0.3, 0.3, 1.]),
            Tile::Wall => ('#', [0.7, 0.4, 0.1, 1.]),
        }
    }
}

pub struct Map {
    pub width: i32,
    pub height: i32,
    tiles: Vec<Tile>,
    rooms: Vec<Rect>,
}

impl Map {
    pub fn new(width: i32, height: i32) -> Self {
        assert!(width > 0 && height > 0);

        Self {
            width,
            height,
            tiles: vec![Tile::Floor; (width * height) as usize],
            rooms: Vec::new(),
        }
    }

    fn idx(&self, x: i32, y: i32) -> usize {
        y as usize * self.width as usize + x as usize
    }

    pub fn get_tile(&self, x: i32, y: i32) -> &Tile {
        &self.tiles[self.idx(x, y)]
    }

    pub fn set_tile(&mut self, x: i32, y: i32, tile: Tile) {
        let idx = self.idx(x, y);
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

    pub fn iter_bounds(
        &self,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
    ) -> impl Iterator<Item = (i32, i32, Option<(char, [f32; 4])>)> + '_ {
        let xs = if x1 <= x2 { x1..=x2 } else { x2..=x1 };

        xs.flat_map(move |x| {
            let ys = if y1 <= y2 { y1..=y2 } else { y2..=y1 };

            std::iter::repeat(x).zip(ys)
        })
        .map(move |(x, y)| {
            if x < 0 || y < 0 || x >= self.width || y >= self.height {
                (x, y, None)
            } else {
                let (ch, color) = self.get_tile(x, y).appearance();

                (x, y, Some((ch, color)))
            }
        })
    }
}

impl ruggle::ViewableField for Map {
    fn bounds(&self) -> (i32, i32, i32, i32) {
        (0, 0, self.width - 1, self.height - 1)
    }

    fn is_opaque(&self, x: i32, y: i32) -> bool {
        matches!(self.get_tile(x, y), Tile::Wall)
    }
}

pub fn generate_rooms_and_corridors(
    mut map: UniqueViewMut<Map>,
    mut rng: UniqueViewMut<RuggleRng>,
) {
    {
        let w = map.width;
        let h = map.height;
        map.set_rect(&Rect::new(0, 0, w, h), Tile::Wall);
    }

    for _ in 0..30 {
        let w: i32 = rng.0.gen_range(6, 15);
        let h: i32 = rng.0.gen_range(6, 11);
        let x: i32 = rng.0.gen_range(1, map.width - w - 1);
        let y: i32 = rng.0.gen_range(1, map.height - h - 1);
        let new_room = Rect::new(x, y, w, h);

        if !map.rooms.iter().any(|r| new_room.intersects(&r, 1)) {
            map.set_rect(&new_room, Tile::Floor);

            // Connect the new room to the last-added room.
            if !map.rooms.is_empty() {
                let (new_x, new_y) = new_room.center();
                let (last_x, last_y) = map.rooms.last().unwrap().center();

                if rng.0.gen() {
                    map.set_hline(last_x, new_x, last_y, Tile::Floor);
                    map.set_vline(last_y, new_y, new_x, Tile::Floor);
                } else {
                    map.set_vline(last_y, new_y, last_x, Tile::Floor);
                    map.set_hline(last_x, new_x, new_y, Tile::Floor);
                }
            }

            map.rooms.push(new_room);
        }
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
    map: UniqueView<Map>,
    player: UniqueView<PlayerId>,
    mut positions: ViewMut<Position>,
) {
    let (room_center_x, room_center_y) = map.rooms.first().unwrap().center();
    let mut player_pos = (&mut positions).get(player.0);

    player_pos.x = room_center_x;
    player_pos.y = room_center_y;
}

pub fn draw_map(world: &World, grid: &mut CharGrid) {
    world.run(
        |map: UniqueView<Map>,
         player: UniqueView<PlayerId>,
         fov: UniqueView<FieldOfView>,
         positions: View<Position>| {
            let (x, y) = get_player_position(&player, &positions);

            for (tx, ty, tile) in map.iter_bounds(x - 40, y - 18, x + 39, y + 17) {
                if let Some((ch, color)) = tile {
                    let color = if fov.0.contains(&(tx, ty)) {
                        color
                    } else {
                        let v = (0.3 * color[0] + 0.59 * color[1] + 0.11 * color[2]) / 2.;
                        [v, v, v, color[3]]
                    };

                    grid.put_color([tx - x + 40, ty - y + 18], Some(color), None, ch);
                }
            }
        },
    );
}
