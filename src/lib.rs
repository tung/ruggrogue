use graphics::character::CharacterCache;
use graphics::types::Color;
use graphics::{Context, Graphics};

type Position = [u32; 2];
type Size = [u32; 2];

pub struct CharGrid {
    size: Size,
    chars: Vec<char>,
    fg: Vec<Color>,
    bg: Vec<Color>,
}

impl CharGrid {
    pub fn new(size: Size) -> CharGrid {
        let [width, height] = size;

        assert_ne!(0, width);
        assert_ne!(0, height);

        let vec_size = (width * height) as usize;

        CharGrid {
            size,
            chars: vec![' '; vec_size],
            fg: vec![[1.; 4]; vec_size],
            bg: vec![[0., 0., 0., 1.]; vec_size],
        }
    }

    pub fn clear(&mut self) {
        for e in self.chars.iter_mut() {
            *e = ' ';
        }

        for e in self.fg.iter_mut() {
            *e = [1.; 4];
        }

        for e in self.bg.iter_mut() {
            *e = [0., 0., 0., 1.];
        }
    }

    pub fn put(&mut self, [x, y]: Position, c: char) {
        if x >= self.size[0] || y >= self.size[1] {
            return;
        }

        let w = self.size[0];

        self.chars[(y * w + x) as usize] = c;
    }

    pub fn print(&mut self, [x, y]: Position, s: &str) {
        let width = self.size[0];

        s.char_indices()
            .take_while(|(i, _)| x + (*i as u32) < width)
            .for_each(|(i, c)| self.put([x + i as u32, y], c));
    }

    pub fn draw<G, C>(&self, font_size: u32, cache: &mut C, c: &Context, g: &mut G)
    where
        G: Graphics,
        C: CharacterCache<Texture = G::Texture>,
        C::Error: std::fmt::Debug,
    {
        use graphics::{Image, Rectangle, Transformed};

        let char_image = Image::new();
        let char_width = cache.width(font_size, "W").unwrap();
        let mut char_bg = Rectangle::new([0., 0., 0., 1.]);

        // Draw default background color.
        char_bg.draw(
            [
                0.,
                0.,
                self.size[0] as f64 * char_width,
                (self.size[1] * font_size) as f64,
            ],
            &c.draw_state,
            c.transform,
            g,
        );

        for y in 0..self.size[1] {
            for x in 0..self.size[0] {
                let index = (y * self.size[0] + x) as usize;
                let px = x as f64 * char_width;
                let py = (y * font_size) as f64;

                // Draw cell background color if it differs from the default.
                if self.bg[index][0] > f32::EPSILON
                    || self.bg[index][1] > f32::EPSILON
                    || self.bg[index][2] > f32::EPSILON
                    || 1. - self.bg[index][3] > f32::EPSILON
                {
                    char_bg.color = self.bg[index];
                    char_bg.draw(
                        [px, py, char_width, font_size as f64],
                        &c.draw_state,
                        c.transform,
                        g,
                    );
                }

                // Draw text character.
                if let Ok(char_glyph) = cache.character(font_size, self.chars[index]) {
                    let char_x = px + char_glyph.left();
                    let char_y = py - char_glyph.top();
                    let char_image = char_image.color(self.fg[index]).src_rect([
                        char_glyph.atlas_offset[0],
                        char_glyph.atlas_offset[1],
                        char_glyph.atlas_size[0],
                        char_glyph.atlas_size[1],
                    ]);

                    char_image.draw(
                        char_glyph.texture,
                        &c.draw_state,
                        c.transform.trans(char_x, char_y),
                        g,
                    );
                }
            }
        }
    }
}
