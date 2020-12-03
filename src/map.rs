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
    width: u32,
    height: u32,
    tiles: Vec<Tile>,
}

impl Map {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            tiles: vec![Tile::Floor; (width * height) as usize],
        }
    }

    fn idx(&self, x: u32, y: u32) -> usize {
        y as usize * self.width as usize + x as usize
    }

    pub fn get_tile(&self, x: u32, y: u32) -> &Tile {
        &self.tiles[self.idx(x, y)]
    }

    pub fn set_tile(&mut self, x: u32, y: u32, tile: Tile) {
        let idx = self.idx(x, y);
        self.tiles[idx] = tile;
    }

    pub fn generate(&mut self) {
        // Border walls.
        for x in 0..self.width {
            self.set_tile(x, 0, Tile::Wall);
        }
        for y in 1..self.height - 1 {
            self.set_tile(0, y, Tile::Wall);
            self.set_tile(self.width - 1, y, Tile::Wall);
        }
        for x in 0..self.width {
            self.set_tile(x, self.height - 1, Tile::Wall);
        }

        // Pillars next to center.
        self.set_tile(39, 18, Tile::Wall);
        self.set_tile(41, 18, Tile::Wall);

        // Oblique pillars down-right.
        self.set_tile(42, 20, Tile::Wall);
        self.set_tile(44, 20, Tile::Wall);
        self.set_tile(45, 22, Tile::Wall);
        self.set_tile(47, 22, Tile::Wall);
        self.set_tile(48, 24, Tile::Wall);
        self.set_tile(50, 24, Tile::Wall);

        // Oblique pillars down-left.
        self.set_tile(36, 20, Tile::Wall);
        self.set_tile(38, 20, Tile::Wall);
        self.set_tile(33, 22, Tile::Wall);
        self.set_tile(35, 22, Tile::Wall);
        self.set_tile(30, 24, Tile::Wall);
        self.set_tile(32, 24, Tile::Wall);

        // Oblique pillars up-right.
        self.set_tile(42, 16, Tile::Wall);
        self.set_tile(44, 16, Tile::Wall);
        self.set_tile(45, 14, Tile::Wall);
        self.set_tile(47, 14, Tile::Wall);
        self.set_tile(48, 12, Tile::Wall);
        self.set_tile(50, 12, Tile::Wall);

        // Oblique pillars up-left.
        self.set_tile(36, 16, Tile::Wall);
        self.set_tile(38, 16, Tile::Wall);
        self.set_tile(33, 14, Tile::Wall);
        self.set_tile(35, 14, Tile::Wall);
        self.set_tile(30, 12, Tile::Wall);
        self.set_tile(32, 12, Tile::Wall);

        // Intersecting corridors.
        for y in 2..6 {
            for x in 30..40 {
                self.set_tile(x, y, Tile::Wall);
            }
            for x in 41..51 {
                self.set_tile(x, y, Tile::Wall);
            }
        }
        for y in 7..11 {
            for x in 30..40 {
                self.set_tile(x, y, Tile::Wall);
            }
            for x in 41..51 {
                self.set_tile(x, y, Tile::Wall);
            }
        }

        // Room with corners.
        for x in 30..40 {
            self.set_tile(x, 26, Tile::Wall);
        }
        for y in 27..30 {
            self.set_tile(30, y, Tile::Wall);
            self.set_tile(39, y, Tile::Wall);
        }
        self.set_tile(30, 30, Tile::Wall);
        for y in 31..34 {
            self.set_tile(30, y, Tile::Wall);
            self.set_tile(39, y, Tile::Wall);
        }
        for x in 31..39 {
            self.set_tile(x, 33, Tile::Wall);
        }

        // Room without corners.
        for x in 42..50 {
            self.set_tile(x, 26, Tile::Wall);
        }
        for y in 27..30 {
            self.set_tile(41, y, Tile::Wall);
            self.set_tile(50, y, Tile::Wall);
        }
        self.set_tile(50, 30, Tile::Wall);
        for y in 31..33 {
            self.set_tile(41, y, Tile::Wall);
            self.set_tile(50, y, Tile::Wall);
        }
        for x in 42..50 {
            self.set_tile(x, 33, Tile::Wall);
        }

        // 25% pillars.
        for y in 1..=8 {
            for x in 1..=7 {
                self.set_tile(x * 2, y * 2, Tile::Wall);
            }
        }

        // Diagonal grid.
        for y in 2..17 {
            for x in 16..29 {
                if (x + y) % 2 == 0 {
                    self.set_tile(x, y, Tile::Wall);
                }
            }
        }

        // 2-wide diagonal walls.
        for i in 1..8 {
            self.set_tile(15 - i, 26 - i, Tile::Wall);
            self.set_tile(15 - (i + 1), 26 - i, Tile::Wall);
            self.set_tile(15 + i, 26 - i, Tile::Wall);
            self.set_tile(15 + (i + 1), 26 - i, Tile::Wall);
            self.set_tile(15 - i, 26 + i, Tile::Wall);
            self.set_tile(15 - (i + 1), 26 + i, Tile::Wall);
            self.set_tile(15 + i, 26 + i, Tile::Wall);
            self.set_tile(15 + (i + 1), 26 + i, Tile::Wall);
        }

        // Oblique intersecting corridors.
        for y in 2..15 {
            for x in 53..78 {
                self.set_tile(x, y, Tile::Wall);
            }
        }
        for y in 2..15 {
            self.set_tile(65, y, Tile::Floor);
        }
        for x in 53..78 {
            self.set_tile(x, 8, Tile::Floor);
        }
        for i in 0..5 {
            let dx = i * 2 + 1;
            self.set_tile(65 - dx, 8 - (i + 1), Tile::Floor);
            self.set_tile(65 - (dx + 1), 8 - (i + 1), Tile::Floor);
            self.set_tile(65 + dx, 8 - (i + 1), Tile::Floor);
            self.set_tile(65 + (dx + 1), 8 - (i + 1), Tile::Floor);
            self.set_tile(65 - dx, 8 + (i + 1), Tile::Floor);
            self.set_tile(65 - (dx + 1), 8 + (i + 1), Tile::Floor);
            self.set_tile(65 + dx, 8 + (i + 1), Tile::Floor);
            self.set_tile(65 + (dx + 1), 8 + (i + 1), Tile::Floor);
        }

        // A pillar in an otherwise open space.
        self.set_tile(65, 24, Tile::Wall);
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
            if x < 0 || y < 0 {
                (x, y, None)
            } else {
                let ux = x as u32;
                let uy = y as u32;
                if ux >= self.width || uy >= self.height {
                    (x, y, None)
                } else {
                    let (ch, color) = self.get_tile(ux, uy).appearance();

                    (x, y, Some((ch, color)))
                }
            }
        })
    }
}

impl ruggle::ViewableField for Map {
    fn bounds(&self) -> (i32, i32, i32, i32) {
        (0, 0, self.width as i32 - 1, self.height as i32 - 1)
    }

    fn is_opaque(&self, x: i32, y: i32) -> bool {
        matches!(self.get_tile(x as u32, y as u32), Tile::Wall)
    }
}
