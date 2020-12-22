use shipyard::{Get, UniqueView, View, World};

use crate::{
    components::{CombatStats, PlayerId},
    message::Messages,
};
use ruggle::CharGrid;

fn draw_bar(grid: &mut CharGrid, y: i32, min_x: i32, max_x: i32, val: i32, max_val: i32) {
    const BAR_FG: Option<[f32; 4]> = Some([1., 0., 0., 1.]);
    const BAR_BG: Option<[f32; 4]> = None;

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
        grid.put_color([min_x + dx_2 / 2, y], BAR_FG, BAR_BG, '█');
        dx_2 += 2;
    }

    if dx_2 < width_2 {
        grid.put_color([min_x + dx_2 / 2, y], BAR_FG, BAR_BG, '▌');
        dx_2 += 2;
    }

    while dx_2 < max_width * 2 {
        grid.put_color([min_x + dx_2 / 2, y], BAR_FG, BAR_BG, '░');
        dx_2 += 2;
    }
}

fn draw_player_hp(world: &World, grid: &mut CharGrid, y: i32) {
    let (hp, max_hp) = world.run(
        |player: UniqueView<PlayerId>, combat_stats: View<CombatStats>| {
            let player_stats = combat_stats.get(player.0);

            (player_stats.hp, player_stats.max_hp)
        },
    );

    grid.print_color(
        [12, y],
        Some([1., 1., 0., 1.]),
        None,
        &format!(" HP: {} / {} ", hp, max_hp),
    );
    draw_bar(grid, y, 28, 78, hp, max_hp);
}

fn draw_messages(world: &World, grid: &mut CharGrid, min_y: i32, max_y: i32) {
    world.run(|messages: UniqueView<Messages>| {
        for (y, message) in (min_y..=max_y).zip(messages.rev_iter()) {
            grid.put([0, y], '>');
            grid.print([2, y], message);
        }
    });
}

pub fn draw_ui(world: &World, grid: &mut CharGrid) {
    for x in 0..80 {
        grid.put([x, 31], '─');
    }

    draw_player_hp(world, grid, 31);
    draw_messages(world, grid, 32, 35);
}
