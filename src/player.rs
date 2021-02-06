use piston::input::{Button, Key};
use shipyard::{
    EntitiesView, EntityId, Get, IntoIter, Shiperator, UniqueView, UniqueViewMut, View, ViewMut,
    World,
};

use crate::{
    components::{CombatStats, FieldOfView, Inventory, Name, Player, Position, RenderOnFloor},
    damage::MeleeQueue,
    map::Map,
    message::Messages,
};
use ruggle::{InputBuffer, InputEvent, PathableMap};

pub struct PlayerId(pub EntityId);

pub struct PlayerAlive(pub bool);

pub enum PlayerInputResult {
    NoResult,
    TurnDone,
    ShowExitPrompt,
    ShowPickUpMenu,
    ShowInventory,
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

pub fn player_pick_up_item(world: &World, item_id: EntityId) {
    world.run(
        |mut map: UniqueViewMut<Map>,
         mut msgs: UniqueViewMut<Messages>,
         player: UniqueView<PlayerId>,
         mut inventories: ViewMut<Inventory>,
         names: View<Name>,
         mut positions: ViewMut<Position>,
         mut render_on_floors: ViewMut<RenderOnFloor>| {
            map.remove_entity(item_id, positions.get(item_id).into(), false);
            positions.remove(item_id);
            render_on_floors.remove(item_id);
            (&mut inventories).get(player.0).items.insert(0, item_id);
            msgs.add(format!(
                "{} picks up {}.",
                names.get(player.0).0,
                names.get(item_id).0
            ));
        },
    );
}

pub fn player_drop_item(world: &World, item_id: EntityId) {
    world.run(
        |mut map: UniqueViewMut<Map>,
         mut msgs: UniqueViewMut<Messages>,
         player: UniqueView<PlayerId>,
         entities: EntitiesView,
         mut inventories: ViewMut<Inventory>,
         names: View<Name>,
         mut positions: ViewMut<Position>,
         mut render_on_floors: ViewMut<RenderOnFloor>| {
            let player_inv = (&mut inventories).get(player.0);

            if let Some(inv_pos) = player_inv.items.iter().position(|id| *id == item_id) {
                player_inv.items.remove(inv_pos);
            }

            let item_pos: (i32, i32) = positions.get(player.0).into();

            entities.add_component(
                (&mut positions, &mut render_on_floors),
                (
                    Position {
                        x: item_pos.0,
                        y: item_pos.1,
                    },
                    RenderOnFloor {},
                ),
                item_id,
            );
            map.place_entity(item_id, item_pos, false);
            msgs.add(format!(
                "{} drops {}.",
                names.get(player.0).0,
                names.get(item_id).0
            ));
        },
    );
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
            Key::Period | Key::NumPad5 | Key::Space => PlayerInputResult::TurnDone,
            Key::Escape => PlayerInputResult::ShowExitPrompt,
            Key::Comma | Key::G => PlayerInputResult::ShowPickUpMenu,
            Key::I | Key::Return => PlayerInputResult::ShowInventory,
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
