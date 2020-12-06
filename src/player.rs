use piston::input::{Button, Key};
use shipyard::{IntoIter, UniqueView, View, ViewMut, World};

use crate::{
    components::{FieldOfView, Player, Position},
    map::{Map, Tile},
};
use ruggle::{InputBuffer, InputEvent};

pub fn try_move_player(world: &World, dx: i32, dy: i32) -> bool {
    world.run(
        |map: UniqueView<Map>,
         players: View<Player>,
         mut positions: ViewMut<Position>,
         mut fovs: ViewMut<FieldOfView>| {
            let mut moved = false;

            for (_, pos, fov) in (&players, &mut positions, &mut fovs).iter() {
                let new_x = pos.x + dx;
                let new_y = pos.y + dy;

                if new_x >= 0
                    && new_y >= 0
                    && new_x < map.width
                    && new_y < map.height
                    && !matches!(map.get_tile(new_x, new_y), &Tile::Wall)
                {
                    pos.x = new_x;
                    pos.y = new_y;
                    fov.dirty = true;
                    moved = true;
                }
            }

            moved
        },
    )
}

pub fn player_input(world: &World, inputs: &mut InputBuffer) -> bool {
    inputs.prepare_input();

    if let Some(InputEvent::Press(Button::Keyboard(key))) = inputs.get_input() {
        match key {
            Key::H | Key::NumPad4 | Key::Left => try_move_player(world, -1, 0),
            Key::J | Key::NumPad2 | Key::Down => try_move_player(world, 0, 1),
            Key::K | Key::NumPad8 | Key::Up => try_move_player(world, 0, -1),
            Key::L | Key::NumPad6 | Key::Right => try_move_player(world, 1, 0),
            Key::Y | Key::NumPad7 => try_move_player(world, -1, -1),
            Key::U | Key::NumPad9 => try_move_player(world, 1, -1),
            Key::B | Key::NumPad1 => try_move_player(world, -1, 1),
            Key::N | Key::NumPad3 => try_move_player(world, 1, 1),
            _ => false,
        }
    } else {
        false
    }
}
