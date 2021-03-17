use sdl2::{
    image::LoadSurface,
    pixels::{Color as Sdl2Color, PixelFormatEnum},
    rect::Rect,
    render::{BlendMode, Texture, TextureCreator, WindowCanvas},
    surface::Surface,
    video::WindowContext,
};
use std::{collections::HashMap, path::PathBuf};

use crate::util::{Color, Position, Size};

const U32_SIZE: usize = std::mem::size_of::<u32>();

/// Glyph position of a character in a font image.
pub type FontIndex = (i32, i32);

/// Data describing a font that can be loaded from an image on a file system.
pub struct FontInfo {
    /// Path to the font image.
    pub image_path: PathBuf,
    /// Pixel width and height of glyphs in the font.
    pub glyph_size: Size,
    /// Map of characters to glyph positions in the font image.
    pub font_map: HashMap<char, FontIndex>,
}

impl FontInfo {
    /// Make a font map that maps characters to a 16-by-16 grid of IBM Code Page 437 glyphs.
    pub fn map_code_page_437() -> HashMap<char, FontIndex> {
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
        let mut font_map = HashMap::new();

        for (i, ch) in code_page_437.chars().enumerate() {
            font_map.insert(ch, (i as i32 % 16, i as i32 / 16));
        }

        font_map
    }
}

/// A set of characters mapped to positions in a font image.
///
/// Used by CharGrid to measure out and render its contents to its buffer.
pub struct Font<'f> {
    surface: Surface<'f>,
    glyph_size: Size,
    font_map: HashMap<char, FontIndex>,
}

impl<'f> Font<'f> {
    /// Check that all FontIndex entries in a font map are within the font image bounds.
    fn validate_font_map(
        font_map: &HashMap<char, FontIndex>,
        surface: &Surface,
        glyph_size: Size,
    ) -> bool {
        let glyph_w = glyph_size.w as i32;
        let glyph_h = glyph_size.h as i32;
        let glyph_span_x = glyph_w;
        let glyph_span_y = glyph_h;

        for font_index in font_map.values() {
            let (glyph_x, glyph_y) = *font_index;

            if glyph_x < 0
                || glyph_y < 0
                || glyph_x * glyph_span_x + glyph_w > surface.width() as i32
                || glyph_y * glyph_span_y + glyph_h > surface.height() as i32
            {
                return false;
            }
        }

        true
    }

    /// Create a new font.  An [sdl2::image::Sdl2ImageContext] must be active at the time that this
    /// is called in order to load the font image.
    ///
    /// The font image should consist of a 16-by-16 grid of IBM Code Page 437 glyphs.
    ///
    /// # Panics
    ///
    /// Panics if the font image cannot be loaded, or if any entry of the font map lies outside the
    /// font image bounds.
    pub fn new(font_info: FontInfo) -> Font<'f> {
        let surface = Surface::from_file(font_info.image_path).unwrap();
        let glyph_size = font_info.glyph_size;

        assert!(Font::validate_font_map(
            &font_info.font_map,
            &surface,
            glyph_size
        ));

        // Reprocess the font surface to make it easier to alpha blit.
        let mut surface = surface.convert_format(PixelFormatEnum::ARGB8888).unwrap();
        surface.set_blend_mode(BlendMode::Blend).unwrap();

        // Convert font image to grayscale and use gray value as alpha.
        {
            let width = surface.width() as usize;
            let height = surface.height() as usize;
            let pitch = surface.pitch() as usize;
            let format = surface.pixel_format();

            surface.with_lock_mut(|bytes| {
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

        Font {
            surface,
            glyph_size,
            font_map: font_info.font_map,
        }
    }

    /// Pixel width of each font glyph.
    pub fn glyph_width(&self) -> u32 {
        self.glyph_size.w
    }

    /// Pixel height of each font glyph.
    pub fn glyph_height(&self) -> u32 {
        self.glyph_size.h
    }

    /// Draw a font glyph onto `dest` at `rect` with a given `color`.
    fn draw_glyph_to(&mut self, ch: char, color: Color, dest: &mut Surface, rect: Rect) {
        if let Some((x, y)) = self.font_map.get(&ch) {
            let color = Sdl2Color::RGB(color.r, color.g, color.b);
            let glyph_rect = Rect::new(
                *x * self.glyph_size.w as i32,
                *y * self.glyph_size.h as i32,
                self.glyph_size.w,
                self.glyph_size.h,
            );

            self.surface.set_color_mod(color);
            self.surface.blit(glyph_rect, dest, rect).unwrap();
        }
    }
}

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

    fn resize(&mut self, new_size: Size) {
        if self.size != new_size {
            assert_ne!(0, new_size.w);
            assert_ne!(0, new_size.h);
            assert!(new_size.w <= i32::MAX as u32);
            assert!(new_size.h <= i32::MAX as u32);

            let new_vec_size = (new_size.w * new_size.h) as usize;

            self.size = new_size;
            self.chars.resize(new_vec_size, ' ');
            self.fg.resize(
                new_vec_size,
                Color {
                    r: 255,
                    g: 255,
                    b: 255,
                },
            );
            self.bg.resize(new_vec_size, Color { r: 0, g: 0, b: 0 });
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

/// Where and how a CharGrid should be displayed on screen.
pub struct CharGridView {
    /// Top-left pixel position of the clipping rectangle in which the CharGrid will be displayed.
    pub pos: Position,
    /// Pixel width and height of the clipping rectangle in which the CharGrid will be displayed.
    pub size: Size,
    /// x position of the CharGrid itself relative to pos.x.
    pub dx: i32,
    /// y position of the CharGrid itself relative to pos.y.
    pub dy: i32,
    /// If false, dont draw the CharGrid or clear behind it.
    pub visible: bool,
    /// Color to clear the clipping rectangle area to before drawing the CharGrid; None to skip.
    pub clear_color: Option<Color>,
}

/// A CharGrid is a grid of cells consisting of a character, a foreground color and a background
/// color.  To use a CharGrid, create a new one, draw characters and colors onto it, and display it
/// on the screen.
pub struct CharGrid<'b, 'r> {
    front: RawCharGrid,
    back: RawCharGrid,
    force_render: bool,
    needs_render: bool,
    needs_upload: bool,
    font_index: usize,
    buffer: Option<Surface<'b>>,
    texture: Option<Texture<'r>>,
    pub view: CharGridView,
}

impl<'b, 'r> CharGrid<'b, 'r> {
    /// Create a new CharGrid with a given width and height.
    ///
    /// White is the default foreground color and black is the default background color.
    ///
    /// By default, the CharGrid will be displayed at (0, 0) with a size of (640, 480) cleared to
    /// black.
    pub fn new(grid_size: Size, fonts: &[Font], font_index: usize) -> CharGrid<'b, 'r> {
        assert!(font_index < fonts.len());

        CharGrid {
            front: RawCharGrid::new(grid_size),
            back: RawCharGrid::new(grid_size),
            force_render: true,
            needs_render: true,
            needs_upload: true,
            font_index,
            buffer: None,
            texture: None,
            view: CharGridView {
                pos: Position { x: 0, y: 0 },
                size: Size { w: 640, h: 480 },
                dx: 0,
                dy: 0,
                visible: true,
                clear_color: Some(Color { r: 0, g: 0, b: 0 }),
            },
        }
    }

    /// The width of the CharGrid in cells.
    pub fn width(&self) -> u32 {
        self.front.size.w
    }

    /// The height of the CharGrid in cells.
    pub fn height(&self) -> u32 {
        self.front.size.h
    }

    /// Resize the CharGrid to the given grid dimensions, skipping if the dimensions are identical.
    ///
    /// If a resize occurs, the grid contents will need to be redrawn, and internal flags will be
    /// set to remake and redraw internal buffers.
    pub fn resize(&mut self, new_grid_size: Size) {
        if self.front.size != new_grid_size {
            self.front.resize(new_grid_size);
            self.back.resize(new_grid_size);
            self.force_render = true;
            self.needs_render = true;
            self.needs_upload = true;
            self.buffer = None;
            self.texture = None;
        }
    }

    /// Make the CharGrid reupload texture contents in the next call to [CharGrid::display].
    pub fn flag_texture_reset(&mut self) {
        self.needs_upload = true;
    }

    /// Make the CharGrid recreate its texture in the next call to [CharGrid::display].
    pub fn flag_texture_recreate(&mut self) {
        self.texture = None;
    }

    /// Get the font index for the Font assigned to the CharGrid.
    pub fn font(&self) -> usize {
        self.font_index
    }

    /// Assign a font for the CharGrid to be rendered with.
    pub fn set_font(&mut self, fonts: &[Font], new_font_index: usize) {
        assert!(new_font_index < fonts.len());

        if self.font_index != new_font_index {
            self.font_index = new_font_index;
            self.force_render = true;
        }
    }

    /// Prepare the CharGrid to be displayed centered within a given rectangle, possibly clipped.
    pub fn view_centered(&mut self, fonts: &[Font], rect_pos: Position, rect_size: Size) {
        let font = &fonts[self.font_index];
        let px_width = self.front.size.w * font.glyph_width();
        let px_height = self.front.size.h * font.glyph_height();

        if px_width <= rect_size.w {
            self.view.size.w = px_width;
            self.view.pos.x = rect_pos.x + (rect_size.w - px_width) as i32 / 2;
            self.view.dx = 0;
        } else {
            self.view.size.w = rect_size.w;
            self.view.pos.x = rect_pos.x;
            self.view.dx = -((px_width - rect_size.w) as i32 / 2);
        }

        if px_height <= rect_size.h {
            self.view.size.h = px_height;
            self.view.pos.y = rect_pos.y + (rect_size.h - px_height) as i32 / 2;
            self.view.dy = 0;
        } else {
            self.view.size.h = rect_size.h;
            self.view.pos.y = rect_pos.y;
            self.view.dy = -((px_height - rect_size.h) as i32 / 2);
        }
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

    fn render(&mut self, font: &mut Font, mut force: bool) -> bool {
        let mut buffer_updated = false;

        assert!(self.front.size == self.back.size);

        let buffer_px_w = self.front.size.w * font.glyph_size.w;
        let buffer_px_h = self.front.size.h * font.glyph_size.h;

        // Reset the buffer if it isn't the correct size to render to.
        if self.buffer.is_some() {
            let self_buffer_px_w = self.buffer.as_ref().unwrap().width();
            let self_buffer_px_h = self.buffer.as_ref().unwrap().height();

            if self_buffer_px_w != buffer_px_w || self_buffer_px_h != buffer_px_h {
                self.buffer = None;
            }
        }

        // Ensure the buffer exists.
        let buffer = match &mut self.buffer {
            Some(buffer) => buffer,
            None => {
                self.buffer = Some(
                    Surface::new(buffer_px_w, buffer_px_h, PixelFormatEnum::ARGB8888).unwrap(),
                );
                force = true;
                self.buffer.as_mut().unwrap()
            }
        };

        // Check the grid for positions to (re)render and (re)render them.
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
            let cell_width = font.glyph_size.w as u32;
            let cell_height = font.glyph_size.h as u32;
            let px = grid_x * cell_width as i32;
            let py = grid_y * cell_height as i32;

            // Render cell if a visible change has occurred.
            if char_diff || (fg_diff && !f_space) || bg_diff {
                let dest_rect = Rect::new(px, py, cell_width, cell_height);
                let bg_color = Sdl2Color::RGB(fbg.r, fbg.g, fbg.b);

                buffer.fill_rect(dest_rect, bg_color).unwrap();

                if !f_space {
                    font.draw_glyph_to(fc, ffg, buffer, dest_rect);
                }

                buffer_updated = true;
            }
        }

        buffer_updated
    }

    /// Display the CharGrid onto the screen.
    ///
    /// A CharGrid maintains internal buffers to track changes since the last display, so it needs
    /// to be mutable in order to update those buffers when these changes are detected.
    ///
    /// # Panics
    ///
    /// Panics if:
    ///
    ///  * buffer creation fails
    ///  * texture creation fails
    ///  * the texture fails to be updated
    ///  * the texture fails to be copied onto the canvas
    pub fn display(
        &mut self,
        fonts: &mut [Font],
        canvas: &mut WindowCanvas,
        texture_creator: &'r TextureCreator<WindowContext>,
    ) {
        if !self.view.visible {
            return;
        }

        // If the buffer doesn't exist yet, it will need to be fully rendered.
        if self.buffer.is_none() {
            self.force_render = true;
        }

        // Render the drawn grid contents to the buffer.
        if self.needs_render || self.force_render {
            if self.render(&mut fonts[self.font_index], self.force_render) {
                self.needs_upload = true;
                self.force_render = false;
            }
            self.needs_render = false;
        }

        // The buffer is guaranteed to exist here; make sure the texture exists too.
        let buffer = self.buffer.as_ref().unwrap();
        let texture = match &mut self.texture {
            Some(texture) => texture,
            None => {
                self.texture = Some(
                    texture_creator
                        .create_texture_streaming(
                            PixelFormatEnum::RGB888,
                            buffer.width(),
                            buffer.height(),
                        )
                        .unwrap(),
                );
                self.needs_upload = true;
                self.texture.as_mut().unwrap()
            }
        };

        // Upload the buffer contents to the texture if needed.
        if self.needs_upload {
            texture
                .update(
                    None,
                    buffer.without_lock().unwrap(),
                    buffer.pitch() as usize,
                )
                .unwrap();
            self.needs_upload = false;
        }

        let clip_rect = Rect::new(
            self.view.pos.x,
            self.view.pos.y,
            self.view.size.w,
            self.view.size.h,
        );
        let dest_rect = Rect::new(
            self.view.pos.x + self.view.dx,
            self.view.pos.y + self.view.dy,
            buffer.width(),
            buffer.height(),
        );

        // Clear the destination rectangle first if requested.
        if let Some(clear_color) = self.view.clear_color {
            canvas.set_draw_color(Sdl2Color::RGB(clear_color.r, clear_color.g, clear_color.b));
            canvas.draw_rect(clip_rect).unwrap();
        }

        // Display the texture on the screen.
        canvas.set_clip_rect(clip_rect);
        canvas.copy(texture, None, dest_rect).unwrap();
    }
}

/// A list of CharGrids that should be treated as a single layer.
pub struct CharGridLayer<'b, 'r> {
    /// If true, draw layers behind this one in a list of layers.
    pub draw_behind: bool,
    /// CharGrids to be drawn to, rendered and displayed as part of the layer.
    pub grids: Vec<CharGrid<'b, 'r>>,
}
