#[derive(Clone, Copy)]
pub enum Tile {
    Floor,
    Wall,
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

    pub fn set_tile(&mut self, x: u32, y: u32, tile: Tile) {
        let idx = self.idx(x, y);
        self.tiles[idx] = tile;
    }

    pub fn generate(&mut self) {
        // Top wall.
        for x in 0..self.width {
            self.set_tile(x, 0, Tile::Wall);
        }

        // Side walls.
        for y in 1..self.height - 1 {
            self.set_tile(0, y, Tile::Wall);
            self.set_tile(self.width - 1, y, Tile::Wall);
        }

        // Bottom wall.
        for x in 0..self.width {
            self.set_tile(x, self.height - 1, Tile::Wall);
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (u32, u32, char, [f32; 4])> + '_ {
        self.tiles.iter().enumerate().map(move |(i, tile)| {
            let x = i % self.width as usize;
            let y = i / self.width as usize;
            let (ch, color) = match tile {
                Tile::Floor => ('∙', [0.3, 0.3, 0.3, 1.]),
                Tile::Wall => ('#', [0.7, 0.4, 0.1, 1.]),
            };

            (x as u32, y as u32, ch, color)
        })
    }
}
