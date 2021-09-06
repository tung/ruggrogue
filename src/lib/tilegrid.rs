use sdl2::{
    image::LoadSurface,
    pixels::{Color as Sdl2Color, PixelFormatEnum},
    rect::Rect,
    render::{BlendMode, Texture, TextureCreator, WindowCanvas},
    surface::Surface,
    video::WindowContext,
};
use std::{collections::HashMap, hash::Hash, path::PathBuf};

use crate::util::{Color, Position, Size};

const U32_SIZE: usize = std::mem::size_of::<u32>();

/// Position of a tile in a tile image.
pub type TileIndex = (i32, i32);

/// Bundle of traits needed for a type to be stored as part of a cell of a [TileGrid].
pub trait Symbol: Copy + Clone + Eq + PartialEq + Hash {
    fn text_fallback(self) -> char;
}

/// Data describing a tileset that can be loaded from an image on a file system.
pub struct TilesetInfo<Y: Symbol> {
    /// Path to the tile image.
    pub image_path: PathBuf,
    /// Pixel width and height of tiles in the tileset.
    pub tile_size: Size,
    /// Pixel offset of the top-left tile in the tileset.
    pub tile_start: Position,
    /// Number of pixels between tiles across.
    pub tile_gap: Size,
    /// Map of characters to glyph positions in the tile image.
    pub font_map: HashMap<char, TileIndex>,
    /// Map of symbols to tile positions in the tile image.
    pub symbol_map: HashMap<Y, TileIndex>,
}

impl<Y: Symbol> TilesetInfo<Y> {
    /// Make a font map that maps characters to a 16-by-16 grid of IBM Code Page 437 glyphs.
    pub fn map_code_page_437() -> HashMap<char, TileIndex> {
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

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
enum CellSym<Y: Symbol> {
    Char(char),
    Sym(Y),
}

/// A set of symbols mapped to positions in a tile image.
///
/// Used by TileGrid to measure out and render its contents to its buffer.
pub struct Tileset<'s, Y: Symbol> {
    surface: Surface<'s>,
    tile_size: Size,
    cellsym_map: HashMap<CellSym<Y>, Option<i32>>,
}

impl<'s, Y: Symbol> Tileset<'s, Y> {
    /// Check that tile indexes in the font map lie within image bounds.
    fn validate_tile_indexes<T: Iterator<Item = TileIndex>>(
        tile_indexes: T,
        tile_size: Size,
        tile_start: Position,
        tile_gap: Size,
        image_size: Size,
    ) {
        let tile_span_x = tile_size.w + tile_gap.w;
        let tile_span_y = tile_size.h + tile_gap.h;

        for (tile_x, tile_y) in tile_indexes {
            assert!(
                tile_x >= 0
                    && tile_y >= 0
                    && tile_start.x as u32 + tile_x as u32 * tile_span_x + tile_size.w
                        <= image_size.w
                    && tile_start.y as u32 + tile_y as u32 * tile_span_y + tile_size.h
                        <= image_size.h,
                "({}, {}) outside of tile image bounds",
                tile_x,
                tile_y,
            );
        }
    }

    /// Give y positions to TileIndex values that aren't already mapped.
    fn add_tile_index_to_pos_mappings<T: Iterator<Item = TileIndex>>(
        mapping: &mut HashMap<TileIndex, i32>,
        tile_indexes: T,
        tile_height: u32,
    ) {
        let tile_height = tile_height as i32;
        let mut tile_indexes_vec: Vec<TileIndex> = tile_indexes.collect();

        if !tile_indexes_vec.is_empty() {
            let mut y_pos = tile_height * mapping.len() as i32;

            // Keep tiles next to each other by index close by final position.
            tile_indexes_vec.sort_unstable_by_key(|&(x, y)| (y, x));

            for tile_index in tile_indexes_vec.iter() {
                if !mapping.contains_key(tile_index) {
                    mapping.insert(*tile_index, y_pos);
                    y_pos = y_pos.checked_add(tile_height).unwrap();
                }
            }
        }
    }

    /// Transfer tiles from source image to destination surface according to the mapping.
    fn transfer_tiles(
        surface: &mut Surface,
        image: &Surface,
        mapping: &HashMap<TileIndex, i32>,
        tile_size: Size,
        tile_start: Position,
        tile_gap: Size,
    ) {
        let surface_pitch = surface.pitch() as usize;
        let surface_format = surface.pixel_format();
        let surface_bytes = surface.without_lock_mut().unwrap();
        let image_bytes = image.without_lock().unwrap();

        for (&(tile_x, tile_y), &surface_y) in mapping {
            for y in 0..tile_size.h as usize {
                let surface_row_start = (surface_y as usize + y) * surface_pitch;
                let image_row_start = (tile_start.y as usize
                    + tile_y as usize * (tile_size.h + tile_gap.h) as usize
                    + y)
                    * image.pitch() as usize;

                for x in 0..tile_size.w as usize {
                    // Read the pixel color from the image.
                    let image_pixel_start = image_row_start
                        + (tile_start.x as usize
                            + tile_x as usize * (tile_size.w + tile_gap.w) as usize
                            + x)
                            * U32_SIZE;
                    let in_color = Sdl2Color::from_u32(
                        &image.pixel_format(),
                        u32::from_ne_bytes([
                            image_bytes[image_pixel_start],
                            image_bytes[image_pixel_start + 1],
                            image_bytes[image_pixel_start + 2],
                            image_bytes[image_pixel_start + 3],
                        ]),
                    );

                    // Use gray level to determine color and alpha:
                    //
                    //  * 0 gray => transparent black
                    //  * any other gray => white with alpha = gray
                    let red = in_color.r as u16;
                    let green = in_color.g as u16;
                    let blue = in_color.b as u16;
                    let gray = ((red * 30 + green * 59 + blue * 11) / 100) as u8;
                    let out_color = if gray == 0 {
                        Sdl2Color::RGBA(0, 0, 0, 0)
                    } else {
                        Sdl2Color::RGBA(255, 255, 255, gray)
                    };

                    // Write the output color to the surface.
                    let out_bytes = out_color.to_u32(&surface_format).to_ne_bytes();
                    let surface_pixel_start = surface_row_start + x * U32_SIZE;

                    surface_bytes[surface_pixel_start..surface_pixel_start + U32_SIZE]
                        .copy_from_slice(&out_bytes[..U32_SIZE]);
                }
            }
        }
    }

    /// Create a new tileset.  An [sdl2::image::Sdl2ImageContext] must be active at the time that
    /// this is called in order to load the tile image.
    ///
    /// # Panics
    ///
    /// Panics if no tiles are mapped, the tile image cannot be loaded, or if any entry of the font
    /// map lies outside the tile image bounds.
    pub fn new(tileset_info: TilesetInfo<Y>) -> Self {
        assert!(
            !tileset_info.font_map.is_empty() || !tileset_info.symbol_map.is_empty(),
            "at least one tile must be mapped"
        );
        assert!(tileset_info.tile_start.x >= 0 && tileset_info.tile_start.y >= 0);

        let tile_w = tileset_info.tile_size.w;
        let tile_h = tileset_info.tile_size.h;
        let image = Surface::from_file(tileset_info.image_path)
            .unwrap()
            .convert_format(PixelFormatEnum::ARGB8888)
            .unwrap();

        Self::validate_tile_indexes(
            tileset_info.font_map.values().copied(),
            tileset_info.tile_size,
            tileset_info.tile_start,
            tileset_info.tile_gap,
            Size {
                w: image.width(),
                h: image.height(),
            },
        );

        Self::validate_tile_indexes(
            tileset_info.symbol_map.values().copied(),
            tileset_info.tile_size,
            tileset_info.tile_start,
            tileset_info.tile_gap,
            Size {
                w: image.width(),
                h: image.height(),
            },
        );

        // Create a mapping from TileIndex to y positions.
        let mut tile_index_to_pos: HashMap<TileIndex, i32> = HashMap::new();

        Self::add_tile_index_to_pos_mappings(
            &mut tile_index_to_pos,
            tileset_info.font_map.values().copied(),
            tile_h,
        );

        Self::add_tile_index_to_pos_mappings(
            &mut tile_index_to_pos,
            tileset_info.symbol_map.values().copied(),
            tile_h,
        );

        let mut cellsym_map: HashMap<CellSym<Y>, Option<i32>> = HashMap::new();

        // Remap font map by y position instead of TileIndex.
        for (ch, tile_index) in tileset_info.font_map {
            cellsym_map.insert(
                CellSym::<Y>::Char(ch),
                tile_index_to_pos.get(&tile_index).copied(),
            );
        }

        // Remap symbol map by y position instead of TileIndex.
        for (sym, tile_index) in tileset_info.symbol_map {
            cellsym_map.insert(
                CellSym::<Y>::Sym(sym),
                tile_index_to_pos.get(&tile_index).copied(),
            );
        }

        // Create a one-tile-wide surface to transfer tiles from the image onto.
        let mut surface = Surface::new(
            tile_w,
            tile_h * tile_index_to_pos.len() as u32,
            PixelFormatEnum::ARGB8888,
        )
        .unwrap();

        surface.set_blend_mode(BlendMode::Blend).unwrap();

        Self::transfer_tiles(
            &mut surface,
            &image,
            &tile_index_to_pos,
            tileset_info.tile_size,
            tileset_info.tile_start,
            tileset_info.tile_gap,
        );

        Self {
            surface,
            tile_size: tileset_info.tile_size,
            cellsym_map,
        }
    }

    /// Pixel width of each tileset tile.
    pub fn tile_width(&self) -> u32 {
        self.tile_size.w
    }

    /// Pixel height of each tileset tile.
    pub fn tile_height(&self) -> u32 {
        self.tile_size.h
    }

    /// Draw a tileset tile onto `dest` at `rect` with a given `color`.
    fn draw_tile_to(&mut self, csym: CellSym<Y>, color: Color, dest: &mut Surface, rect: Rect) {
        let maybe_y: Option<i32> = match self.cellsym_map.get(&csym) {
            Some(&maybe_y) => maybe_y,
            None => match csym {
                CellSym::<Y>::Sym(sym) => {
                    // Cache fallback mappping result.
                    let fallback_ch = CellSym::<Y>::Char(sym.text_fallback());
                    let fallback_y = self.cellsym_map.get(&fallback_ch).copied().unwrap_or(None);
                    self.cellsym_map.insert(csym, fallback_y);
                    fallback_y
                }
                CellSym::<Y>::Char(_) => None,
            },
        };

        if let Some(y) = maybe_y {
            let color = Sdl2Color::RGB(color.r, color.g, color.b);
            let tile_rect = Rect::new(0, y, self.tile_size.w, self.tile_size.h);

            self.surface.set_color_mod(color);
            self.surface.blit(tile_rect, dest, rect).unwrap();
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct Cell<Y: Symbol> {
    csym: CellSym<Y>,
    fg: Color,
    bg: Color,
}

impl<Y: Symbol> Cell<Y> {
    #[inline]
    fn visible_diff(&self, other: &Cell<Y>) -> bool {
        self.csym != other.csym
            || (!matches!(self.csym, CellSym::<Y>::Char(' ')) && self.fg != other.fg)
            || self.bg != other.bg
    }
}

const DEFAULT_FG: Color = Color::WHITE;
const DEFAULT_BG: Color = Color::BLACK;

struct RawTileGrid<Y: Symbol> {
    size: Size,
    draw_offset: Position,
    cells: Vec<Cell<Y>>,
}

impl<Y: Symbol> RawTileGrid<Y> {
    fn new(size: Size) -> Self {
        assert_ne!(0, size.w);
        assert_ne!(0, size.h);
        assert!(size.w <= i32::MAX as u32);
        assert!(size.h <= i32::MAX as u32);

        Self {
            size,
            draw_offset: Position { x: 0, y: 0 },
            cells: vec![
                Cell {
                    csym: CellSym::<Y>::Char(' '),
                    fg: DEFAULT_FG,
                    bg: DEFAULT_BG,
                };
                (size.w * size.h) as usize
            ],
        }
    }

    fn resize(&mut self, new_size: Size) {
        if self.size != new_size {
            assert_ne!(0, new_size.w);
            assert_ne!(0, new_size.h);
            assert!(new_size.w <= i32::MAX as u32);
            assert!(new_size.h <= i32::MAX as u32);

            self.size = new_size;
            self.draw_offset = Position { x: 0, y: 0 };
            self.cells.resize(
                (new_size.w * new_size.h) as usize,
                Cell {
                    csym: CellSym::<Y>::Char(' '),
                    fg: DEFAULT_FG,
                    bg: DEFAULT_BG,
                },
            );
        }
    }

    fn set_draw_offset(&mut self, pos: Position) {
        // Keep draw_offset within the bounds of the grid.
        self.draw_offset.x = if pos.x >= 0 {
            pos.x % self.size.w as i32
        } else {
            self.size.w as i32 - (-pos.x % self.size.w as i32)
        };
        self.draw_offset.y = if pos.y >= 0 {
            pos.y % self.size.h as i32
        } else {
            self.size.h as i32 - (-pos.y % self.size.h as i32)
        };
    }

    fn clear_color<F, B>(&mut self, fg: F, bg: B)
    where
        F: Into<Option<Color>> + Copy,
        B: Into<Option<Color>> + Copy,
    {
        if let (Some(fg), Some(bg)) = (fg.into(), bg.into()) {
            self.cells.fill(Cell {
                csym: CellSym::<Y>::Char(' '),
                fg,
                bg,
            });
        } else {
            for cell in self.cells.iter_mut() {
                cell.csym = CellSym::<Y>::Char(' ');
                if let Some(fg) = fg.into() {
                    cell.fg = fg;
                }
                if let Some(bg) = bg.into() {
                    cell.bg = bg;
                }
            }
        }
    }

    #[inline]
    fn index(&self, Position { x, y }: Position) -> usize {
        let mut real_x = x + self.draw_offset.x;
        let mut real_y = y + self.draw_offset.y;

        if real_x >= self.size.w as i32 {
            real_x -= self.size.w as i32;
        }
        if real_y >= self.size.h as i32 {
            real_y -= self.size.h as i32;
        }

        (real_y * self.size.w as i32 + real_x) as usize
    }

    fn put_color_raw<F, B>(&mut self, pos: Position, csym: CellSym<Y>, fg: F, bg: B)
    where
        F: Into<Option<Color>> + Copy,
        B: Into<Option<Color>> + Copy,
    {
        let index = self.index(pos);
        let cell = &mut self.cells[index];

        cell.csym = csym;
        if let Some(fg) = fg.into() {
            cell.fg = fg;
        }
        if let Some(bg) = bg.into() {
            cell.bg = bg;
        }
    }

    fn put_color<F, B>(&mut self, pos: Position, sym: CellSym<Y>, fg: F, bg: B)
    where
        F: Into<Option<Color>> + Copy,
        B: Into<Option<Color>> + Copy,
    {
        if pos.x >= 0 && pos.y >= 0 && pos.x < self.size.w as i32 && pos.y < self.size.h as i32 {
            self.put_color_raw(pos, sym, fg, bg);
        }
    }

    fn recolor_pos<F, B>(&mut self, pos: Position, fg: F, bg: B)
    where
        F: Into<Option<Color>> + Copy,
        B: Into<Option<Color>> + Copy,
    {
        if pos.x >= 0 && pos.y >= 0 && pos.x < self.size.w as i32 && pos.y < self.size.h as i32 {
            let index = self.index(pos);

            if let Some(fg) = fg.into() {
                self.cells[index].fg = fg;
            }
            if let Some(bg) = bg.into() {
                self.cells[index].bg = bg;
            }
        }
    }

    fn print_color<F, B>(&mut self, pos: Position, s: &str, draw_space: bool, fg: F, bg: B)
    where
        F: Into<Option<Color>> + Copy,
        B: Into<Option<Color>> + Copy,
    {
        if pos.y >= 0
            && pos.y < self.size.h as i32
            && pos.x < self.size.w as i32
            && pos.x + s.len() as i32 > 0
        {
            for (i, c) in s
                .chars()
                .skip(-pos.x.min(0) as usize)
                .take(self.size.w.saturating_sub(pos.x.max(0) as u32) as usize)
                .filter(|c| draw_space || *c != ' ')
                .enumerate()
            {
                self.put_color_raw(
                    Position {
                        x: pos.x + i as i32,
                        y: pos.y,
                    },
                    CellSym::<Y>::Char(c),
                    fg,
                    bg,
                );
            }
        }
    }

    fn draw_box<F, B>(&mut self, pos: Position, size: Size, fg: F, bg: B)
    where
        F: Into<Option<Color>> + Copy,
        B: Into<Option<Color>> + Copy,
    {
        let Position { x, y } = pos;
        let w = size.w as i32;
        let h = size.h as i32;
        let grid_w = self.size.w as i32;
        let grid_h = self.size.h as i32;

        if w > 0 && h > 0 && x + w > 0 && y + h > 0 && x < grid_w && y < grid_h {
            if y >= 0 {
                if x >= 0 {
                    self.put_color_raw(Position { x, y }, CellSym::<Y>::Char('┌'), fg, bg);
                }
                for xx in std::cmp::max(0, x + 1)..std::cmp::min(grid_w, x + w - 1) {
                    self.put_color_raw(Position { x: xx, y }, CellSym::<Y>::Char('─'), fg, bg);
                }
                if x + w - 1 < grid_w {
                    self.put_color_raw(
                        Position { x: x + w - 1, y },
                        CellSym::<Y>::Char('┐'),
                        fg,
                        bg,
                    );
                }
            }
            for yy in std::cmp::max(0, y + 1)..std::cmp::min(grid_h, y + h - 1) {
                if x >= 0 {
                    self.put_color_raw(Position { x, y: yy }, CellSym::<Y>::Char('│'), fg, bg);
                }
                for xx in std::cmp::max(0, x + 1)..std::cmp::min(grid_w, x + w - 1) {
                    self.put_color_raw(Position { x: xx, y: yy }, CellSym::<Y>::Char(' '), fg, bg);
                }
                if x + w - 1 < grid_w {
                    self.put_color_raw(
                        Position {
                            x: x + w - 1,
                            y: yy,
                        },
                        CellSym::<Y>::Char('│'),
                        fg,
                        bg,
                    );
                }
            }
            if y + h - 1 < grid_h {
                if x >= 0 {
                    self.put_color_raw(
                        Position { x, y: y + h - 1 },
                        CellSym::<Y>::Char('└'),
                        fg,
                        bg,
                    );
                }
                for xx in std::cmp::max(0, x + 1)..std::cmp::min(grid_w, x + w - 1) {
                    self.put_color_raw(
                        Position {
                            x: xx,
                            y: y + h - 1,
                        },
                        CellSym::<Y>::Char('─'),
                        fg,
                        bg,
                    );
                }
                if x + w - 1 < grid_w {
                    self.put_color_raw(
                        Position {
                            x: x + w - 1,
                            y: y + h - 1,
                        },
                        CellSym::<Y>::Char('┘'),
                        fg,
                        bg,
                    );
                }
            }
        }
    }

    fn draw_bar<F, B>(
        &mut self,
        vertical: bool,
        pos: Position,
        length: i32,
        offset: i32,
        amount: i32,
        max: i32,
        fg: F,
        bg: B,
    ) where
        F: Into<Option<Color>> + Copy,
        B: Into<Option<Color>> + Copy,
    {
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

        #[allow(clippy::collapsible_else_if)]
        if vertical {
            if x >= 0 && x < grid_w && y < grid_h && y + length >= 0 {
                for i in std::cmp::max(0, y)..std::cmp::min(grid_h, y + fill_start) {
                    self.put_color_raw(Position { x, y: i }, CellSym::<Y>::Char('░'), fg, bg);
                }
                for i in std::cmp::max(0, y + fill_start)
                    ..std::cmp::min(grid_h, y + fill_start + fill_length)
                {
                    self.put_color_raw(Position { x, y: i }, CellSym::<Y>::Char(' '), bg, fg);
                }
                for i in std::cmp::max(0, y + fill_start + fill_length)
                    ..std::cmp::min(grid_h, y + length)
                {
                    self.put_color_raw(Position { x, y: i }, CellSym::<Y>::Char('░'), fg, bg);
                }
            }
        } else {
            if y >= 0 && y < grid_h && x < grid_w && x + length >= 0 {
                for i in std::cmp::max(0, x)..std::cmp::min(grid_w, x + fill_start) {
                    self.put_color_raw(Position { x: i, y }, CellSym::<Y>::Char('░'), fg, bg);
                }
                for i in std::cmp::max(0, x + fill_start)
                    ..std::cmp::min(grid_w, x + fill_start + fill_length)
                {
                    self.put_color_raw(Position { x: i, y }, CellSym::<Y>::Char(' '), bg, fg);
                }
                for i in std::cmp::max(0, x + fill_start + fill_length)
                    ..std::cmp::min(grid_w, x + length)
                {
                    self.put_color_raw(Position { x: i, y }, CellSym::<Y>::Char('░'), fg, bg);
                }
            }
        }
    }
}

/// Where and how a TileGrid should be displayed on screen.
pub struct TileGridView {
    /// Top-left pixel position of the clipping rectangle in which the TileGrid will be displayed.
    pub pos: Position,
    /// Pixel width and height of the clipping rectangle in which the TileGrid will be displayed.
    pub size: Size,
    /// x position of the TileGrid itself relative to pos.x.
    pub dx: i32,
    /// y position of the TileGrid itself relative to pos.y.
    pub dy: i32,
    /// If false, dont draw the TileGrid or clear behind it.
    pub visible: bool,
    /// Color to clear the clipping rectangle area to before drawing the TileGrid; None to skip.
    pub clear_color: Option<Color>,
    /// Color to multiply with the texture as it's displayed on the screen.
    pub color_mod: Color,
    /// Zoom factor of the TileGrid when displayed on screen.
    pub zoom: u32,
}

/// A TileGrid is a grid of cells consisting of a character, a foreground color and a background
/// color.  To use a TileGrid, create a new one, draw characters and colors onto it, and display it
/// on the screen.
pub struct TileGrid<'b, 'r, Y: Symbol> {
    front: RawTileGrid<Y>,
    back: RawTileGrid<Y>,
    force_render: bool,
    needs_render: bool,
    needs_upload: bool,
    tileset_index: usize,
    buffer: Option<Surface<'b>>,
    texture: Option<Texture<'r>>,
    pub view: TileGridView,
}

impl<'b, 'r, Y: Symbol> TileGrid<'b, 'r, Y> {
    /// Create a new TileGrid with a given width and height.
    ///
    /// White is the default foreground color and black is the default background color.
    ///
    /// By default, the TileGrid will be displayed at (0, 0) with a size of (640, 480) cleared to
    /// black.
    pub fn new(grid_size: Size, tilesets: &[Tileset<Y>], tileset_index: usize) -> Self {
        assert!(tileset_index < tilesets.len());

        Self {
            front: RawTileGrid::<Y>::new(grid_size),
            back: RawTileGrid::<Y>::new(grid_size),
            force_render: true,
            needs_render: true,
            needs_upload: true,
            tileset_index,
            buffer: None,
            texture: None,
            view: TileGridView {
                pos: Position { x: 0, y: 0 },
                size: Size { w: 640, h: 480 },
                dx: 0,
                dy: 0,
                visible: true,
                clear_color: Some(Color::BLACK),
                color_mod: Color::WHITE,
                zoom: 1,
            },
        }
    }

    /// The width of the TileGrid in cells.
    pub fn width(&self) -> u32 {
        self.front.size.w
    }

    /// The height of the TileGrid in cells.
    pub fn height(&self) -> u32 {
        self.front.size.h
    }

    /// Resize the TileGrid to the given grid dimensions, skipping if the dimensions are identical.
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

    /// Make the TileGrid reupload texture contents in the next call to [TileGrid::display].
    pub fn flag_texture_reset(&mut self) {
        self.needs_upload = true;
    }

    /// Make the TileGrid recreate its texture in the next call to [TileGrid::display].
    pub fn flag_texture_recreate(&mut self) {
        self.texture = None;
    }

    /// Get the tileset index for the Tileset assigned to the TileGrid.
    pub fn tileset(&self) -> usize {
        self.tileset_index
    }

    /// Assign a tileset for the TileGrid to be rendered with.
    pub fn set_tileset(&mut self, tilesets: &[Tileset<Y>], new_tileset_index: usize) {
        assert!(new_tileset_index < tilesets.len());

        if self.tileset_index != new_tileset_index {
            self.tileset_index = new_tileset_index;
            self.force_render = true;
        }
    }

    /// Prepare the TileGrid to be displayed centered within a given rectangle, possibly clipped.
    pub fn view_centered(
        &mut self,
        tilesets: &[Tileset<Y>],
        zoom: u32,
        rect_pos: Position,
        rect_size: Size,
    ) {
        let tileset = &tilesets[self.tileset_index];
        let px_width = self.front.size.w * tileset.tile_width() * zoom;
        let px_height = self.front.size.h * tileset.tile_height() * zoom;

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

    /// Set internal drawing offset hint to take advantage of wrapped offset rendering to reduce
    /// time spent rendering later on.
    ///
    /// This can greatly reduce the amount of rendering needed in the common case of a grid drawing
    /// a mostly static map centered on a camera position.  By setting the drawing offset to the
    /// camera position, the grid's internal view of the map can be kept still while the camera
    /// moves, instead of the other way around, reducing the number of tiles that need to be
    /// rerendered.  At display time, the internal buffer is rearranged to appear as if the camera
    /// had been centered with the map shifting around it the whole time.
    pub fn set_draw_offset(&mut self, pos: Position) {
        self.front.set_draw_offset(pos);
    }

    /// Clear the entire TileGrid.
    pub fn clear(&mut self) {
        self.clear_color(DEFAULT_FG, DEFAULT_BG);
    }

    /// Clear the entire TileGrid, optionally changing the foreground and/or background colors.
    pub fn clear_color<F, B>(&mut self, fg: F, bg: B)
    where
        F: Into<Option<Color>> + Copy,
        B: Into<Option<Color>> + Copy,
    {
        self.front.clear_color(fg, bg);
        self.needs_render = true;
    }

    /// Put a single character in a given position.
    pub fn put_char<P: Into<Position>>(&mut self, pos: P, ch: char) {
        self.put_char_color(pos, ch, DEFAULT_FG, DEFAULT_BG);
    }

    /// Put a symbol in a given position.
    pub fn put_sym<P: Into<Position>>(&mut self, pos: P, sym: Y) {
        self.put_sym_color(pos, sym, DEFAULT_FG, DEFAULT_BG);
    }

    /// Put a single character in a given position, optionally changing the foreground and/or
    /// background colors.
    pub fn put_char_color<P, F, B>(&mut self, pos: P, ch: char, fg: F, bg: B)
    where
        P: Into<Position>,
        F: Into<Option<Color>> + Copy,
        B: Into<Option<Color>> + Copy,
    {
        self.front
            .put_color(pos.into(), CellSym::<Y>::Char(ch), fg, bg);
        self.needs_render = true;
    }

    /// Put a symbol in a given position, optionally changing the foreground and/or background
    /// colors.
    pub fn put_sym_color<P, F, B>(&mut self, pos: P, sym: Y, fg: F, bg: B)
    where
        P: Into<Position>,
        F: Into<Option<Color>> + Copy,
        B: Into<Option<Color>> + Copy,
    {
        self.front
            .put_color(pos.into(), CellSym::<Y>::Sym(sym), fg, bg);
        self.needs_render = true;
    }

    /// Like [TileGrid::put_char_color], but skips bounds checking.
    pub fn put_char_color_raw<P, F, B>(&mut self, pos: P, ch: char, fg: F, bg: B)
    where
        P: Into<Position>,
        F: Into<Option<Color>> + Copy,
        B: Into<Option<Color>> + Copy,
    {
        self.front
            .put_color_raw(pos.into(), CellSym::<Y>::Char(ch), fg, bg);
        self.needs_render = true;
    }

    /// Like [TileGrid::put_sym_color], but skips bounds checking.
    pub fn put_sym_color_raw<P, F, B>(&mut self, pos: P, sym: Y, fg: F, bg: B)
    where
        P: Into<Position>,
        F: Into<Option<Color>> + Copy,
        B: Into<Option<Color>> + Copy,
    {
        self.front
            .put_color_raw(pos.into(), CellSym::<Y>::Sym(sym), fg, bg);
        self.needs_render = true;
    }

    /// Set foreground and background colors at a given position.
    pub fn recolor_pos<P, F, B>(&mut self, pos: P, fg: F, bg: B)
    where
        P: Into<Position>,
        F: Into<Option<Color>> + Copy,
        B: Into<Option<Color>> + Copy,
    {
        self.front.recolor_pos(pos.into(), fg, bg);
        self.needs_render = true;
    }

    /// Print a string on the TileGrid starting at the given position.  If the string goes past the
    /// right edge of the TileGrid it will be truncated.
    pub fn print<P: Into<Position>>(&mut self, pos: P, s: &str) {
        self.print_color(pos.into(), s, true, DEFAULT_FG, DEFAULT_BG);
    }

    /// Print a string on the TileGrid starting at the given position, optionally changing the
    /// foreground and/or background colors.  If the string goes past the right edge of the
    /// TileGrid it will be truncated.  If `skip_space` is true space characters will overwrite
    /// cells instead of skipping them and preserving their contents.
    pub fn print_color<P, F, B>(&mut self, pos: P, s: &str, draw_space: bool, fg: F, bg: B)
    where
        P: Into<Position>,
        F: Into<Option<Color>> + Copy,
        B: Into<Option<Color>> + Copy,
    {
        self.front.print_color(pos.into(), s, draw_space, fg, bg);
        self.needs_render = true;
    }

    /// Draw a box on the TileGrid with the given size and position.  Any part of the box that
    /// falls outside of the TileGrid will be clipped off.
    pub fn draw_box<P, S, F, B>(&mut self, pos: P, size: S, fg: F, bg: B)
    where
        P: Into<Position>,
        S: Into<Size>,
        F: Into<Option<Color>> + Copy,
        B: Into<Option<Color>> + Copy,
    {
        self.front.draw_box(pos.into(), size.into(), fg, bg);
        self.needs_render = true;
    }

    /// Draw a bar of a given length starting at the given position.  Part of the bar is filled
    /// based on the offset, amount and max values, and the entire bar is colored based on the
    /// current foreground and background colors.
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
        F: Into<Option<Color>> + Copy,
        B: Into<Option<Color>> + Copy,
    {
        self.front
            .draw_bar(vertical, pos.into(), length, offset, amount, max, fg, bg);
        self.needs_render = true;
    }

    fn render(&mut self, tileset: &mut Tileset<Y>, mut force: bool) -> bool {
        let mut buffer_updated = false;

        assert!(self.front.size == self.back.size);

        let buffer_px_w = self.front.size.w * tileset.tile_size.w;
        let buffer_px_h = self.front.size.h * tileset.tile_size.h;

        // Reset the buffer if it isn't the correct size to render to.
        if self.buffer.is_some() {
            let self_buffer_px_w = self.buffer.as_ref().unwrap().width();
            let self_buffer_px_h = self.buffer.as_ref().unwrap().height();

            if self_buffer_px_w != buffer_px_w || self_buffer_px_h != buffer_px_h {
                self.buffer = None;
                self.texture = None;
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

        let grid_width = self.front.size.w as i32;
        let cell_width = tileset.tile_size.w as u32;
        let cell_height = tileset.tile_size.h as u32;

        // Check the grid for positions to (re)render and (re)render them.
        for (i, (fcell, bcell)) in self
            .front
            .cells
            .iter_mut()
            .zip(self.back.cells.iter())
            .enumerate()
        {
            // Render cell if requested or a visible change has occurred.
            if force || fcell.visible_diff(bcell) {
                let dest_rect = Rect::new(
                    i as i32 % grid_width * cell_width as i32,
                    i as i32 / grid_width * cell_height as i32,
                    cell_width,
                    cell_height,
                );
                let bg_color = Sdl2Color::RGB(fcell.bg.r, fcell.bg.g, fcell.bg.b);

                buffer.fill_rect(dest_rect, bg_color).unwrap();

                if !matches!(fcell.csym, CellSym::<Y>::Char(' ')) {
                    tileset.draw_tile_to(fcell.csym, fcell.fg, buffer, dest_rect);
                }

                buffer_updated = true;
            }
        }

        // Update back buffer with front buffer contents.
        self.back.cells.copy_from_slice(&self.front.cells[..]);

        buffer_updated
    }

    /// Display the TileGrid onto the screen.
    ///
    /// A TileGrid maintains internal buffers to track changes since the last display, so it needs
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
        tilesets: &mut [Tileset<Y>],
        canvas: &mut WindowCanvas,
        texture_creator: &'r TextureCreator<WindowContext>,
    ) {
        if !self.view.visible || self.view.zoom == 0 {
            return;
        }

        let tileset = &mut tilesets[self.tileset_index];

        // If the buffer doesn't exist yet, it will need to be fully rendered.
        if self.buffer.is_none() {
            self.force_render = true;
        }

        // Render the drawn grid contents to the buffer.
        if self.needs_render || self.force_render {
            if self.render(tileset, self.force_render) {
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

        // Clear the destination rectangle first if requested.
        if let Some(clear_color) = self.view.clear_color {
            canvas.set_draw_color(Sdl2Color::RGB(clear_color.r, clear_color.g, clear_color.b));
            canvas.draw_rect(clip_rect).unwrap();
        }

        // Display the texture on the screen.
        texture.set_color_mod(
            self.view.color_mod.r,
            self.view.color_mod.g,
            self.view.color_mod.b,
        );
        canvas.set_clip_rect(clip_rect);

        let offset_x_px = self.front.draw_offset.x * tileset.tile_width() as i32;
        let offset_y_px = self.front.draw_offset.y * tileset.tile_height() as i32;

        // Display bottom-right of the texture at the top-left of the destination.
        let src_x = offset_x_px;
        let src_y = offset_y_px;
        let src_w = buffer.width() - offset_x_px as u32;
        let src_h = buffer.height() - offset_y_px as u32;
        let dest_x = self.view.pos.x + self.view.dx;
        let dest_y = self.view.pos.y + self.view.dy;
        canvas
            .copy(
                texture,
                Rect::new(src_x, src_y, src_w, src_h),
                Rect::new(
                    dest_x,
                    dest_y,
                    src_w * self.view.zoom,
                    src_h * self.view.zoom,
                ),
            )
            .unwrap();

        if offset_x_px > 0 {
            // Display bottom-left of the texture at the top-right of the destination.
            let src_x = 0;
            let src_y = offset_y_px;
            let src_w = offset_x_px as u32;
            let src_h = buffer.height() - offset_y_px as u32;
            let dest_x = self.view.pos.x
                + self.view.dx
                + (buffer.width() as i32 - offset_x_px) * self.view.zoom as i32;
            let dest_y = self.view.pos.y + self.view.dy;
            canvas
                .copy(
                    texture,
                    Rect::new(src_x, src_y, src_w, src_h),
                    Rect::new(
                        dest_x,
                        dest_y,
                        src_w * self.view.zoom,
                        src_h * self.view.zoom,
                    ),
                )
                .unwrap();

            if offset_y_px > 0 {
                // Display top-left of the texture at the bottom-right of the destination.
                let src_x = 0;
                let src_y = 0;
                let src_w = offset_x_px as u32;
                let src_h = offset_y_px as u32;
                let dest_x = self.view.pos.x
                    + self.view.dx
                    + (buffer.width() as i32 - offset_x_px) * self.view.zoom as i32;
                let dest_y = self.view.pos.y
                    + self.view.dy
                    + (buffer.height() as i32 - offset_y_px) * self.view.zoom as i32;
                canvas
                    .copy(
                        texture,
                        Rect::new(src_x, src_y, src_w, src_h),
                        Rect::new(
                            dest_x,
                            dest_y,
                            src_w * self.view.zoom,
                            src_h * self.view.zoom,
                        ),
                    )
                    .unwrap();
            }
        }

        if offset_y_px > 0 {
            // Display top-right of the texture at the bottom-left of the destination.
            let src_x = offset_x_px;
            let src_y = 0;
            let src_w = buffer.width() - offset_x_px as u32;
            let src_h = offset_y_px as u32;
            let dest_x = self.view.pos.x + self.view.dx;
            let dest_y = self.view.pos.y
                + self.view.dy
                + (buffer.height() as i32 - offset_y_px) * self.view.zoom as i32;
            canvas
                .copy(
                    texture,
                    Rect::new(src_x, src_y, src_w, src_h),
                    Rect::new(
                        dest_x,
                        dest_y,
                        src_w * self.view.zoom,
                        src_h * self.view.zoom,
                    ),
                )
                .unwrap();
        }
    }
}

/// A list of TileGrids that should be treated as a single layer.
pub struct TileGridLayer<'b, 'r, Y: Symbol> {
    /// If true, draw layers behind this one in a list of layers.
    pub draw_behind: bool,
    /// TileGrids to be drawn to, rendered and displayed as part of the layer.
    pub grids: Vec<TileGrid<'b, 'r, Y>>,
}
