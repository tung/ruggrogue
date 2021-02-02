use shipyard::{Get, UniqueView, View, World};

use crate::{components::CombatStats, message::Messages, player::PlayerId};
use ruggle::CharGrid;

pub mod color {
    pub const WHITE: [f32; 4] = [1.; 4];
    pub const BLACK: [f32; 4] = [0., 0., 0., 1.];
    pub const RED: [f32; 4] = [1., 0., 0., 1.];
    pub const YELLOW: [f32; 4] = [1., 1., 0., 1.];

    pub const SELECTED_BG: [f32; 4] = [0., 0.5, 1., 1.];
}

pub fn recolor(c: [f32; 4], active: bool) -> [f32; 4] {
    if active {
        c
    } else {
        // Dim and desaturate.
        let gray = c[0] * 0.3 + c[1] * 0.59 + c[2] * 0.11;

        [(c[0] + gray) * 0.3, (c[1] + gray) * 0.3, (c[2] + gray) * 0.3, c[3]]
    }
}

pub const HUD_LINES: i32 = 5;

fn draw_bar(grid: &mut CharGrid, active: bool, y: i32, min_x: i32, max_x: i32, val: i32, max_val: i32) {
    let bar_fg = Some(recolor(color::RED, active));
    let bar_bg = None;

    let max_width = max_x - min_x + 1;
    let mut width_2 = val * max_width * 2 / max_val;

    if width_2 < 1 && val > 0 {
        width_2 = 1;
    }
    if width_2 > max_width * 2 {
        width_2 = max_width * 2;
    }

    let mut dx_2 = 0;

    while dx_2 + 2 <= width_2 {
        grid.put_color([min_x + dx_2 / 2, y], bar_fg, bar_bg, '█');
        dx_2 += 2;
    }

    if dx_2 < width_2 {
        grid.put_color([min_x + dx_2 / 2, y], bar_fg, bar_bg, '▌');
        dx_2 += 2;
    }

    while dx_2 < max_width * 2 {
        grid.put_color([min_x + dx_2 / 2, y], bar_fg, bar_bg, '░');
        dx_2 += 2;
    }
}

fn draw_player_hp(world: &World, grid: &mut CharGrid, active: bool, y: i32) {
    let (hp, max_hp) = world.run(
        |player: UniqueView<PlayerId>, combat_stats: View<CombatStats>| {
            let player_stats = combat_stats.get(player.0);

            (player_stats.hp, player_stats.max_hp)
        },
    );
    let hp_string = format!(" HP: {} / {} ", hp, max_hp);
    let hp_bar_begin = hp_string.len() as i32 + 6;
    let hp_bar_end = std::cmp::max(hp_bar_begin + 1, grid.size_cells()[0] - 4);

    grid.print_color([3, y], Some(recolor(color::YELLOW, active)), None, &hp_string);
    draw_bar(grid, active, y, hp_bar_begin, hp_bar_end, hp, max_hp);
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
