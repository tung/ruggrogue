use shipyard::{Get, UniqueView, View, World};

use crate::{components::FieldOfView, gamesym::GameSym, map::Map, player::PlayerId, ui::Options};
use ruggrogue::{
    util::{Color, Position, Size},
    Symbol, TileGrid, Tileset,
};

pub const CHUNK_TILE_WIDTH: i32 = 8;
pub const CHUNK_TILE_HEIGHT: i32 = 8;

pub struct Camera(pub Position);

impl Camera {
    pub fn new() -> Self {
        Self(Position { x: 40, y: 25 })
    }
}

#[derive(Copy, Clone)]
struct ScreenChunk {
    dirty: bool,
    map_chunk: Position,
}

/// State and functions to support selective drawing of chunks of a map onto a TileGrid.
///
/// The TileGrid retains its drawn contents, parts of which are only redrawn if they are assigned
/// to draw a different map chunk or are manually flagged dirty.
///
/// TileGrids retain their drawn contents and are only redrawn if they are assigned to draw a new
/// map chunk, or are manually flagged dirty.  A ChunkedMapGrid is typically stored as part of a
/// Mode, which will call ChunkedMapGrid methods to arrange for the TileGrid to be drawn and
/// positioned, mark parts of the map as dirty so that they can be redrawn, and draw onto the
/// TileGrid.
///
/// A ChunkedMapGrid is centered on the map position of the camera.
pub struct ChunkedMapGrid {
    screen_chunks: Vec<ScreenChunk>,
    chunks_across: i32,
    chunks_down: i32,
    tile_size: Size,
    screen_size: Size,
    dirty_rects: Vec<(Position, Size)>,
}

impl ChunkedMapGrid {
    /// Create a new ChunkedMapGrid.
    pub fn new() -> Self {
        Self {
            screen_chunks: Vec::new(),
            chunks_across: 0,
            chunks_down: 0,
            tile_size: Size { w: 0, h: 0 },
            screen_size: Size { w: 0, h: 0 },
            dirty_rects: Vec::new(),
        }
    }

    /// Get the map chunk that should be at the top-left of the screen based on the position of the
    /// camera.
    fn screen_top_left_map_chunk(&self, world: &World) -> Position {
        let camera_pos = world.borrow::<UniqueView<Camera>>().0;
        let tile_px_w = self.tile_size.w as i32;
        let tile_px_h = self.tile_size.h as i32;
        let screen_px_w = self.screen_size.w as i32;
        let screen_px_h = self.screen_size.h as i32;
        let chunk_px_w = CHUNK_TILE_WIDTH * tile_px_w;
        let chunk_px_h = CHUNK_TILE_HEIGHT * tile_px_h;

        Position {
            x: (tile_px_w * (2 * camera_pos.x + 1) - screen_px_w).div_euclid(2 * chunk_px_w),
            y: (tile_px_h * (2 * camera_pos.y + 1) - screen_px_h).div_euclid(2 * chunk_px_h),
        }
    }

    /// Set up the grid to show chunks of the map centered about the camera at the given position
    /// and size on screen.
    pub fn prepare_grid<Y: Symbol>(
        &mut self,
        world: &World,
        grid: &mut TileGrid<Y>,
        tilesets: &[Tileset<Y>],
        pos: Position,
        size: Size,
    ) {
        let Options {
            tileset: map_tileset_index,
            map_zoom,
            ..
        } = *world.borrow::<UniqueView<Options>>();
        let map_tileset = &tilesets
            .get(map_tileset_index as usize)
            .unwrap_or(&tilesets[0]);
        let tile_px_w = map_zoom * map_tileset.tile_width();
        let tile_px_h = map_zoom * map_tileset.tile_height();
        let chunk_px_w = CHUNK_TILE_WIDTH * tile_px_w as i32;
        let chunk_px_h = CHUNK_TILE_HEIGHT * tile_px_h as i32;
        let screen_px_w = size.w as i32;
        let screen_px_h = size.h as i32;
        // We need to count (chunk_px_w - 1) twice: once to allow offsetting (chunk_px_w, 0], and
        // again to round up the chunk count to guarantee that the right edge of the screen is
        // covered.
        let new_chunks_across = (screen_px_w + 2 * (chunk_px_w - 1)) / chunk_px_w;
        // ... and again for (chunk_px_h - 1).
        let new_chunks_down = (screen_px_h + 2 * (chunk_px_h - 1)) / chunk_px_h;
        let new_len = (new_chunks_across * new_chunks_down) as usize;

        // Resize and invalidate all screen chunks if dimensions change.
        if new_chunks_across != self.chunks_across || new_chunks_down != self.chunks_down {
            for screen_chunk in self.screen_chunks.iter_mut().take(new_len) {
                screen_chunk.dirty = true;
            }
            self.screen_chunks.resize(
                new_len,
                ScreenChunk {
                    dirty: true,
                    map_chunk: Position { x: 0, y: 0 },
                },
            );

            self.chunks_across = new_chunks_across;
            self.chunks_down = new_chunks_down;
        }

        self.tile_size.w = tile_px_w;
        self.tile_size.h = tile_px_h;
        self.screen_size = size;

        grid.resize(Size {
            w: (self.chunks_across * CHUNK_TILE_WIDTH) as u32,
            h: (self.chunks_down * CHUNK_TILE_HEIGHT) as u32,
        });
        grid.set_tileset(tilesets, map_tileset_index as usize);

        grid.view.pos = pos;
        grid.view.size = size;
        grid.view.visible = true;
        grid.view.zoom = map_zoom;
    }

    /// Mark screen chunks as dirty so that they will be redrawn the next time that
    /// [ChunkedMapGrid::draw] is called.
    pub fn mark_dirty(&mut self, pos: Position, size: Size) {
        self.dirty_rects.push((pos, size));
    }

    /// Mark all screen chunks as dirty so that they will be redrawn the next time that
    /// [ChunkedMapGrid::draw] is called.
    pub fn mark_all_dirty(&mut self) {
        self.dirty_rects.clear();
        for screen_chunk in self.screen_chunks.iter_mut() {
            screen_chunk.dirty = true;
        }
    }

    /// Convert a map position into a grid position.
    pub fn map_to_grid_pos(&self, world: &World, map_pos: Position) -> Option<Position> {
        let top_left_chunk = self.screen_top_left_map_chunk(world);
        let top_left_x = top_left_chunk.x * CHUNK_TILE_WIDTH;
        let top_left_y = top_left_chunk.y * CHUNK_TILE_HEIGHT;

        if map_pos.x < top_left_x
            || map_pos.y < top_left_y
            || map_pos.x >= top_left_x + self.chunks_across * CHUNK_TILE_WIDTH
            || map_pos.y >= top_left_y + self.chunks_down * CHUNK_TILE_HEIGHT
        {
            return None;
        }

        Some(Position {
            x: map_pos.x - top_left_x,
            y: map_pos.y - top_left_y,
        })
    }

    /// Draw all screen chunks flagged dirty to their destination on the grid with their matching
    /// map chunk and clear their dirty flags.
    pub fn draw(&mut self, world: &World, grid: &mut TileGrid<GameSym>) {
        let camera_pos = world.borrow::<UniqueView<Camera>>().0;
        let camera_chunk_x = camera_pos.x / CHUNK_TILE_WIDTH;
        let camera_chunk_y = camera_pos.y / CHUNK_TILE_HEIGHT;
        let tile_px_w = self.tile_size.w as i32;
        let tile_px_h = self.tile_size.h as i32;
        let chunk_px_w = tile_px_w * CHUNK_TILE_WIDTH;
        let chunk_px_h = tile_px_h * CHUNK_TILE_HEIGHT;
        let screen_px_w = self.screen_size.w as i32;
        let screen_px_h = self.screen_size.h as i32;
        // Center pixel of the camera position inside the map chunk that it exists within.
        let camera_in_chunk_x = (camera_pos.x % CHUNK_TILE_WIDTH * 2 + 1) * tile_px_w / 2;
        let camera_in_chunk_y = (camera_pos.y % CHUNK_TILE_HEIGHT * 2 + 1) * tile_px_h / 2;
        let top_left_chunk = self.screen_top_left_map_chunk(world);
        let top_left_tile_x = top_left_chunk.x * CHUNK_TILE_WIDTH;
        let top_left_tile_y = top_left_chunk.y * CHUNK_TILE_HEIGHT;

        // Calculate where the top-left pixel of the top-left grid should be relative to pos.
        grid.view.dx =
            screen_px_w / 2 - (camera_chunk_x - top_left_chunk.x) * chunk_px_w - camera_in_chunk_x;
        grid.view.dy =
            screen_px_h / 2 - (camera_chunk_y - top_left_chunk.y) * chunk_px_h - camera_in_chunk_y;

        // Arrange for the top-left chunk to be drawn at the top left of its designated rectangle.
        grid.set_draw_offset(Position {
            x: top_left_tile_x,
            y: top_left_tile_y,
        });

        // Check if the screen chunks need to be redrawn due to being assigned to a different map
        // chunk.
        for chunk_y in 0..self.chunks_down {
            for chunk_x in 0..self.chunks_across {
                let screen_chunk_x = {
                    let tmp_x = top_left_chunk.x + chunk_x;
                    (tmp_x
                        + (tmp_x.abs() + self.chunks_across - 1) / self.chunks_across
                            * self.chunks_across)
                        % self.chunks_across
                };
                let screen_chunk_y = {
                    let tmp_y = top_left_chunk.y + chunk_y;
                    (tmp_y
                        + (tmp_y.abs() + self.chunks_down - 1) / self.chunks_down
                            * self.chunks_down)
                        % self.chunks_down
                };
                let index = (screen_chunk_y * self.chunks_across + screen_chunk_x) as usize;
                let screen_chunk = &mut self.screen_chunks[index];
                let new_map_chunk = Position {
                    x: top_left_chunk.x + chunk_x,
                    y: top_left_chunk.y + chunk_y,
                };

                if new_map_chunk != screen_chunk.map_chunk {
                    screen_chunk.map_chunk = new_map_chunk;
                    screen_chunk.dirty = true;
                }
            }
        }

        // Mark dirty rectangles too.
        for (dirty_pos, dirty_size) in self.dirty_rects.drain(..) {
            let start_chunk_x = dirty_pos.x.div_euclid(CHUNK_TILE_WIDTH);
            let start_chunk_y = dirty_pos.y.div_euclid(CHUNK_TILE_HEIGHT);
            let end_chunk_x = (dirty_pos.x + dirty_size.w as i32 - 1).div_euclid(CHUNK_TILE_WIDTH);
            let end_chunk_y = (dirty_pos.y + dirty_size.h as i32 - 1).div_euclid(CHUNK_TILE_HEIGHT);

            for dirty_chunk_y in start_chunk_y..=end_chunk_y {
                for dirty_chunk_x in start_chunk_x..=end_chunk_x {
                    let dirty_chunk_x = (dirty_chunk_x
                        + (dirty_chunk_x.abs() + self.chunks_across - 1) / self.chunks_across
                            * self.chunks_across)
                        % self.chunks_across;
                    let dirty_chunk_y = (dirty_chunk_y
                        + (dirty_chunk_y.abs() + self.chunks_down - 1) / self.chunks_down
                            * self.chunks_down)
                        % self.chunks_down;
                    let index = (dirty_chunk_y * self.chunks_across + dirty_chunk_x) as usize;

                    self.screen_chunks[index].dirty = true;
                }
            }
        }

        let map = world.borrow::<UniqueView<Map>>();
        let fovs = world.borrow::<View<FieldOfView>>();
        let player_fov = {
            let player_id = world.borrow::<UniqueView<PlayerId>>();
            fovs.get(player_id.0)
        };

        // Draw dirty grids and unflag them.
        for screen_chunk in self.screen_chunks.iter_mut() {
            if screen_chunk.dirty {
                for (tx, ty, tile) in map.iter_bounds(
                    screen_chunk.map_chunk.x * CHUNK_TILE_WIDTH,
                    screen_chunk.map_chunk.y * CHUNK_TILE_HEIGHT,
                    (screen_chunk.map_chunk.x + 1) * CHUNK_TILE_WIDTH - 1,
                    (screen_chunk.map_chunk.y + 1) * CHUNK_TILE_HEIGHT - 1,
                ) {
                    if let Some((sym, color)) = tile {
                        let color = if player_fov.get((tx, ty)) {
                            color
                        } else {
                            let v =
                                ((color.r as i32 * 30 + color.g as i32 * 59 + color.b as i32 * 11)
                                    / 200) as u8;
                            Color { r: v, g: v, b: v }
                        };

                        grid.put_sym_color_raw(
                            (tx - top_left_tile_x, ty - top_left_tile_y),
                            sym,
                            color,
                            Color::BLACK,
                        );
                    } else {
                        grid.put_char_color_raw(
                            (tx - top_left_tile_x, ty - top_left_tile_y),
                            ' ',
                            Color::WHITE,
                            Color::BLACK,
                        );
                    }
                }

                screen_chunk.dirty = false;
            }
        }
    }
}
