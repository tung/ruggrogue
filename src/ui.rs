use shipyard::{Get, UniqueView, View, World};

use crate::{components::CombatStats, map::Map, message::Messages, player::PlayerId};
use ruggle::{
    util::{Color, Position, Size},
    CharGrid, Font,
};

pub mod color {
    use super::Color;

    pub const WHITE: Color = Color {
        r: 255,
        g: 255,
        b: 255,
    };
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0 };
    pub const RED: Color = Color { r: 255, g: 0, b: 0 };
    pub const BLUE: Color = Color { r: 0, g: 0, b: 255 };
    pub const YELLOW: Color = Color {
        r: 255,
        g: 255,
        b: 0,
    };
    pub const MAGENTA: Color = Color {
        r: 255,
        g: 0,
        b: 255,
    };
    pub const CYAN: Color = Color {
        r: 0,
        g: 255,
        b: 255,
    };
    pub const ORANGE: Color = Color {
        r: 255,
        g: 166,
        b: 0,
    };
    pub const PURPLE: Color = Color {
        r: 128,
        g: 0,
        b: 128,
    };
    pub const PINK: Color = Color {
        r: 255,
        g: 191,
        b: 204,
    };

    pub const SELECTED_BG: Color = Color {
        r: 0,
        g: 128,
        b: 255,
    };
}

pub fn recolor(c: Color, active: bool) -> Color {
    if active {
        c
    } else {
        // Dim and desaturate.
        let Color { r, g, b } = c;
        let (r, g, b) = (r as i32, g as i32, b as i32);
        let gray = (r * 30 + g * 59 + b * 11) / 100;

        Color {
            r: ((r + gray) * 3 / 10) as u8,
            g: ((g + gray) * 3 / 10) as u8,
            b: ((b + gray) * 3 / 10) as u8,
        }
    }
}

pub const MAP_GRID: usize = 0;
pub const UI_GRID: usize = 1;
pub const DEFAULT_MAP_FONT: usize = 1;

fn draw_status_line(world: &World, grid: &mut CharGrid, active: bool, y: i32) {
    let mut x = 2;

    let depth = format!(" Depth: {} ", world.borrow::<UniqueView<Map>>().depth);
    grid.print_color((x, y), recolor(color::YELLOW, active), None, &depth);
    x += depth.len() as i32 + 1;

    let (hp, max_hp) = world.run(
        |player_id: UniqueView<PlayerId>, combat_stats: View<CombatStats>| {
            let player_stats = combat_stats.get(player_id.0);

            (player_stats.hp, player_stats.max_hp)
        },
    );
    let hp_string = format!(" HP: {} / {} ", hp, max_hp);
    grid.print_color((x, y), recolor(color::YELLOW, active), None, &hp_string);
    x += hp_string.len() as i32 + 1;

    let hp_bar_length = grid.width() as i32 - x - 2;
    grid.draw_bar(
        false,
        (x, y),
        hp_bar_length,
        0,
        hp,
        max_hp,
        recolor(color::RED, active),
        None,
    );
}

fn draw_messages(world: &World, grid: &mut CharGrid, active: bool, min_y: i32, max_y: i32) {
    world.run(|messages: UniqueView<Messages>| {
        let fg = recolor(color::WHITE, active);

        for (y, message) in (min_y..=max_y).zip(messages.rev_iter()) {
            grid.put_color((0, y), fg, None, '>');
            grid.print_color((2, y), fg, None, message);
        }
    });
}

pub fn draw_ui(world: &World, grid: &mut CharGrid, active: bool, prompt: Option<&str>) {
    let w = grid.width() as i32;
    let h = grid.height() as i32;
    let fg = recolor(color::WHITE, active);

    for x in 0..w {
        grid.put_color((x, 0), fg, None, '─');
    }

    draw_status_line(world, grid, active, 0);

    if let Some(prompt) = prompt {
        grid.print_color((2, 1), fg, None, prompt);
        draw_messages(world, grid, false, 2, h - 1);
    } else {
        draw_messages(world, grid, active, 1, h);
    }
}

/// Prepares grid 0 and grid 1 to display the dungeon map and user interface respectively.
pub fn prepare_main_grids(grids: &mut Vec<CharGrid>, fonts: &[Font], window_size: Size) {
    let map_font = &fonts[grids.get(MAP_GRID).map_or(DEFAULT_MAP_FONT, CharGrid::font)];
    let ui_font = &fonts[grids.get(UI_GRID).map_or(0, CharGrid::font)];

    let new_ui_size = Size {
        w: (window_size.w / ui_font.glyph_width()).max(40),
        h: 5,
    };
    let new_ui_px_h = new_ui_size.h * ui_font.glyph_height();

    // 17 == standard field of view range * 2 + 1
    let mut new_map_w = (window_size.w / map_font.glyph_width()).max(17);
    if window_size.w % map_font.glyph_width() > 0 {
        // Fill to the edge of the screen.
        new_map_w += 1;
    }
    if new_map_w & 1 == 0 {
        // Ensure a single center tile exists using an odd number of tiles.
        new_map_w += 1;
    }

    // 17 == standard field of view range * 2 + 1
    let mut new_map_h =
        (window_size.h.saturating_sub(new_ui_px_h) / map_font.glyph_height()).max(17);
    if window_size.h % map_font.glyph_height() > 0 {
        // Fill to the edge of the screen.
        new_map_h += 1;
    }
    if new_map_h & 1 == 0 {
        // Ensure a single center tile exists using an odd number of tiles.
        new_map_h += 1;
    }

    let new_map_size = Size {
        w: new_map_w,
        h: new_map_h,
    };

    if !grids.is_empty() {
        grids[MAP_GRID].resize(new_map_size);
        grids[UI_GRID].resize(new_ui_size);
    } else {
        grids.push(CharGrid::new(new_map_size, fonts, DEFAULT_MAP_FONT));
        grids.push(CharGrid::new(new_ui_size, fonts, 0));
        grids[MAP_GRID].view.clear_color = None;
        grids[UI_GRID].view.clear_color = Some(color::BLACK);
    }

    grids[MAP_GRID].view_centered(
        fonts,
        Position { x: 0, y: 0 },
        Size {
            w: window_size.w,
            h: window_size.h.saturating_sub(new_ui_px_h).max(1),
        },
    );

    grids[UI_GRID].view.pos = Position {
        x: 0,
        y: window_size.h.saturating_sub(new_ui_px_h) as i32,
    };
    grids[UI_GRID].view.size = Size {
        w: window_size.w,
        h: new_ui_px_h,
    };
}
