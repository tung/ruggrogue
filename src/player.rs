use piston::input::{Button, Key};
use shipyard::{EntityId, IntoIter, Shiperator, UniqueViewMut, View, ViewMut, World};

use crate::{
    components::{CombatStats, FieldOfView, Player, Position},
    damage::MeleeQueue,
    map::Map,
};
use ruggle::{InputBuffer, InputEvent, PathableMap};

pub struct PlayerId(pub EntityId);

pub struct PlayerAlive(pub bool);

pub enum PlayerInputResult {
    NoResult,
    TurnDone,
    ShowExitPrompt,
}

pub fn try_move_player(world: &World, dx: i32, dy: i32) -> PlayerInputResult {
    world.run(
        |mut map: UniqueViewMut<Map>,
         mut melee_queue: UniqueViewMut<MeleeQueue>,
         combat_stats: View<CombatStats>,
         mut fovs: ViewMut<FieldOfView>,
         players: View<Player>,
         mut positions: ViewMut<Position>| {
            let mut moved = false;

            for (id, (_, pos, fov)) in (&players, &mut positions, &mut fovs).iter().with_id() {
                let new_x = pos.x + dx;
                let new_y = pos.y + dy;

                if new_x >= 0 && new_y >= 0 && new_x < map.width && new_y < map.height {
                    let melee_target = map
                        .iter_entities_at(new_x, new_y)
                        .find(|e| combat_stats.contains(*e));

                    if let Some(melee_target) = melee_target {
                        melee_queue.push_back(id, melee_target);
                        moved = true;
                    } else if !map.is_blocked(new_x, new_y) {
                        map.move_entity(id, pos.into(), (new_x, new_y), false);
                        pos.x = new_x;
                        pos.y = new_y;
                        fov.dirty = true;
                        moved = true;
                    }
                }
            }

            if moved {
                PlayerInputResult::TurnDone
            } else {
                PlayerInputResult::NoResult
            }
        },
    )
}

pub fn player_input(world: &World, inputs: &mut InputBuffer) -> PlayerInputResult {
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
            Key::Period | Key::NumPad5 => PlayerInputResult::TurnDone,
            Key::Escape => PlayerInputResult::ShowExitPrompt,
            _ => PlayerInputResult::NoResult,
        }
    } else {
        PlayerInputResult::NoResult
    }
}

pub fn player_is_dead_input(inputs: &mut InputBuffer) -> bool {
    inputs.prepare_input();

    matches!(
        inputs.get_input(),
        Some(InputEvent::Press(Button::Keyboard(Key::Space)))
    )
}
