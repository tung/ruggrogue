use piston_window::*;
use std::convert::TryInto;
use std::path::PathBuf;

type Color = [f32; 4];
type Position = [u32; 2];
type Size = [u32; 2];

struct CharGrid {
    size: Size,
    chars: Vec<char>,
    fg: Vec<Color>,
    bg: Vec<Color>,
}

impl CharGrid {
    fn new(size: Size) -> CharGrid {
        let [width, height] = size;

        assert_ne!(0, width);
        assert_ne!(0, height);

        let vec_size: usize = (width * height).try_into().unwrap();

        CharGrid {
            size,
            chars: vec![' '; vec_size],
            fg: vec![[1.; 4]; vec_size],
            bg: vec![[0., 0., 0., 1.]; vec_size],
        }
    }

    fn put(&mut self, [x, y]: Position, c: char) {
        let w = self.size[0];

        self.chars[(y * w + x) as usize] = c;
    }

    fn print(&mut self, [x, y]: Position, s: &str) {
        let width = self.size[0];

        s.char_indices()
            .take_while(|(i, _)| x + (*i as u32) < width)
            .for_each(|(i, c)| self.put([x + i as u32, y], c));
    }

    fn draw<G, C>(
        &self,
        font_size: u32,
        cache: &mut C,
        draw_state: &DrawState,
        transform: [[f64; 3]; 2],
        g: &mut G,
    ) where
        G: Graphics,
        C: character::CharacterCache<Texture = G::Texture>,
        <C as character::CharacterCache>::Error: std::fmt::Debug,
    {
        let char_image = Image::new();
        let char_width = cache.width(font_size, "W").unwrap();

        for y in 0..self.size[1] {
            for x in 0..self.size[0] {
                let index = (y * self.size[0] + x) as usize;
                let px = x as f64 * char_width;
                let py = (y * font_size) as f64;

                // Draw grid cell background.
                Rectangle::new(self.bg[index]).draw(
                    [px, py, char_width, font_size as f64],
                    draw_state,
                    transform,
                    g,
                );

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
                        draw_state,
                        transform.trans(char_x, char_y),
                        g,
                    );
                }
            }
        }
    }
}

fn main() {
    let mut window: PistonWindow = WindowSettings::new("Ruggle", [640, 480])
        .exit_on_esc(true)
        .build()
        .unwrap();
    window.set_lazy(true);

    let font_path = PathBuf::from("assets/LiberationMono-Regular.ttf");
    let font_size: u32 = 11;
    let mut glyph_cache = window.load_font(font_path).unwrap();

    let mut grid = CharGrid::new([80, 43]);
    grid.print([34, 21], "Hello world!");

    while let Some(e) = window.next() {
        window.draw_2d(&e, |c, g, d| {
            clear([0., 0., 1., 1.], g);

            grid.draw(font_size, &mut glyph_cache, &c.draw_state, c.transform, g);

            // Update glyphs before rendering.
            glyph_cache.factory.encoder.flush(d);
        });
    }
}
