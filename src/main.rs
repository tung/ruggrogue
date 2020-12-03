mod map;

use piston::input::{Button, Key};
use shipyard::{EntitiesViewMut, EntityId, Get, IntoIter, View, ViewMut, World};
use std::collections::HashSet;
use std::path::PathBuf;

use map::{Map, Tile};
use ruggle::{FovShape, InputBuffer, InputEvent, KeyMods, RunSettings};

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

struct LeftMover;

fn get_player_position(world: &World, player: &EntityId) -> (i32, i32) {
    world.run(|positions: View<Position>| {
        let p = positions.get(*player);

        (p.x, p.y)
    })
}

fn try_move_player(world: &World, dx: i32, dy: i32) -> bool {
    world.run(|players: View<Player>, mut positions: ViewMut<Position>| {
        let mut moved = false;

        for (_, pos) in (&players, &mut positions).iter() {
            let new_x = pos.x + dx;
            let new_y = pos.y + dy;

            if dx != 0 && new_x >= 0 && new_x < 80 {
                pos.x = new_x;
                moved = true;
            }
            if dy != 0 && new_y >= 0 && new_y < 36 {
                pos.y = new_y;
                moved = true;
            }
        }

        moved
    })
}

fn player_input(
    world: &World,
    inputs: &mut InputBuffer,
    player: &EntityId,
    map: &Map,
    fov: &mut HashSet<(i32, i32)>,
) -> bool {
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

            fov.clear();
            for (x, y, symmetric) in ruggle::field_of_view(
                map,
                get_player_position(world, player),
                8,
                FovShape::CirclePlus,
            ) {
                if symmetric || matches!(map.get_tile(x as u32, y as u32), &Tile::Wall) {
                    fov.insert((x, y));
                }
            }
        }
    }

    time_passed
}

fn left_move(left_movers: View<LeftMover>, mut positions: ViewMut<Position>) {
    for (_, pos) in (&left_movers, &mut positions).iter() {
        pos.x -= 1;
        if pos.x < 0 {
            pos.x = 79;
        }
    }
}

fn main() {
    let mut map = Map::new(80, 36);
    let mut fov = HashSet::new();

    map.generate();

    let world = World::new();

    // Add player.
    let player = world.run(
        |mut entities: EntitiesViewMut,
         mut positions: ViewMut<Position>,
         mut renderables: ViewMut<Renderable>,
         mut players: ViewMut<Player>| {
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
        },
    );

    // Calculate initial field of view.
    for (x, y, symmetric) in ruggle::field_of_view(
        &map,
        get_player_position(&world, &player),
        8,
        FovShape::CirclePlus,
    ) {
        if symmetric || matches!(map.get_tile(x as u32, y as u32), &Tile::Wall) {
            fov.insert((x, y));
        }
    }

    // Add creatures.
    world.run(
        |mut entities: EntitiesViewMut,
         mut positions: ViewMut<Position>,
         mut renderables: ViewMut<Renderable>,
         mut left_movers: ViewMut<LeftMover>| {
            for i in 0..10 {
                entities.add_entity(
                    (&mut positions, &mut renderables, &mut left_movers),
                    (
                        Position { x: i * 7, y: 15 },
                        Renderable {
                            ch: 'g',
                            fg: [1., 0., 0., 1.],
                            bg: [0., 0., 0., 1.],
                        },
                        LeftMover {},
                    ),
                );
            }
        },
    );

    let settings = RunSettings {
        title: "Ruggle".to_string(),
        grid_size: [80, 36],
        font_path: PathBuf::from("assets/gohufont-uni-14.ttf"),
        font_size: 14.0,
        min_fps: 30,
        max_fps: 60,
    };

    ruggle::run(settings, |mut inputs, grid| {
        if player_input(&world, &mut inputs, &player, &map, &mut fov) {
            world.run(left_move);
        }

        let (x, y) = get_player_position(&world, &player);

        grid.clear();

        // Draw the map.
        for (tx, ty, tile) in map.iter_bounds(x - 40, y - 18, x + 39, y + 17) {
            if let Some((ch, color)) = tile {
                let color = if fov.contains(&(tx, ty)) {
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

        // Draw renderables.
        world.run(|positions: View<Position>, renderables: View<Renderable>| {
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
        });

        false
    });
}
