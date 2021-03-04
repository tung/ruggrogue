use shipyard::{Get, UniqueView, View, World};

use crate::{components::CombatStats, message::Messages, player::PlayerId};
use ruggle::CharGrid;

pub mod color {
    pub const WHITE: [u8; 3] = [255; 3];
    pub const BLACK: [u8; 3] = [0; 3];
    pub const RED: [u8; 3] = [255, 0, 0];
    pub const BLUE: [u8; 3] = [0, 0, 255];
    pub const YELLOW: [u8; 3] = [255, 255, 0];
    pub const MAGENTA: [u8; 3] = [255, 0, 255];
    pub const CYAN: [u8; 3] = [0, 255, 255];
    pub const ORANGE: [u8; 3] = [255, 166, 0];
    pub const PURPLE: [u8; 3] = [128, 0, 128];
    pub const PINK: [u8; 3] = [255, 191, 204];

    pub const SELECTED_BG: [u8; 3] = [0, 128, 255];
}

pub fn recolor(c: [u8; 3], active: bool) -> [u8; 3] {
    if active {
        c
    } else {
        // Dim and desaturate.
        let (red, green, blue) = (c[0] as i32, c[1] as i32, c[2] as i32);
        let gray = (red * 30 + green * 59 + blue * 11) / 100;

        [
            ((red + gray) * 3 / 10) as u8,
            ((green + gray) * 3 / 10) as u8,
            ((blue + gray) * 3 / 10) as u8,
        ]
    }
}

pub const HUD_LINES: i32 = 5;

fn draw_player_hp(world: &World, grid: &mut CharGrid, active: bool, y: i32) {
    let (hp, max_hp) = world.run(
        |player_id: UniqueView<PlayerId>, combat_stats: View<CombatStats>| {
            let player_stats = combat_stats.get(player_id.0);

            (player_stats.hp, player_stats.max_hp)
        },
    );
    let hp_string = format!(" HP: {} / {} ", hp, max_hp);
    let hp_bar_begin = hp_string.len() as i32 + 6;
    let hp_bar_length = grid.size_cells()[0] - 3 - hp_bar_begin;

    grid.print_color(
        [3, y],
        Some(recolor(color::YELLOW, active)),
        None,
        &hp_string,
    );
    grid.draw_bar(
        false,
        [hp_bar_begin, y],
        hp_bar_length,
        0,
        hp,
        max_hp,
        Some(recolor(color::RED, active)),
        None,
    );
}

fn draw_messages(world: &World, grid: &mut CharGrid, active: bool, min_y: i32, max_y: i32) {
    world.run(|messages: UniqueView<Messages>| {
        let fg = Some(recolor(color::WHITE, active));

        for (y, message) in (min_y..=max_y).zip(messages.rev_iter()) {
            grid.put_color([0, y], fg, None, '>');
            grid.print_color([2, y], fg, None, message);
        }
    });
}

pub fn draw_ui(world: &World, grid: &mut CharGrid, active: bool, prompt: Option<&str>) {
    let [w, h] = grid.size_cells();
    let y = h - HUD_LINES;
    let fg = Some(recolor(color::WHITE, active));

    for x in 0..w {
        grid.put_color([x, y], fg, None, '─');
    }

    draw_player_hp(world, grid, active, y);

    if let Some(prompt) = prompt {
        grid.print_color([2, y + 1], fg, None, prompt);
        draw_messages(world, grid, false, y + 2, h - 1);
    } else {
        draw_messages(world, grid, active, y + 1, h);
    }
}
