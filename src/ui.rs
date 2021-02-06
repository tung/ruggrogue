use shipyard::{Get, UniqueView, View, World};

use crate::{components::CombatStats, message::Messages, player::PlayerId};
use ruggle::CharGrid;

pub mod color {
    pub const WHITE: [f32; 4] = [1.; 4];
    pub const BLACK: [f32; 4] = [0., 0., 0., 1.];
    pub const RED: [f32; 4] = [1., 0., 0., 1.];
    pub const YELLOW: [f32; 4] = [1., 1., 0., 1.];
    pub const MAGENTA: [f32; 4] = [1., 0., 1., 1.];

    pub const SELECTED_BG: [f32; 4] = [0., 0.5, 1., 1.];
}

pub fn recolor(c: [f32; 4], active: bool) -> [f32; 4] {
    if active {
        c
    } else {
        // Dim and desaturate.
        let gray = c[0] * 0.3 + c[1] * 0.59 + c[2] * 0.11;

        [
            (c[0] + gray) * 0.3,
            (c[1] + gray) * 0.3,
            (c[2] + gray) * 0.3,
            c[3],
        ]
    }
}

pub const HUD_LINES: i32 = 5;

fn draw_player_hp(world: &World, grid: &mut CharGrid, active: bool, y: i32) {
    let (hp, max_hp) = world.run(
        |player: UniqueView<PlayerId>, combat_stats: View<CombatStats>| {
            let player_stats = combat_stats.get(player.0);

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

pub fn draw_ui(world: &World, grid: &mut CharGrid, active: bool) {
    let [w, h] = grid.size_cells();
    let y = h - HUD_LINES;
    let fg = Some(recolor(color::WHITE, active));

    for x in 0..w {
        grid.put_color([x, y], fg, None, '─');
    }

    draw_player_hp(world, grid, active, y);
    draw_messages(world, grid, active, y + 1, h);
}
