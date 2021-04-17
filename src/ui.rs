use shipyard::{Get, UniqueView, View, World};

use crate::{
    chunked::ChunkedMapGrid, components::CombatStats, map::Map, message::Messages, player::PlayerId,
};
use ruggle::{
    util::{Color, Position, Size},
    Symbol, TileGrid, Tileset,
};

pub const SELECTED_BG: Color = Color {
    r: 0,
    g: 128,
    b: 255,
};

pub struct Options {
    pub tileset: u32,
    pub font: u32,
    pub map_zoom: u32,
    pub text_zoom: u32,
}

pub const MAP_GRID: usize = 0;
pub const UI_GRID: usize = 1;

fn draw_status_line<Y: Symbol>(world: &World, grid: &mut TileGrid<Y>, y: i32) {
    let mut x = 2;

    let depth = format!(" Depth: {} ", world.borrow::<UniqueView<Map>>().depth);
    grid.print_color((x, y), &depth, true, Color::YELLOW, None);
    x += depth.len() as i32 + 1;

    let (hp, max_hp) = world.run(
        |player_id: UniqueView<PlayerId>, combat_stats: View<CombatStats>| {
            let player_stats = combat_stats.get(player_id.0);

            (player_stats.hp, player_stats.max_hp)
        },
    );

    let hp_bar_length = grid.width() as i32 - x - 2;
    grid.draw_bar(
        false,
        (x, y),
        hp_bar_length,
        0,
        hp,
        max_hp,
        Color { r: 192, g: 0, b: 0 },
        None,
    );
    grid.print_color(
        (x, y),
        &format!("HP: {} / {}", hp, max_hp),
        false,
        Color::YELLOW,
        None,
    );
}

fn draw_messages<Y>(world: &World, grid: &mut TileGrid<Y>, active: bool, min_y: i32, max_y: i32)
where
    Y: Symbol,
{
    world.run(|messages: UniqueView<Messages>| {
        let fg = if active { Color::WHITE } else { Color::GRAY };
        for (y, message) in (min_y..=max_y).zip(messages.rev_iter()) {
            grid.put_char_color((0, y), '>', fg, None);
            grid.print_color((2, y), message, true, fg, None);
        }
    });
}

pub fn draw_ui<Y: Symbol>(world: &World, grid: &mut TileGrid<Y>, prompt: Option<&str>) {
    let w = grid.width() as i32;
    let h = grid.height() as i32;

    for x in 0..w {
        grid.put_char_color((x, 0), '─', Color::WHITE, None);
    }

    draw_status_line(world, grid, 0);

    if let Some(prompt) = prompt {
        grid.print_color((2, 1), prompt, true, Color::WHITE, None);
        draw_messages(world, grid, false, 2, h - 1);
    } else {
        draw_messages(world, grid, true, 1, h);
    }
}

/// Prepares grids to display the dungeon map and user interface.
pub fn prepare_main_grids<Y: Symbol>(
    chunked_map_grid: &mut ChunkedMapGrid,
    world: &World,
    grids: &mut Vec<TileGrid<Y>>,
    tilesets: &[Tileset<Y>],
    window_size: Size,
) {
    let Options {
        tileset: map_tileset_index,
        font: ui_tileset_index,
        text_zoom,
        ..
    } = *world.borrow::<UniqueView<Options>>();
    let ui_tileset = &tilesets
        .get(ui_tileset_index as usize)
        .unwrap_or(&tilesets[0]);

    let new_ui_size = Size {
        w: (window_size.w / (ui_tileset.tile_width() * text_zoom)).max(40),
        h: 5,
    };
    let new_ui_px_h = new_ui_size.h * ui_tileset.tile_height() * text_zoom;

    if !grids.is_empty() {
        // MAP_GRID resizing is handled by ChunkedMapGrid::prepare_grid below.
        grids[UI_GRID].resize(new_ui_size);
    } else {
        // Use a bogus size for MAP_GRID; ChunkedMapGrid::prepare_grid will resize it below.
        grids.push(TileGrid::new(
            Size { w: 1, h: 1 },
            tilesets,
            map_tileset_index as usize,
        ));
        grids.push(TileGrid::new(
            new_ui_size,
            tilesets,
            ui_tileset_index as usize,
        ));
        grids[UI_GRID].view.clear_color = Some(Color::BLACK);
    }

    chunked_map_grid.prepare_grid(
        world,
        &mut grids[MAP_GRID],
        tilesets,
        Position { x: 0, y: 0 },
        Size {
            w: window_size.w,
            h: window_size.h.saturating_sub(new_ui_px_h).max(1),
        },
    );

    grids[UI_GRID].set_tileset(tilesets, ui_tileset_index as usize);
    grids[UI_GRID].view.pos = Position {
        x: 0,
        y: window_size.h.saturating_sub(new_ui_px_h) as i32,
    };
    grids[UI_GRID].view.size = Size {
        w: window_size.w,
        h: new_ui_px_h,
    };
    grids[UI_GRID].view.zoom = text_zoom;
}
