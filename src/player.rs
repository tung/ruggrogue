use piston::input::{Button, Key};
use shipyard::{Get, IntoIter, UniqueView, UniqueViewMut, View, ViewMut, World};

use crate::{
    components::{FieldOfView, Player, PlayerId, Position},
    map::{Map, Tile},
};
use ruggle::{FovShape, InputBuffer, InputEvent};

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
        time_passed = match key {
            Key::H | Key::NumPad4 | Key::Left => try_move_player(world, -1, 0),
            Key::J | Key::NumPad2 | Key::Down => try_move_player(world, 0, 1),
            Key::K | Key::NumPad8 | Key::Up => try_move_player(world, 0, -1),
            Key::L | Key::NumPad6 | Key::Right => try_move_player(world, 1, 0),
            Key::Y | Key::NumPad7 => try_move_player(world, -1, -1),
            Key::U | Key::NumPad9 => try_move_player(world, 1, -1),
            Key::B | Key::NumPad1 => try_move_player(world, -1, 1),
            Key::N | Key::NumPad3 => try_move_player(world, 1, 1),
            _ => false,
        };

        if time_passed {
            world.run(calculate_player_fov);
        }
    }

    time_passed
}
