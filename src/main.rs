mod components;
mod map;
mod player;

use shipyard::{EntitiesViewMut, EntityId, IntoIter, UniqueView, View, ViewMut, World};
use std::collections::HashSet;
use std::path::PathBuf;

use crate::{
    components::{FieldOfView, Player, PlayerId, Position, Renderable},
    map::{draw_map, Map},
    player::{calculate_player_fov, get_player_position, player_input},
};
use ruggle::{CharGrid, RunSettings};

fn spawn_player(
    mut entities: EntitiesViewMut,
    mut positions: ViewMut<Position>,
    mut renderables: ViewMut<Renderable>,
    mut players: ViewMut<Player>,
) -> EntityId {
    entities.add_entity(
        (&mut positions, &mut renderables, &mut players),
        (
            Position { x: 40, y: 18 },
            Renderable {
                ch: '@',
                fg: [1., 1., 0., 1.],
                bg: [0., 0., 0., 1.],
            },
            Player {},
        ),
    )
}

fn draw_renderables(world: &World, grid: &mut CharGrid) {
    world.run(
        |player: UniqueView<PlayerId>, positions: View<Position>, renderables: View<Renderable>| {
            let (x, y) = get_player_position(&player, &positions);

            for (pos, render) in (&positions, &renderables).iter() {
                let rx = pos.x - x + 40;
                let ry = pos.y - y + 18;

                if rx >= 0 && ry >= 0 {
                    grid.put_color(
                        [rx as u32, ry as u32],
                        Some(render.fg),
                        Some(render.bg),
                        render.ch,
                    );
                }
            }
        },
    );
}

fn main() {
    let world = World::new();

    world.add_unique(Map::new(80, 36));
    world.run(map::generate_test_pattern);

    world.add_unique(PlayerId(world.run(spawn_player)));

    world.add_unique(FieldOfView(HashSet::new()));
    world.run(calculate_player_fov);

    let settings = RunSettings {
        title: "Ruggle".to_string(),
        grid_size: [80, 36],
        font_path: PathBuf::from("assets/gohufont-uni-14.ttf"),
        font_size: 14.0,
        min_fps: 30,
        max_fps: 60,
        start_inactive: true,
    };

    ruggle::run(settings, |mut inputs, mut grid| {
        player_input(&world, &mut inputs);

        grid.clear();
        draw_map(&world, &mut grid);
        draw_renderables(&world, &mut grid);

        false
    });
}
