use shipyard::{Get, UniqueView, View, World};

use crate::{components::CombatStats, map::Map, message::Messages, player::PlayerId};
use ruggle::{
    util::{Color, Size},
    CharGrid,
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

pub const HUD_LINES: i32 = 5;

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

    let hp_bar_length = grid.size_cells().w - x - 2;
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
    let Size { w, h } = grid.size_cells();
    let y = h - HUD_LINES;
    let fg = recolor(color::WHITE, active);

    for x in 0..w {
        grid.put_color((x, y), fg, None, '─');
    }

    draw_status_line(world, grid, active, y);

    if let Some(prompt) = prompt {
        grid.print_color((2, y + 1), fg, None, prompt);
        draw_messages(world, grid, false, y + 2, h - 1);
    } else {
        draw_messages(world, grid, active, y + 1, h);
    }
}
