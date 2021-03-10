use sdl2::{
    pixels::{Color as Sdl2Color, PixelFormatEnum},
    rect::Rect,
    render::{BlendMode, Texture, TextureCreator, WindowCanvas},
    surface::Surface,
    video::WindowContext,
};
use std::collections::HashMap;

use crate::util::{Color, Position, Size};

const U32_SIZE: usize = std::mem::size_of::<u32>();

struct RawCharGrid {
    size: Size,
    chars: Vec<char>,
    fg: Vec<Color>,
    bg: Vec<Color>,
}

impl RawCharGrid {
    fn new(size: Size) -> RawCharGrid {
        assert_ne!(0, size.w);
        assert_ne!(0, size.h);
        assert!(size.w <= i32::MAX as u32);
        assert!(size.h <= i32::MAX as u32);

        let vec_size = (size.w * size.h) as usize;

        RawCharGrid {
            size,
            chars: vec![' '; vec_size],
            fg: vec![
                Color {
                    r: 255,
                    g: 255,
                    b: 255
                };
                vec_size
            ],
            bg: vec![Color { r: 0, g: 0, b: 0 }; vec_size],
        }
    }

    fn clear_color(&mut self, fg: Option<Color>, bg: Option<Color>) {
        for e in self.chars.iter_mut() {
            *e = ' ';
        }

        let fg = fg.unwrap_or(Color {
            r: 255,
            g: 255,
            b: 255,
        });
        for e in self.fg.iter_mut() {
            *e = fg;
        }

        let bg = bg.unwrap_or(Color { r: 0, g: 0, b: 0 });
        for e in self.bg.iter_mut() {
            *e = bg;
        }
    }

    fn put_color_raw(&mut self, pos: Position, fg: Option<Color>, bg: Option<Color>, c: char) {
        let Position { x, y } = pos;
        let index = (y * self.size.w as i32 + x) as usize;

        self.chars[index] = c;
        if let Some(c) = fg {
            self.fg[index] = c;
        }
        if let Some(c) = bg {
            self.bg[index] = c;
        }
    }

    fn put_color(&mut self, pos: Position, fg: Option<Color>, bg: Option<Color>, c: char) {
        if pos.x >= 0 && pos.y >= 0 && pos.x < self.size.w as i32 && pos.y < self.size.h as i32 {
            self.put_color_raw(pos, fg, bg, c);
        }
    }

    fn set_bg(&mut self, pos: Position, bg: Color) {
        if pos.x >= 0 && pos.y >= 0 && pos.x < self.size.w as i32 && pos.y < self.size.h as i32 {
            let index = (pos.y * self.size.w as i32 + pos.x) as usize;

            self.bg[index] = bg;
        }
    }

    fn print_color(&mut self, pos: Position, fg: Option<Color>, bg: Option<Color>, s: &str) {
        if pos.y >= 0
            && pos.y < self.size.h as i32
            && pos.x < self.size.w as i32
            && pos.x + s.len() as i32 > 0
        {
            let skip_chars = if pos.x < 0 { -pos.x as usize } else { 0 };

            for (i, c) in s.char_indices().skip(skip_chars).take(self.size.w as usize) {
                self.put_color_raw(
                    Position {
                        x: pos.x + i as i32,
                        y: pos.y,
                    },
                    fg,
                    bg,
                    c,
                );
            }
        }
    }

    fn draw_box(&mut self, pos: Position, size: Size, fg: Color, bg: Color) {
        let Position { x, y } = pos;
        let w = size.w as i32;
        let h = size.h as i32;
        let grid_w = self.size.w as i32;
        let grid_h = self.size.h as i32;

        if w > 0 && h > 0 && x + w > 0 && y + h > 0 && x < grid_w && y < grid_h {
            let fg = Some(fg);
            let bg = Some(bg);

            if y >= 0 {
                if x >= 0 {
                    self.put_color_raw(Position { x, y }, fg, bg, '┌');
                }
                for xx in std::cmp::max(0, x + 1)..std::cmp::min(grid_w, x + w - 1) {
                    self.put_color_raw(Position { x: xx, y }, fg, bg, '─');
                }
                if x + w - 1 < grid_w {
                    self.put_color_raw(Position { x: x + w - 1, y }, fg, bg, '┐');
                }
            }
            for yy in std::cmp::max(0, y + 1)..std::cmp::min(grid_h, y + h - 1) {
                if x >= 0 {
                    self.put_color_raw(Position { x, y: yy }, fg, bg, '│');
                }
                for xx in std::cmp::max(0, x + 1)..std::cmp::min(grid_w, x + w - 1) {
                    self.put_color_raw(Position { x: xx, y: yy }, fg, bg, ' ');
                }
                if x + w - 1 < grid_w {
                    self.put_color_raw(
                        Position {
                            x: x + w - 1,
                            y: yy,
                        },
                        fg,
                        bg,
                        '│',
                    );
                }
            }
            if y + h - 1 < grid_h {
                if x >= 0 {
                    self.put_color_raw(Position { x, y: y + h - 1 }, fg, bg, '└');
                }
                for xx in std::cmp::max(0, x + 1)..std::cmp::min(grid_w, x + w - 1) {
                    self.put_color_raw(
                        Position {
                            x: xx,
                            y: y + h - 1,
                        },
                        fg,
                        bg,
                        '─',
                    );
                }
                if x + w - 1 < grid_w {
                    self.put_color_raw(
                        Position {
                            x: x + w - 1,
                            y: y + h - 1,
                        },
                        fg,
                        bg,
                        '┘',
                    );
                }
            }
        }
    }

    fn draw_bar(
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
        assert!(length > 0);
        assert!(max >= 0);

        let Position { x, y } = pos;
        let grid_w = self.size.w as i32;
        let grid_h = self.size.h as i32;
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
            if x >= 0 && x < grid_w && y < grid_h && y + length >= 0 {
                for i in std::cmp::max(0, y)..std::cmp::min(grid_h, y + fill_start) {
                    self.put_color_raw(Position { x, y: i }, fg, bg, '░');
                }
                for i in std::cmp::max(0, y + fill_start)
                    ..std::cmp::min(grid_h, y + fill_start + fill_length)
                {
                    self.put_color_raw(Position { x, y: i }, fg, bg, '█');
                }
                for i in std::cmp::max(0, y + fill_start + fill_length)
                    ..std::cmp::min(grid_h, y + length)
                {
                    self.put_color_raw(Position { x, y: i }, fg, bg, '░');
                }
            }
        } else {
            if y >= 0 && y < grid_h && x < grid_w && x + length >= 0 {
                for i in std::cmp::max(0, x)..std::cmp::min(grid_w, x + fill_start) {
                    self.put_color_raw(Position { x: i, y }, fg, bg, '░');
                }
                for i in std::cmp::max(0, x + fill_start)
                    ..std::cmp::min(grid_w, x + fill_start + fill_length)
                {
                    self.put_color_raw(Position { x: i, y }, fg, bg, '█');
                }
                for i in std::cmp::max(0, x + fill_start + fill_length)
                    ..std::cmp::min(grid_w, x + length)
                {
                    self.put_color_raw(Position { x: i, y }, fg, bg, '░');
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
    force_render: bool,
    needs_render: bool,
    needs_upload: bool,
    buffer: Surface<'b>,
    texture: Option<Texture<'r>>,
}

impl<'b, 'f, 'r> CharGrid<'b, 'f, 'r> {
    /// Create a new CharGrid with a given width and height.  White is the default foreground color
    /// and black is the default background color.
    ///
    /// The font image should consist of a 16-by-16 grid of IBM code page 437 glyphs.
    pub fn new<G, M>(font: Surface, grid_size: G, min_grid_size: M) -> CharGrid<'b, 'f, 'r>
    where
        G: Into<Size>,
        M: Into<Size>,
    {
        let grid_size: Size = grid_size.into();
        let min_grid_size: Size = min_grid_size.into();

        assert!(grid_size.w > 0 && grid_size.h > 0);
        assert!(min_grid_size.w > 0 && min_grid_size.h > 0);

        let cell_width = font.width() / 16;
        let cell_height = font.height() / 16;

        assert!(cell_width <= i32::MAX as u32);
        assert!(cell_height <= i32::MAX as u32);

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
                Position {
                    x: i as i32 % 16 * cell_width as i32,
                    y: i as i32 / 16 * cell_height as i32,
                },
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

        let grid_size = Size {
            w: grid_size.w.max(min_grid_size.w).min(255),
            h: grid_size.h.max(min_grid_size.h).min(255),
        };

        CharGrid {
            front: RawCharGrid::new(grid_size),
            back: RawCharGrid::new(grid_size),
            font,
            glyph_positions,
            min_grid_size,
            cell_size: Size {
                w: cell_width,
                h: cell_height,
            },
            force_render: true,
            needs_render: true,
            needs_upload: true,
            buffer: Surface::new(
                (cell_width * grid_size.w) as u32,
                (cell_height * grid_size.h) as u32,
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
    pub fn size_px<G, M>(font_image: &Surface, grid_size: Size, min_grid_size: Size) -> [u32; 2]
    where
        G: Into<Size>,
        M: Into<Size>,
    {
        [
            font_image.width() / 16 * (grid_size.w as u32).max(min_grid_size.w as u32).min(255),
            font_image.height() / 16 * (grid_size.h as u32).max(min_grid_size.h as u32).min(255),
        ]
    }

    /// Prepare internal CharGrid buffers, adapting to the given pixel dimensions.
    ///
    /// # Panics
    ///
    /// Panics if the back buffer creation fails.
    pub fn prepare<P: Into<Size>>(&mut self, px_size: P) {
        let Size {
            w: px_width,
            h: px_height,
        } = px_size.into();
        let new_size_cells = Size {
            w: (px_width / self.cell_size.w)
                .max(self.min_grid_size.w)
                .min(255),
            h: (px_height / self.cell_size.h)
                .max(self.min_grid_size.h)
                .min(255),
        };

        if self.size_cells() != new_size_cells {
            self.front = RawCharGrid::new(new_size_cells);
            self.back = RawCharGrid::new(new_size_cells);
            self.force_render = true;
            self.needs_render = true;
            self.needs_upload = true;
            self.buffer = Surface::new(
                (new_size_cells.w * self.cell_size.w) as u32,
                (new_size_cells.h * self.cell_size.h) as u32,
                PixelFormatEnum::ARGB8888,
            )
            .unwrap();
            self.texture = None;
        }
    }

    /// Make the CharGrid reupload texture contents in the next call to [CharGrid::draw].
    pub fn flag_texture_reset(&mut self) {
        self.needs_upload = true;
    }

    /// Make the CharGrid recreate its texture in the next call to [CharGrid::draw].
    pub fn flag_texture_recreate(&mut self) {
        self.texture = None;
    }

    /// Clear the entire CharGrid.
    pub fn clear(&mut self) {
        self.clear_color(None, None);
    }

    /// Clear the entire CharGrid, optionally changing the foreground and/or background colors.
    pub fn clear_color<F, B>(&mut self, fg: F, bg: B)
    where
        F: Into<Option<Color>>,
        B: Into<Option<Color>>,
    {
        self.front.clear_color(fg.into(), bg.into());
        self.needs_render = true;
    }

    /// Put a single character in a given position.
    pub fn put<P: Into<Position>>(&mut self, pos: P, c: char) {
        self.put_color(pos.into(), None, None, c);
    }

    /// Put a single character in a given position, optionally changing the foreground and/or
    /// background colors.
    pub fn put_color<P, F, B>(&mut self, pos: P, fg: F, bg: B, c: char)
    where
        P: Into<Position>,
        F: Into<Option<Color>>,
        B: Into<Option<Color>>,
    {
        self.front.put_color(pos.into(), fg.into(), bg.into(), c);
        self.needs_render = true;
    }

    /// Like [CharGrid::put_color], but skips bounds checking.
    pub fn put_color_raw<P, F, B>(&mut self, pos: P, fg: F, bg: B, c: char)
    where
        P: Into<Position>,
        F: Into<Option<Color>>,
        B: Into<Option<Color>>,
    {
        self.front
            .put_color_raw(pos.into(), fg.into(), bg.into(), c);
        self.needs_render = true;
    }

    /// Set background color at a given position.
    pub fn set_bg<P, B>(&mut self, pos: P, bg: B)
    where
        P: Into<Position>,
        B: Into<Color>,
    {
        self.front.set_bg(pos.into(), bg.into());
        self.needs_render = true;
    }

    /// Print a string on the CharGrid starting at the given position.  If the string goes past the
    /// right edge of the CharGrid it will be truncated.
    pub fn print<P: Into<Position>>(&mut self, pos: P, s: &str) {
        self.print_color(pos.into(), None, None, s);
    }

    /// Print a string on the CharGrid starting at the given position, optionally changing the
    /// foreground and/or background colors.  If the string goes past the right edge of the
    /// CharGrid it will be truncated.
    pub fn print_color<P, F, B>(&mut self, pos: P, fg: F, bg: B, s: &str)
    where
        P: Into<Position>,
        F: Into<Option<Color>>,
        B: Into<Option<Color>>,
    {
        self.front.print_color(pos.into(), fg.into(), bg.into(), s);
        self.needs_render = true;
    }

    /// Draw a box on the CharGrid with the given size, position and foreground/background colors.
    /// Any part of the box that falls outside of the CharGrid will be clipped off.
    pub fn draw_box<P, S, F, B>(&mut self, pos: P, size: S, fg: F, bg: B)
    where
        P: Into<Position>,
        S: Into<Size>,
        F: Into<Color>,
        B: Into<Color>,
    {
        self.front
            .draw_box(pos.into(), size.into(), fg.into(), bg.into());
        self.needs_render = true;
    }

    /// Draw a bar of a given length starting at the given position.  Part of the bar is filled
    /// based on the offset, amount and max values, and the entire bar is colored based on the fg
    /// and bg colors provided.
    pub fn draw_bar<P, F, B>(
        &mut self,
        vertical: bool,
        pos: P,
        length: i32,
        offset: i32,
        amount: i32,
        max: i32,
        fg: F,
        bg: B,
    ) where
        P: Into<Position>,
        F: Into<Option<Color>>,
        B: Into<Option<Color>>,
    {
        self.front.draw_bar(
            vertical,
            pos.into(),
            length,
            offset,
            amount,
            max,
            fg.into(),
            bg.into(),
        );
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

            let grid_width = self.front.size.w as i32;
            let grid_x = index as i32 % grid_width;
            let grid_y = index as i32 / grid_width;
            let cell_width = self.cell_size.w as u32;
            let cell_height = self.cell_size.h as u32;
            let px = grid_x * cell_width as i32;
            let py = grid_y * cell_height as i32;

            // Render cell if a visible change has occurred.
            if char_diff || (fg_diff && !f_space) || bg_diff {
                let dest_rect = Rect::new(px, py, cell_width, cell_height);
                let bg_color = Sdl2Color::RGB(fbg.r, fbg.g, fbg.b);

                self.buffer.fill_rect(dest_rect, bg_color).unwrap();

                if !f_space {
                    if let Some(Position {
                        x: glyph_x,
                        y: glyph_y,
                    }) = self.glyph_positions.get(&fc)
                    {
                        let src_rect = Rect::new(*glyph_x, *glyph_y, cell_width, cell_height);
                        let fg_color = Sdl2Color::RGB(ffg.r, ffg.g, ffg.b);

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
        if self.needs_render || self.force_render {
            if self.render(self.force_render) {
                self.needs_upload = true;
                self.force_render = false;
            }
            self.needs_render = false;
        }

        let texture = match &mut self.texture {
            Some(texture) => texture,
            None => {
                self.texture = Some(
                    texture_creator
                        .create_texture_streaming(
                            PixelFormatEnum::RGB888,
                            (self.front.size.w * self.cell_size.w) as u32,
                            (self.front.size.h * self.cell_size.h) as u32,
                        )
                        .unwrap(),
                );
                self.needs_upload = true;
                self.texture.as_mut().unwrap()
            }
        };

        if self.needs_upload {
            texture
                .update(
                    None,
                    self.buffer.without_lock().unwrap(),
                    self.buffer.pitch() as usize,
                )
                .unwrap();
            self.needs_upload = false;
        }

        canvas
            .copy(
                texture,
                None,
                Rect::new(0, 0, self.buffer.width(), self.buffer.height()),
            )
            .unwrap();
    }
}
