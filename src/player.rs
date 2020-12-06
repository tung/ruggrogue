use piston::input::{Button, Key};
use shipyard::{Get, IntoIter, UniqueView, UniqueViewMut, View, ViewMut, World};

use crate::{
    components::{FieldOfView, Player, PlayerId, Position},
    map::{Map, Tile},
};
use ruggle::{FovShape, InputBuffer, InputEvent, KeyMods};

pub fn get_player_position(
    player: &UniqueView<PlayerId>,
    positions: &View<Position>,
) -> (i32, i32) {
    let player_pos = positions.get(player.0);

    (player_pos.x, player_pos.y)
}

pub fn calculate_player_fov(
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
        if symmetric || matches!(map.get_tile(x, y), &Tile::Wall) {
            fov.0.insert((x, y));
        }
    }
}

pub fn try_move_player(world: &World, dx: i32, dy: i32) -> bool {
    world.run(
        |map: UniqueView<Map>, players: View<Player>, mut positions: ViewMut<Position>| {
            let mut moved = false;

            for (_, pos) in (&players, &mut positions).iter() {
                let new_x = pos.x + dx;
                let new_y = pos.y + dy;

                if new_x >= 0
                    && new_y >= 0
                    && new_x < map.width
                    && new_y < map.height
                    && !matches!(map.get_tile(new_x, new_y), &Tile::Wall)
                {
                    if dx != 0 {
                        pos.x = new_x;
                        moved = true;
                    }
                    if dy != 0 {
                        pos.y = new_y;
                        moved = true;
                    }
                }
            }

            moved
        },
    )
}

pub fn player_input(world: &World, inputs: &mut InputBuffer) -> bool {
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
