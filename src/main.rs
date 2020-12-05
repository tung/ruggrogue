mod map;

use piston::input::{Button, Key};
use shipyard::{
    EntitiesViewMut, EntityId, Get, IntoIter, UniqueView, UniqueViewMut, View, ViewMut, World,
};
use std::collections::HashSet;
use std::path::PathBuf;

use map::{Map, Tile};
use ruggle::{CharGrid, FovShape, InputBuffer, InputEvent, KeyMods, RunSettings};

struct FieldOfView(HashSet<(i32, i32)>);

struct Position {
    x: i32,
    y: i32,
}

struct Renderable {
    ch: char,
    fg: [f32; 4],
    bg: [f32; 4],
}

struct Player;

struct PlayerId(EntityId);

fn get_player_position(player: &UniqueView<PlayerId>, positions: &View<Position>) -> (i32, i32) {
    let player_pos = positions.get(player.0);

    (player_pos.x, player_pos.y)
}

fn calculate_player_fov(
    map: UniqueView<Map>,
    player: UniqueView<PlayerId>,
    mut fov: UniqueViewMut<FieldOfView>,
    positions: View<Position>,
) {
    fov.0.clear();
    for (x, y, symmetric) in ruggle::field_of_view(
        &*map,
        get_player_position(&player, &positions),
        8,
        FovShape::CirclePlus,
    ) {
        if symmetric || matches!(map.get_tile(x as u32, y as u32), &Tile::Wall) {
            fov.0.insert((x, y));
        }
    }
}

fn try_move_player(world: &World, dx: i32, dy: i32) -> bool {
    world.run(
        |map: UniqueView<Map>, players: View<Player>, mut positions: ViewMut<Position>| {
            let mut moved = false;

            for (_, pos) in (&players, &mut positions).iter() {
                let new_x = pos.x + dx;
                let new_y = pos.y + dy;

                if new_x >= 0
                    && new_y >= 0
                    && !matches!(map.get_tile(new_x as u32, new_y as u32), &Tile::Wall)
                {
                    if dx != 0 && new_x < 80 {
                        pos.x = new_x;
                        moved = true;
                    }
                    if dy != 0 && new_y < 36 {
                        pos.y = new_y;
                        moved = true;
                    }
                }
            }

            moved
        },
    )
}

fn player_input(world: &World, inputs: &mut InputBuffer) -> bool {
    let mut time_passed = false;

    inputs.prepare_input();
    if let Some(InputEvent::Press(Button::Keyboard(key))) = inputs.get_input() {
        let dist = if inputs.get_mods(KeyMods::SHIFT) {
            2
        } else if inputs.get_mods(KeyMods::CTRL) {
            3
        } else if inputs.get_mods(KeyMods::ALT) {
            5
        } else {
            1
        };
        let mut moved = false;

        match key {
            Key::Up => moved = try_move_player(world, 0, -dist),
            Key::Down => moved = try_move_player(world, 0, dist),
            Key::Left => moved = try_move_player(world, -dist, 0),
            Key::Right => moved = try_move_player(world, dist, 0),
            _ => {}
        }

        if moved {
            time_passed = true;
            world.run(calculate_player_fov);
        }
    }

    time_passed
}

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

fn draw_map(world: &World, grid: &mut CharGrid) {
    world.run(
        |map: UniqueView<Map>,
         player: UniqueView<PlayerId>,
         fov: UniqueView<FieldOfView>,
         positions: View<Position>| {
            let (x, y) = get_player_position(&player, &positions);

            for (tx, ty, tile) in map.iter_bounds(x - 40, y - 18, x + 39, y + 17) {
                if let Some((ch, color)) = tile {
                    let color = if fov.0.contains(&(tx, ty)) {
                        color
                    } else {
                        let v = (0.3 * color[0] + 0.59 * color[1] + 0.11 * color[2]) / 2.;
                        [v, v, v, color[3]]
                    };

                    grid.put_color(
                        [(tx - x + 40) as u32, (ty - y + 18) as u32],
                        Some(color),
                        None,
                        ch,
                    );
                }
            }
        },
    );
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
    world.run(|mut map: UniqueViewMut<Map>| map.generate());

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
