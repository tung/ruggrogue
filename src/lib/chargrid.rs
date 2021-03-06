use sdl2::{
    pixels::{Color as Sdl2Color, PixelFormatEnum},
    rect::Rect,
    render::{BlendMode, Texture, TextureCreator, WindowCanvas},
    surface::Surface,
    video::WindowContext,
};
use std::collections::HashMap;

type Color = [u8; 3];
type Position = [i32; 2];
type Size = [i32; 2];

const U32_SIZE: usize = std::mem::size_of::<u32>();

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
            fg: vec![[255; 3]; vec_size],
            bg: vec![[0; 3]; vec_size],
        }
    }

    fn clear_color(&mut self, fg: Option<Color>, bg: Option<Color>) {
        for e in self.chars.iter_mut() {
            *e = ' ';
        }

        let fg: Color = fg.unwrap_or([255; 3]);
        for e in self.fg.iter_mut() {
            *e = fg;
        }

        let bg: Color = bg.unwrap_or([0; 3]);
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
pub struct CharGrid<'b, 'f, 'r> {
    front: RawCharGrid,
    back: RawCharGrid,
    font: Surface<'f>,
    glyph_positions: HashMap<char, Position>,
    min_grid_size: Size,
    cell_size: Size,
    needs_render: bool,
    buffer: Surface<'b>,
    texture: Option<Texture<'r>>,
}

impl<'b, 'f, 'r> CharGrid<'b, 'f, 'r> {
    /// Create a new CharGrid with a given [width, height].  White is the default foreground color
    /// and black is the default background color.
    ///
    /// The font image should consist of a 16-by-16 grid of IBM code page 437 glyphs.
    pub fn new(font: Surface, grid_size: Size, min_grid_size: Size) -> CharGrid<'b, 'f, 'r> {
        assert!(grid_size[0] > 0 && grid_size[1] > 0);
        assert!(min_grid_size[0] > 0 && min_grid_size[1] > 0);

        let cell_width = font.width() as i32 / 16;
        let cell_height = font.height() as i32 / 16;
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
                             ≡±≥≤⌠⌡÷≈°∙·√ⁿ²■";
        let mut glyph_positions = HashMap::new();

        for (i, ch) in code_page_437.chars().enumerate() {
            glyph_positions.insert(
                ch,
                [i as i32 % 16 * cell_width, i as i32 / 16 * cell_height],
            );
        }

        // Reprocess the font image to make it easier to alpha blit.
        let mut font = font.convert_format(PixelFormatEnum::ARGB8888).unwrap();
        font.set_blend_mode(BlendMode::Blend).unwrap();

        // Convert font image to grayscale and use gray value as alpha.
        {
            let width = font.width() as usize;
            let height = font.height() as usize;
            let pitch = font.pitch() as usize;
            let format = font.pixel_format();

            font.with_lock_mut(|bytes| {
                for y in 0..height {
                    let row_start = y * pitch;

                    for x in 0..width {
                        let pixel_start = row_start + x * U32_SIZE;
                        let in_color = Sdl2Color::from_u32(
                            &format,
                            u32::from_ne_bytes([
                                bytes[pixel_start],
                                bytes[pixel_start + 1],
                                bytes[pixel_start + 2],
                                bytes[pixel_start + 3],
                            ]),
                        );
                        let red = in_color.r as u16;
                        let green = in_color.r as u16;
                        let blue = in_color.r as u16;
                        let gray = ((red * 30 + green * 59 + blue * 11) / 100) as u8;
                        let out_color = if gray == 0 {
                            Sdl2Color::RGBA(0, 0, 0, 0)
                        } else {
                            Sdl2Color::RGBA(255, 255, 255, gray)
                        };
                        let out_bytes = out_color.to_u32(&format).to_ne_bytes();

                        bytes[pixel_start..pixel_start + U32_SIZE]
                            .copy_from_slice(&out_bytes[..U32_SIZE]);
                    }
                }
            });
        }

        let grid_size = [
            grid_size[0].max(min_grid_size[0]).min(255),
            grid_size[1].max(min_grid_size[1]).min(255),
        ];

        CharGrid {
            front: RawCharGrid::new(grid_size),
            back: RawCharGrid::new(grid_size),
            font,
            glyph_positions,
            min_grid_size,
            cell_size: [cell_width, cell_height],
            needs_render: true,
            buffer: Surface::new(
                (cell_width * grid_size[0]) as u32,
                (cell_height * grid_size[1]) as u32,
                PixelFormatEnum::ARGB8888,
            )
            .unwrap(),
            texture: None,
        }
    }

    /// The grid width and height of the full CharGrid in cells.
    pub fn size_cells(&self) -> Size {
        self.front.size
    }

    /// Calculate the pixel width and height of a full CharGrid, given a font image and desired
    /// grid dimensions.
    pub fn size_px(font_image: &Surface, grid_size: Size, min_grid_size: Size) -> [u32; 2] {
        [
            font_image.width() / 16 * (grid_size[0] as u32).max(min_grid_size[0] as u32).min(255),
            font_image.height() / 16 * (grid_size[1] as u32).max(min_grid_size[1] as u32).min(255),
        ]
    }

    /// Prepare internal CharGrid buffers, adapting to the given pixel dimensions.
    ///
    /// # Panics
    ///
    /// Panics if the back buffer creation fails.
    pub fn prepare(&mut self, [px_width, px_height]: Size) {
        let new_size_cells = [
            (px_width / self.cell_size[0])
                .max(self.min_grid_size[0])
                .min(255),
            (px_height / self.cell_size[1])
                .max(self.min_grid_size[1])
                .min(255),
        ];

        if self.size_cells() != new_size_cells {
            self.front = RawCharGrid::new(new_size_cells);
            self.back = RawCharGrid::new(new_size_cells);
            self.needs_render = true;
            self.buffer = Surface::new(
                (new_size_cells[0] * self.cell_size[0]) as u32,
                (new_size_cells[1] * self.cell_size[1]) as u32,
                PixelFormatEnum::ARGB8888,
            )
            .unwrap();
            self.texture = None;
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
            let fg_diff = force || ffg != bfg;
            let bg_diff = force || fbg != bbg;
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
            let grid_x = index as i32 % grid_width;
            let grid_y = index as i32 / grid_width;
            let cell_width = self.cell_size[0] as u32;
            let cell_height = self.cell_size[1] as u32;
            let px = grid_x * cell_width as i32;
            let py = grid_y * cell_height as i32;

            // Render cell if a visible change has occurred.
            if char_diff || (fg_diff && !f_space) || bg_diff {
                let dest_rect = Rect::new(px, py, cell_width, cell_height);
                let bg_color = Sdl2Color::RGB(fbg[0], fbg[1], fbg[2]);

                self.buffer.fill_rect(dest_rect, bg_color).unwrap();

                if !f_space {
                    if let Some([glyph_x, glyph_y]) = self.glyph_positions.get(&fc) {
                        let src_rect = Rect::new(*glyph_x, *glyph_y, cell_width, cell_height);
                        let fg_color = Sdl2Color::RGB(ffg[0], ffg[1], ffg[2]);

                        self.font.set_color_mod(fg_color);
                        self.font
                            .blit(src_rect, &mut self.buffer, dest_rect)
                            .unwrap();
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
    ///
    /// # Panics
    ///
    /// Panics if:
    ///
    ///  * texture creation fails for whatever reason
    ///  * the texture fails to be updated
    ///  * the texture fails to be copied onto the canvas for whatever reason
    pub fn draw(
        &mut self,
        canvas: &mut WindowCanvas,
        texture_creator: &'r TextureCreator<WindowContext>,
    ) {
        if self.needs_render || self.texture.is_none() {
            if self.render(self.texture.is_none()) {
                let texture = match &mut self.texture {
                    Some(texture) => texture,
                    None => {
                        self.texture = Some(
                            texture_creator
                                .create_texture_streaming(
                                    PixelFormatEnum::RGB888,
                                    (self.front.size[0] * self.cell_size[0]) as u32,
                                    (self.front.size[1] * self.cell_size[1]) as u32,
                                )
                                .unwrap(),
                        );
                        self.texture.as_mut().unwrap()
                    }
                };

                texture
                    .update(
                        None,
                        self.buffer.without_lock().unwrap(),
                        self.buffer.pitch() as usize,
                    )
                    .unwrap();
            }
            self.needs_render = false;
        }

        canvas
            .copy(
                self.texture.as_ref().unwrap(),
                None,
                Rect::new(0, 0, self.buffer.width(), self.buffer.height()),
            )
            .unwrap();
    }
}
