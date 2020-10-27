use graphics::types::Color;
use graphics::{Context, Graphics};
use image::{GenericImageView, ImageBuffer, Rgba, RgbaImage};
use opengl_graphics::{Texture, TextureSettings};
use rusttype::{Font, Scale};

type Position = [u32; 2];
type Size = [u32; 2];

fn eq_color(a: &Color, b: &Color) -> bool {
    (a[0] - b[0]).abs() <= f32::EPSILON
        && (a[1] - b[1]).abs() <= f32::EPSILON
        && (a[2] - b[2]).abs() <= f32::EPSILON
        && (a[3] - b[3]).abs() <= f32::EPSILON
}

struct RawCharGrid {
    size: Size,
    chars: Vec<char>,
    fg: Vec<Color>,
    bg: Vec<Color>,
}

impl RawCharGrid {
    fn new(size: Size) -> RawCharGrid {
        let [width, height] = size;

        assert_ne!(0, width);
        assert_ne!(0, height);

        let vec_size = (width * height) as usize;

        RawCharGrid {
            size,
            chars: vec![' '; vec_size],
            fg: vec![[1.; 4]; vec_size],
            bg: vec![[0., 0., 0., 1.]; vec_size],
        }
    }

    fn clear_color(&mut self, fg: Option<Color>, bg: Option<Color>) {
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

    fn put_color(&mut self, [x, y]: Position, fg: Option<Color>, bg: Option<Color>, c: char) {
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

    fn print_color(&mut self, [x, y]: Position, fg: Option<Color>, bg: Option<Color>, s: &str) {
        let width = self.size[0];

        s.char_indices()
            .take_while(|(i, _)| x + (*i as u32) < width)
            .for_each(|(i, c)| self.put_color([x + i as u32, y], fg, bg, c));
    }
}

/// A CharGrid is a grid of cells consisting of a character, a foreground color and a background
/// color.  To use a CharGrid, create a new one, plot characters and colors onto it, and draw it to
/// the screen.
pub struct CharGrid<'f> {
    front: RawCharGrid,
    back: RawCharGrid,
    font: &'f Font<'f>,
    font_scale: Scale,
    font_offset_y: u32,
    cell_size: Size,
    needs_render: bool,
    buffer: RgbaImage,
    texture: Option<Texture>,
}

impl<'f> CharGrid<'f> {
    /// Create a new CharGrid with a given [width, height].  White is the default foreground color
    /// and black is the default background color.
    pub fn new(grid_size: Size, font: &'f Font, font_size: f32) -> CharGrid<'f> {
        // Calculate the cell size based on font metrics in the desired size.
        let code_page_437 = "☺☻♥♦♣♠•◘○◙♂♀♪♫☼\
                             ►◄↕‼¶§▬↨↑↓→←∟↔▲▼ \
                             !\"#$%&'()*+,-./\
                             0123456789:;<=>?\
                             @ABCDEFGHIJKLMNO\
                             PQRSTUVWXYZ[\\]^_\
                             `abcdefghijklmno\
                             pqrstuvwxyz{|}~⌂\
                             ÇüéâäàåçêëèïîìÄÅ\
                             ÉæÆôöòûùÿÖÜ¢£¥₧ƒ\
                             áíóúñÑªº¿⌐¬½¼¡«»\
                             ░▒▓│┤╡╢╖╕╣║╗╝╜╛┐\
                             └┴┬├─┼╞╟╚╔╩╦╠═╬╧\
                             ╨╤╥╙╘╒╓╫╪┘┌█▄▌▐▀\
                             αßΓπΣσµτΦΘΩδ∞φε∩\
                             ≡±≥≤⌠⌡÷≈°∙·√ⁿ²■";

        let font_scale = Scale::uniform(font_size);
        let point = rusttype::point(0., 0.);

        // Don't track min_x; we'll just clip anything that draws left of x = 0.
        let mut max_x: i32 = 0;
        let mut min_y: i32 = 0;
        let mut max_y: i32 = 0;

        for c in code_page_437.chars() {
            let glyph = font.glyph(c).scaled(font_scale).positioned(point);

            if let Some(pbb) = glyph.pixel_bounding_box() {
                if pbb.max.x > max_x {
                    max_x = pbb.max.x;
                }
                if pbb.min.y < min_y {
                    min_y = pbb.min.y;
                }
                if pbb.max.y > max_y {
                    max_y = pbb.max.y;
                }
            }
        }

        let cell_width = max_x as u32;
        let cell_height = (max_y - min_y + 1) as u32;

        CharGrid {
            front: RawCharGrid::new(grid_size),
            back: RawCharGrid::new(grid_size),
            font,
            font_scale,
            font_offset_y: (-min_y) as u32,
            cell_size: [cell_width, cell_height],
            needs_render: true,
            buffer: ImageBuffer::new(cell_width * grid_size[0], cell_height * grid_size[1]),
            texture: None,
        }
    }

    /// Clear the entire CharGrid.
    pub fn clear(&mut self) {
        self.clear_color(None, None);
    }

    /// Clear the entire CharGrid, optionally changing the foreground and/or background colors.
    pub fn clear_color(&mut self, fg: Option<Color>, bg: Option<Color>) {
        self.front.clear_color(fg, bg);
        self.needs_render = true;
    }

    /// Put a single character in a given position.
    pub fn put(&mut self, pos: Position, c: char) {
        self.put_color(pos, None, None, c);
    }

    /// Put a single character in a given position, optionally changing the foreground and/or
    /// background colors.
    pub fn put_color(&mut self, pos: Position, fg: Option<Color>, bg: Option<Color>, c: char) {
        self.front.put_color(pos, fg, bg, c);
        self.needs_render = true;
    }

    /// Print a string on the CharGrid starting at the given position.  If the string goes past the
    /// right edge of the CharGrid it will be truncated.
    pub fn print(&mut self, pos: Position, s: &str) {
        self.print_color(pos, None, None, s);
    }

    /// Print a string on the CharGrid starting at the given position, optionally changing the
    /// foreground and/or background colors.  If the string goes past the right edge of the
    /// CharGrid it will be truncated.
    pub fn print_color(&mut self, pos: Position, fg: Option<Color>, bg: Option<Color>, s: &str) {
        self.front.print_color(pos, fg, bg, s);
        self.needs_render = true;
    }

    fn render(&mut self, force: bool) -> bool {
        let mut buffer_updated = false;

        assert!(self.front.size == self.back.size);

        for index in 0..self.front.chars.len() {
            let fc = self.front.chars[index];
            let ffg = self.front.fg[index];
            let fbg = self.front.bg[index];
            let bc = self.back.chars[index];
            let bfg = self.back.fg[index];
            let bbg = self.back.bg[index];

            // Check for any changes between the front and back.
            let char_diff = force || fc != bc;
            let fg_diff = force || !eq_color(&ffg, &bfg);
            let bg_diff = force || !eq_color(&fbg, &bbg);
            let f_space = !force && fc == ' ';
            let b_space = !force && bc == ' ';

            // Update the back data with the front data.
            if char_diff {
                self.back.chars[index] = fc;
            }
            if fg_diff {
                self.back.fg[index] = ffg;
            }
            if bg_diff {
                self.back.bg[index] = fbg;
            }

            let grid_width = self.front.size[0];
            let grid_x = index as u32 % grid_width;
            let grid_y = index as u32 / grid_width;
            let cell_width = self.cell_size[0];
            let cell_height = self.cell_size[1];
            let px = grid_x * cell_width;
            let py = grid_y * cell_height;

            // Render background if chars differ, backgrounds differ or foregrounds differ and
            // either the front or back chars were non-space.
            if char_diff || bg_diff || (fg_diff && (!f_space || !b_space)) {
                for y in py..py + cell_height {
                    for x in px..px + cell_width {
                        let c = Rgba([
                            (fbg[0] * 255.0) as u8,
                            (fbg[1] * 255.0) as u8,
                            (fbg[2] * 255.0) as u8,
                            255,
                        ]);

                        self.buffer.put_pixel(x, y, c);
                    }
                }

                buffer_updated = true;
            }

            // Render the char if it's not space and anything changed at all.
            if !f_space && (char_diff || fg_diff || bg_diff) {
                let point = rusttype::point(px as f32, (py + self.font_offset_y) as f32);
                let glyph = self
                    .font
                    .glyph(fc)
                    .scaled(self.font_scale)
                    .positioned(point);

                if let Some(pbb) = glyph.pixel_bounding_box() {
                    glyph.draw(|x, y, v| {
                        if self.buffer.in_bounds(x, y) {
                            // Exaggerate font pixels so they stand out more.
                            let v = 1.0f32 - (1.0f32 - v) * (1.0f32 - v);
                            let c = Rgba([
                                ((v * ffg[0] + (1. - v) * fbg[0]) * 255.0) as u8,
                                ((v * ffg[1] + (1. - v) * fbg[1]) * 255.0) as u8,
                                ((v * ffg[2] + (1. - v) * fbg[2]) * 255.0) as u8,
                                255,
                            ]);

                            self.buffer.put_pixel(
                                (pbb.min.x + x as i32) as u32,
                                (pbb.min.y + y as i32) as u32,
                                c,
                            );
                        }
                    });

                    buffer_updated = true;
                }
            }
        }

        buffer_updated
    }

    /// Draw the CharGrid onto the screen.
    ///
    /// A CharGrid maintains internal buffers to track changes since the last draw, so it needs to
    /// be mutable in order to update those buffers when these changes are detected.
    pub fn draw<G>(&mut self, c: &Context, g: &mut G)
    where
        G: Graphics<Texture = opengl_graphics::Texture>,
    {
        if self.needs_render {
            let no_texture = self.texture.is_none();
            let buffer_updated = self.render(no_texture);

            if no_texture {
                self.texture = Some(Texture::from_image(&self.buffer, &TextureSettings::new()));
            } else if buffer_updated {
                self.texture.as_mut().unwrap().update(&self.buffer);
            }

            self.needs_render = false;
        }

        if let Some(texture) = &self.texture {
            use graphics::Image;

            Image::new().draw(texture, &c.draw_state, c.transform, g);
        }
    }
}
