use graphics::character::CharacterCache;
use graphics::types::Color;
use graphics::{Context, Graphics};

type Position = [u32; 2];
type Size = [u32; 2];

fn eq_color(a: Color, b: Color) -> bool {
    (a[0] - b[0]).abs() <= f32::EPSILON
        && (a[1] - b[1]).abs() <= f32::EPSILON
        && (a[2] - b[2]).abs() <= f32::EPSILON
        && (a[3] - b[3]).abs() <= f32::EPSILON
}

/// A CharGrid is a grid of characters that can be drawn onto, and afterwards the whole grid can be
/// drawn on screen.  Each cell has a foreground and background color.
///
/// To use a CharGrid, create a new one, put and print characters and colors to it, then call draw
/// to put it all on screen.
pub struct CharGrid {
    /// Dimensions of the grid in characters: [width, height].
    size: Size,
    /// Text character in each cell.
    chars: Vec<char>,
    /// Foreground in each cell.
    fg: Vec<Color>,
    /// Background in each cell.
    bg: Vec<Color>,
}

impl CharGrid {
    /// Create a new CharGrid with a given [width, height].  Sets white and black as the default
    /// foreground and background colors respectively.
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

    /// Clear the entire CharGrid.
    pub fn clear(&mut self) {
        self.clear_color(None, None);
    }

    /// Clear the entire CharGrid, optionally changing the foreground and/or background colors.
    pub fn clear_color(&mut self, fg: Option<Color>, bg: Option<Color>) {
        for e in self.chars.iter_mut() {
            *e = ' ';
        }

        let fg: Color = fg.unwrap_or([1.; 4]);
        for e in self.fg.iter_mut() {
            *e = fg;
        }

        let bg: Color = bg.unwrap_or([0., 0., 0., 1.]);
        for e in self.bg.iter_mut() {
            *e = bg;
        }
    }

    /// Put a single character in a given position.
    pub fn put(&mut self, pos: Position, c: char) {
        self.put_color(pos, None, None, c);
    }

    /// Put a single character in a given position, optionally changing the foreground and/or
    /// background colors.
    pub fn put_color(&mut self, [x, y]: Position, fg: Option<Color>, bg: Option<Color>, c: char) {
        if x >= self.size[0] || y >= self.size[1] {
            return;
        }

        let index = (y * self.size[0] + x) as usize;

        self.chars[index] = c;

        if let Some(c) = fg {
            self.fg[index] = c;
        }

        if let Some(c) = bg {
            self.bg[index] = c;
        }
    }

    /// Print a string on the CharGrid starting at the given position.  If the string goes past the
    /// right edge of the CharGrid it will be truncated.
    pub fn print(&mut self, pos: Position, s: &str) {
        self.print_color(pos, None, None, s);
    }

    /// Print a string on the CharGrid starting at the given position, optionally changing the
    /// foreground and/or background colors.  If the string goes past the right edge of the
    /// CharGrid it will be truncated.
    pub fn print_color(&mut self, [x, y]: Position, fg: Option<Color>, bg: Option<Color>, s: &str) {
        let width = self.size[0];

        s.char_indices()
            .take_while(|(i, _)| x + (*i as u32) < width)
            .for_each(|(i, c)| self.put_color([x + i as u32, y], fg, bg, c));
    }

    /// Draw the whole CharGrid on screen with the given font and font size.
    pub fn draw<G, C>(&self, font_size: u32, cache: &mut C, c: &Context, g: &mut G)
    where
        G: Graphics,
        C: CharacterCache<Texture = G::Texture>,
        C::Error: std::fmt::Debug,
    {
        use graphics::{Image, Rectangle, Transformed};

        let char_image = Image::new();
        let sample_char = cache.character(font_size, '@').unwrap();
        let char_width = sample_char.atlas_size[0].ceil();
        let char_height = sample_char.atlas_size[1].ceil();
        let char_y_offset = sample_char.top();
        let mut char_bg = Rectangle::new([0., 0., 0., 1.]);

        // Draw default background color.
        char_bg.draw(
            [
                0.,
                0.,
                self.size[0] as f64 * char_width,
                self.size[1] as f64 * char_height,
            ],
            &c.draw_state,
            c.transform,
            g,
        );

        for y in 0..self.size[1] {
            for x in 0..self.size[0] {
                let index = (y * self.size[0] + x) as usize;
                let px = x as f64 * char_width;
                let py = y as f64 * char_height;

                // Draw cell background color if it differs from the default.
                if !eq_color(self.bg[index], [0., 0., 0., 1.]) {
                    char_bg.color = self.bg[index];
                    char_bg.draw(
                        [px, py, char_width, char_height],
                        &c.draw_state,
                        c.transform,
                        g,
                    );
                }

                // Draw text character.
                if let Ok(char_glyph) = cache.character(font_size, self.chars[index]) {
                    let char_x = px + char_glyph.left();
                    let char_y = py + char_y_offset - char_glyph.top();
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
