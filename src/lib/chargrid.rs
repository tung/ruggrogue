use graphics::types::Color;
use graphics::{Context, Graphics};
use image::{ImageBuffer, Rgba, RgbaImage};
use opengl_graphics::{Texture, TextureSettings};
use std::collections::HashMap;

type Position = [i32; 2];
type Size = [i32; 2];

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

    fn put_color_raw(&mut self, [x, y]: Position, fg: Option<Color>, bg: Option<Color>, c: char) {
        let index = (y * self.size[0] + x) as usize;

        self.chars[index] = c;
        if let Some(c) = fg {
            self.fg[index] = c;
        }
        if let Some(c) = bg {
            self.bg[index] = c;
        }
    }

    fn put_color(&mut self, pos: Position, fg: Option<Color>, bg: Option<Color>, c: char) {
        if pos[0] >= 0 && pos[1] >= 0 && pos[0] < self.size[0] && pos[1] < self.size[1] {
            self.put_color_raw(pos, fg, bg, c);
        }
    }

    fn set_bg(&mut self, [x, y]: Position, bg: Color) {
        if x >= 0 && y >= 0 && x < self.size[0] && y < self.size[1] {
            let index = (y * self.size[0] + x) as usize;

            self.bg[index] = bg;
        }
    }

    fn print_color(&mut self, [x, y]: Position, fg: Option<Color>, bg: Option<Color>, s: &str) {
        if y >= 0 && y < self.size[1] && x < self.size[0] && x + s.len() as i32 > 0 {
            let skip_chars = if x < 0 { -x as usize } else { 0 };

            for (i, c) in s
                .char_indices()
                .skip(skip_chars)
                .take(self.size[0] as usize)
            {
                self.put_color_raw([x + i as i32, y], fg, bg, c);
            }
        }
    }

    fn draw_box(&mut self, [x, y]: Position, [w, h]: Size, fg: Color, bg: Color) {
        if w > 0 && h > 0 && x + w > 0 && y + h > 0 && x < self.size[0] && y < self.size[1] {
            let fg = Some(fg);
            let bg = Some(bg);

            if y >= 0 {
                if x >= 0 {
                    self.put_color_raw([x, y], fg, bg, '┌');
                }
                for xx in std::cmp::max(0, x + 1)..std::cmp::min(self.size[0], x + w - 1) {
                    self.put_color_raw([xx, y], fg, bg, '─');
                }
                if x + w - 1 < self.size[0] {
                    self.put_color_raw([x + w - 1, y], fg, bg, '┐');
                }
            }
            for yy in std::cmp::max(0, y + 1)..std::cmp::min(self.size[1], y + h - 1) {
                if x >= 0 {
                    self.put_color_raw([x, yy], fg, bg, '│');
                }
                for xx in std::cmp::max(0, x + 1)..std::cmp::min(self.size[0], x + w - 1) {
                    self.put_color_raw([xx, yy], fg, bg, ' ');
                }
                if x + w - 1 < self.size[0] {
                    self.put_color_raw([x + w - 1, yy], fg, bg, '│');
                }
            }
            if y + h - 1 < self.size[1] {
                if x >= 0 {
                    self.put_color_raw([x, y + h - 1], fg, bg, '└');
                }
                for xx in std::cmp::max(0, x + 1)..std::cmp::min(self.size[0], x + w - 1) {
                    self.put_color_raw([xx, y + h - 1], fg, bg, '─');
                }
                if x + w - 1 < self.size[0] {
                    self.put_color_raw([x + w - 1, y + h - 1], fg, bg, '┘');
                }
            }
        }
    }

    fn draw_bar(
        &mut self,
        vertical: bool,
        [x, y]: Position,
        length: i32,
        offset: i32,
        amount: i32,
        max: i32,
        fg: Option<Color>,
        bg: Option<Color>,
    ) {
        assert!(length > 0);
        assert!(max >= 0);

        let fill_length = if max > 0 {
            (length * amount / max).clamp(0, length)
        } else {
            0
        };
        let gap = length - fill_length;
        let fill_start = if gap > 0 && amount < max {
            gap * offset / (max - amount)
        } else {
            0
        };

        #[allow(clippy::collapsible_if)]
        if vertical {
            if x >= 0 && x < self.size[0] && y < self.size[1] && y + length >= 0 {
                for i in std::cmp::max(0, y)..std::cmp::min(self.size[1], y + fill_start) {
                    self.put_color_raw([x, i], fg, bg, '░');
                }
                for i in std::cmp::max(0, y + fill_start)
                    ..std::cmp::min(self.size[1], y + fill_start + fill_length)
                {
                    self.put_color_raw([x, i], fg, bg, '█');
                }
                for i in std::cmp::max(0, y + fill_start + fill_length)
                    ..std::cmp::min(self.size[1], y + length)
                {
                    self.put_color_raw([x, i], fg, bg, '░');
                }
            }
        } else {
            if y >= 0 && y < self.size[1] && x < self.size[0] && x + length >= 0 {
                for i in std::cmp::max(0, x)..std::cmp::min(self.size[0], x + fill_start) {
                    self.put_color_raw([i, y], fg, bg, '░');
                }
                for i in std::cmp::max(0, x + fill_start)
                    ..std::cmp::min(self.size[0], x + fill_start + fill_length)
                {
                    self.put_color_raw([i, y], fg, bg, '█');
                }
                for i in std::cmp::max(0, x + fill_start + fill_length)
                    ..std::cmp::min(self.size[0], x + length)
                {
                    self.put_color_raw([i, y], fg, bg, '░');
                }
            }
        }
    }
}

/// A CharGrid is a grid of cells consisting of a character, a foreground color and a background
/// color.  To use a CharGrid, create a new one, plot characters and colors onto it, and draw it to
/// the screen.
pub struct CharGrid {
    front: RawCharGrid,
    back: RawCharGrid,
    glyph_cache: HashMap<char, Vec<f32>>,
    min_grid_size: Size,
    cell_size: Size,
    needs_render: bool,
    buffer: RgbaImage,
    texture: Option<Texture>,
}

impl CharGrid {
    /// Create a new CharGrid with a given [width, height].  White is the default foreground color
    /// and black is the default background color.
    pub fn new(font_path: &std::path::PathBuf, grid_size: Size, min_grid_size: Size) -> CharGrid {
        assert!(grid_size[0] > 0 && grid_size[1] > 0);
        assert!(min_grid_size[0] > 0 && min_grid_size[1] > 0);

        use image::GenericImageView;

        // The font image should consist of a 16-by-16 grid of IBM code page 437 glyphs.
        let image = image::io::Reader::open(font_path)
            .unwrap()
            .decode()
            .unwrap();
        let cell_width = image.dimensions().0 as i32 / 16;
        let cell_height = image.dimensions().1 as i32 / 16;
        let code_page_437 = " ☺☻♥♦♣♠•◘○◙♂♀♪♫☼\
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
                             ≡±≥≤⌠⌡÷≈°∙·√ⁿ²■ ";
        let mut glyph_cache: HashMap<char, Vec<f32>> = HashMap::new();

        for (i, ch) in code_page_437.chars().enumerate() {
            let mut glyph = vec![0.0f32; (cell_width * cell_height) as usize];
            let char_x = i as u32 % 16 * cell_width as u32;
            let char_y = i as u32 / 16 * cell_height as u32;

            for (px_idx, glyph_px) in glyph.iter_mut().enumerate() {
                let px_x = char_x + px_idx as u32 % cell_width as u32;
                let px_y = char_y + px_idx as u32 / cell_width as u32;
                let px = image.get_pixel(px_x, px_y);

                // White for foreground color, black for background color.
                *glyph_px = px.0[0] as f32 * 0.3 / 255.0
                    + px.0[1] as f32 * 0.59 / 255.0
                    + px.0[2] as f32 * 0.11 / 255.0;
            }
            glyph_cache.insert(ch, glyph);
        }

        let grid_size = [
            std::cmp::min(255, std::cmp::max(min_grid_size[0], grid_size[0])),
            std::cmp::min(255, std::cmp::max(min_grid_size[1], grid_size[1])),
        ];

        CharGrid {
            front: RawCharGrid::new(grid_size),
            back: RawCharGrid::new(grid_size),
            glyph_cache,
            min_grid_size,
            cell_size: [cell_width, cell_height],
            needs_render: true,
            buffer: ImageBuffer::new(
                (cell_width * grid_size[0]) as u32,
                (cell_height * grid_size[1]) as u32,
            ),
            texture: None,
        }
    }

    /// The grid width and height of the full CharGrid in cells.
    pub fn size_cells(&self) -> Size {
        self.front.size
    }

    /// The pixel width and height of the full CharGrid.
    pub fn size_px(&self) -> Size {
        [
            self.front.size[0] * self.cell_size[0],
            self.front.size[1] * self.cell_size[1],
        ]
    }

    /// Resize the CharGrid to fill the specified pixel dimensions.
    pub fn resize_for_px(&mut self, px_size: Size) {
        let w = std::cmp::min(
            255,
            std::cmp::max(self.min_grid_size[0], px_size[0] / self.cell_size[0]),
        );
        let h = std::cmp::min(
            255,
            std::cmp::max(self.min_grid_size[1], px_size[1] / self.cell_size[1]),
        );

        self.front = RawCharGrid::new([w, h]);
        self.back = RawCharGrid::new([w, h]);
        self.needs_render = true;
        self.buffer = ImageBuffer::new(
            (w * self.cell_size[0]) as u32,
            (h * self.cell_size[1]) as u32,
        );
        self.texture = None;
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

    /// Like [CharGrid::put_color], but skips bounds checking.
    pub fn put_color_raw(&mut self, pos: Position, fg: Option<Color>, bg: Option<Color>, c: char) {
        self.front.put_color_raw(pos, fg, bg, c);
        self.needs_render = true;
    }

    /// Set background color at a given position.
    pub fn set_bg(&mut self, pos: Position, bg: Color) {
        self.front.set_bg(pos, bg);
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

    /// Draw a box on the CharGrid with the given size, position and foreground/background colors.
    /// Any part of the box that falls outside of the CharGrid will be clipped off.
    pub fn draw_box(&mut self, pos: Position, size: Size, fg: Color, bg: Color) {
        self.front.draw_box(pos, size, fg, bg);
        self.needs_render = true;
    }

    /// Draw a bar of a given length starting at the given position.  Part of the bar is filled
    /// based on the offset, amount and max values, and the entire bar is colored based on the fg
    /// and bg colors provided.
    pub fn draw_bar(
        &mut self,
        vertical: bool,
        pos: Position,
        length: i32,
        offset: i32,
        amount: i32,
        max: i32,
        fg: Option<Color>,
        bg: Option<Color>,
    ) {
        self.front
            .draw_bar(vertical, pos, length, offset, amount, max, fg, bg);
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
            let grid_x = index as u32 % grid_width as u32;
            let grid_y = index as u32 / grid_width as u32;
            let cell_width = self.cell_size[0] as u32;
            let cell_height = self.cell_size[1] as u32;
            let px = grid_x * cell_width;
            let py = grid_y * cell_height;

            // Render cell if a visible change has occurred.
            if char_diff || (fg_diff && !f_space) || bg_diff {
                let cached_glyph = match self.glyph_cache.get(&fc) {
                    Some(data) => data,
                    None => self.glyph_cache.get(&' ').unwrap(),
                };

                for y in 0..cell_height {
                    for x in 0..cell_width {
                        let v = cached_glyph[(y * cell_width + x) as usize];
                        let c = Rgba([
                            ((v * ffg[0] + (1. - v) * fbg[0]) * 255.) as u8,
                            ((v * ffg[1] + (1. - v) * fbg[1]) * 255.) as u8,
                            ((v * ffg[2] + (1. - v) * fbg[2]) * 255.) as u8,
                            ((v * ffg[3] + (1. - v) * fbg[3]) * 255.) as u8,
                        ]);

                        self.buffer.put_pixel(px + x, py + y, c);
                    }
                }

                buffer_updated = true;
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
            graphics::Image::new().draw(texture, &c.draw_state, c.transform, g);
        }
    }
}
