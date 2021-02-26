use piston::input::{Button, Key};
use shipyard::{
    EntityId, Get, IntoIter, Shiperator, UniqueView, UniqueViewMut, View, ViewMut, World,
};

use crate::{
    components::{CombatStats, FieldOfView, Item, Monster, Name, Player, Position},
    damage,
    gamekey::GameKey,
    item,
    map::Map,
    message::Messages,
};
use ruggle::{InputBuffer, InputEvent, KeyMods, PathableMap};

pub struct PlayerId(pub EntityId);

pub struct PlayerAlive(pub bool);

#[derive(Clone, Copy, PartialEq)]
enum AutoRunWallSide {
    Neither,
    Left,
    Right,
}

#[derive(Clone, Copy)]
enum AutoRunType {
    Corridor,
    Straight { expect_wall: AutoRunWallSide },
}

pub struct AutoRun {
    limit: i32,
    dir: (i32, i32),
    run_type: AutoRunType,
}

pub enum PlayerInputResult {
    NoResult,
    TurnDone,
    ShowExitPrompt,
    ShowPickUpMenu,
    ShowInventory,
}

pub fn player_is_auto_running(player_id: UniqueView<PlayerId>, players: View<Player>) -> bool {
    players.get(player_id.0).auto_run.is_some()
}

pub fn player_stop_auto_run(player_id: UniqueView<PlayerId>, mut players: ViewMut<Player>) {
    (&mut players).get(player_id.0).auto_run = None;
}

pub fn player_sees_foes(
    map: UniqueView<Map>,
    player_id: UniqueView<PlayerId>,
    fovs: View<FieldOfView>,
    monsters: View<Monster>,
) -> bool {
    fovs.get(player_id.0)
        .iter()
        .any(|(x, y)| map.iter_entities_at(x, y).any(|id| monsters.contains(id)))
}

pub fn can_see_player(world: &World, who: EntityId) -> bool {
    let (player_id, fovs, positions) =
        world.borrow::<(UniqueView<PlayerId>, View<FieldOfView>, View<Position>)>();

    if let Ok(fov) = fovs.try_get(who) {
        let player_pos = positions.get(player_id.0);

        fov.get(player_pos.into())
    } else {
        false
    }
}

/// Calculate a 2-by-2 matrix to rotate any `(dx, dy)` to only face `(+dx, 0)` or `(+dx, +dy)`.
///
/// Returns `real_x_from_x`, `real_x_from_y`, `real_y_from_x` and `real_y_from_y` in that order.
fn rotate_view(dx: i32, dy: i32) -> (i32, i32, i32, i32) {
    match (dx, dy) {
        (1, 0) => (1, 0, 0, 1),
        (1, 1) => (1, 0, 0, 1),
        (0, 1) => (0, -1, 1, 0),
        (-1, 1) => (0, -1, 1, 0),
        (-1, 0) => (-1, 0, 0, -1),
        (-1, -1) => (-1, 0, 0, -1),
        (0, -1) => (0, 1, -1, 0),
        (1, -1) => (0, 1, -1, 0),
        (_, _) => (1, 0, 0, 1),
    }
}

/// Check newly-adjacent tiles to the player for things worth stopping for during auto run.
fn player_check_frontier(
    map: UniqueView<Map>,
    player_id: UniqueView<PlayerId>,
    items: View<Item>,
    players: View<Player>,
    positions: View<Position>,
) -> bool {
    let player = players.get(player_id.0);
    let (auto_run_dx, auto_run_dy) = player.auto_run.as_ref().unwrap().dir;
    let player_pos = positions.get(player_id.0);
    let (real_x_from_x, real_x_from_y, real_y_from_x, real_y_from_y) =
        rotate_view(auto_run_dx, auto_run_dy);
    let real_x = |dx, dy| player_pos.x + dx * real_x_from_x + dy * real_x_from_y;
    let real_y = |dx, dy| player_pos.y + dx * real_y_from_x + dy * real_y_from_y;
    let stop_for = |dx, dy| {
        // Just stop for items for now.
        map.iter_entities_at(real_x(dx, dy), real_y(dx, dy))
            .any(|id| items.contains(id))
    };

    if auto_run_dx != 0 && auto_run_dy != 0 {
        // There are five newly-adjacent tiles after a diagonal move.
        stop_for(-1, 1) || stop_for(0, 1) || stop_for(1, 1) || stop_for(1, 0) || stop_for(1, -1)
    } else {
        // There are three newly-adjacent tiles after a cardinal move.
        stop_for(1, 1) || stop_for(1, 0) || stop_for(1, -1)
    }
}

/// Check if the current player position and desired run direction should perform an auto run along
/// a corridor.
///
/// Returns the new direction, or `None` if the corridor seems to have ended.
fn auto_run_corridor_check(world: &World, run_dx: i32, run_dy: i32) -> Option<(i32, i32)> {
    let (player_x, player_y) = world.run(
        |player_id: UniqueView<PlayerId>, positions: View<Position>| {
            let pos = positions.get(player_id.0);
            (pos.x, pos.y)
        },
    );
    let (real_x_from_x, real_x_from_y, real_y_from_x, real_y_from_y) = rotate_view(run_dx, run_dy);
    let real_x = |dx, dy| player_x + dx * real_x_from_x + dy * real_x_from_y;
    let real_y = |dx, dy| player_y + dx * real_y_from_x + dy * real_y_from_y;

    let (down_left, down, down_right, left, right, up_left, up, up_right) =
        world.run(|map: UniqueView<Map>| {
            let check_wall = |dx, dy| map.wall_or_oob(real_x(dx, dy), real_y(dx, dy));
            (
                check_wall(-1, -1),
                check_wall(0, -1),
                check_wall(1, -1),
                check_wall(-1, 0),
                check_wall(1, 0),
                check_wall(-1, 1),
                check_wall(0, 1),
                check_wall(1, 1),
            )
        });

    // These are needed to decide if we should auto run into the corner of a corridor.
    let (
        up_up_left,
        up_up,
        up_up_right,
        up_right_right,
        right_right,
        down_right_right,
        down_down_left,
        down_down,
        down_down_right,
    ) = world.run(
        |map: UniqueView<Map>, player_id: UniqueView<PlayerId>, fovs: View<FieldOfView>| {
            let player_fov = fovs.get(player_id.0);
            let check_unknown_or_wall = |dx, dy| {
                !player_fov.get((real_x(dx, dy), real_y(dx, dy)))
                    || map.wall_or_oob(real_x(dx, dy), real_y(dx, dy))
            };
            (
                check_unknown_or_wall(-1, 2),
                check_unknown_or_wall(0, 2),
                check_unknown_or_wall(1, 2),
                check_unknown_or_wall(2, 1),
                check_unknown_or_wall(2, 0),
                check_unknown_or_wall(2, -1),
                check_unknown_or_wall(-1, -2),
                check_unknown_or_wall(0, -2),
                check_unknown_or_wall(1, -2),
            )
        },
    );

    if run_dx != 0 && run_dy != 0 {
        // Arrived here diagonally (moved up-right post-transform).

        // Check for a single space bordered with walls to advance forward or turn up to 90 degrees
        // left or right.
        //
        // ```
        // 1##  #2#  ##3  ###  ###
        // #@#  .@#  .@#  .@4  .@#
        // ..#  ..#  ..#  ..#  .#5
        // ```
        {
            let mut single_dir = None;
            let mut too_many = false;

            if !up_left && left && up && up_right && right && down_right {
                single_dir = Some((-1, 1));
            }
            if !up && up_left && up_right && right && down_right {
                if single_dir.is_none() {
                    single_dir = Some((0, 1));
                } else {
                    too_many = true;
                }
            }
            if !too_many && !up_right && up_left && up && right && down_right {
                if single_dir.is_none() {
                    single_dir = Some((1, 1));
                } else {
                    too_many = true;
                }
            }
            if !too_many && !right && up_left && up && up_right && down_right {
                if single_dir.is_none() {
                    single_dir = Some((1, 0));
                } else {
                    too_many = true;
                }
            }
            if !too_many && !down_right && up_left && up && up_right && right && down {
                if single_dir.is_none() {
                    single_dir = Some((1, -1));
                } else {
                    too_many = true;
                }
            }

            if let Some((dx, dy)) = single_dir {
                if !too_many {
                    // Found a single space to advance into.
                    return Some((
                        dx * real_x_from_x + dy * real_x_from_y,
                        dx * real_y_from_x + dy * real_y_from_y,
                    ));
                }
            }
        }

        // Cardinal directions may lead into the corner of a corridor...
        if !up && right {
            // ... like this going up:
            //
            // ```
            //  ##  ##
            // ..#  #..
            // #@    @#
            // ```
            let up_left_corner = !up_left && left && up_right && up_up_right;
            let up_right_corner = !up_right && up_left && up_up_left;

            if up_up && (up_left_corner || up_right_corner) {
                // Go up into the corridor corner.
                return Some((real_x_from_y, real_y_from_y));
            }
        } else if !right && up {
            // ... like this going right:
            //
            // ```
            // #.    ##
            // @.#  @.#
            //  ##  #.
            // ```
            let right_up_corner = !up_right && down_right && down_right_right;
            let right_down_corner = !down_right && down && up_right && up_right_right;

            if right_right && (right_up_corner || right_down_corner) {
                // Go right into the corridor corner.
                return Some((real_x_from_x, real_y_from_x));
            }
        }

        // No obvious corridor to follow.
        None
    } else {
        // Arrived here cardinally (moved right post-transform).

        // Check for a single space bordered with walls to advance forward or turn up to 90 degrees
        // left or right.
        //
        // ```
        // #1#  .#2  .##  .##  .##
        // .@#  .@#  .@3  .@#  .@#
        // .##  .##  .##  .#4  #5#
        // ```
        {
            let mut single_dir = None;
            let mut too_many = false;

            if !up && up_left && up_right && right && down_right && down {
                single_dir = Some((0, 1));
            }
            if !up_right && up && right && down_right && down {
                if single_dir.is_none() {
                    single_dir = Some((1, 1));
                } else {
                    too_many = true;
                }
            }
            if !too_many && !right && up && up_right && down_right && down {
                if single_dir.is_none() {
                    single_dir = Some((1, 0));
                } else {
                    too_many = true;
                }
            }
            if !too_many && !down_right && up && up_right && right && down {
                if single_dir.is_none() {
                    single_dir = Some((1, -1));
                } else {
                    too_many = true;
                }
            }
            if !too_many && !down && up && up_right && right && down_right && down_left {
                if single_dir.is_none() {
                    single_dir = Some((0, -1));
                } else {
                    too_many = true;
                }
            }

            if let Some((dx, dy)) = single_dir {
                if !too_many {
                    // Found a single space to advance into.
                    return Some((
                        dx * real_x_from_x + dy * real_x_from_y,
                        dx * real_y_from_x + dy * real_y_from_y,
                    ));
                }
            }
        }

        // Cardinal directions may lead into the corner of a corridor...
        if !up && down && right {
            // ... like this going up:
            //
            // ```
            //  ##  ##
            // ..#  #..
            // #@    @#
            // ```
            let up_left_corner = !up_left && left && up_right && up_up_right;
            let up_right_corner = !up_right && up_left && up_up_left;

            if up_up && (up_left_corner || up_right_corner) {
                // Go up into the corridor corner.
                return Some((real_x_from_y, real_y_from_y));
            }
        } else if !right && up && down {
            // ... like this going right:
            //
            // ```
            // #.    ##
            // @.#  @.#
            //  ##  #.
            // ```
            let right_up_corner = !up_right && down_right && down_right_right;
            let right_down_corner = !down_right && up_right && up_right_right;

            if right_right && (right_up_corner || right_down_corner) {
                // Go right into the corridor corner.
                return Some((real_x_from_x, real_y_from_x));
            }
        } else if !down && up && right {
            // ... like this going down:
            //
            // ```
            // #@    @#
            // ..#  #..
            //  ##  ##
            // ```
            let down_left_corner = !down_left && left && down_right && down_down_right;
            let down_right_corner = !down_right && down_left && down_down_left;

            if down_down && (down_left_corner || down_right_corner) {
                // Go down into the corridor corner.
                return Some((-real_x_from_y, -real_y_from_y));
            }
        }

        // No obvious corridor to follow.
        None
    }
}

/// Check if the current player position and desired run direction should perform an auto run
/// across the open or along a single wall.
///
/// Returns the detected wall side or open space, or `None` if walls are on both sides or are
/// partially present.
fn auto_run_straight_check(world: &World, run_dx: i32, run_dy: i32) -> Option<AutoRunWallSide> {
    let (player_x, player_y, forward_blocked) = world.run(
        |map: UniqueView<Map>, player_id: UniqueView<PlayerId>, positions: View<Position>| {
            let pos = positions.get(player_id.0);
            (
                pos.x,
                pos.y,
                map.wall_or_oob(pos.x + run_dx, pos.y + run_dy),
            )
        },
    );

    if forward_blocked {
        None
    } else {
        let (real_x_from_x, real_x_from_y, real_y_from_x, real_y_from_y) =
            rotate_view(run_dx, run_dy);
        let real_x = |dx, dy| player_x + dx * real_x_from_x + dy * real_x_from_y;
        let real_y = |dx, dy| player_y + dx * real_y_from_x + dy * real_y_from_y;

        let (up_left, up, up_right, right, down_right, down) = world.run(|map: UniqueView<Map>| {
            let check_wall = |dx, dy| map.wall_or_oob(real_x(dx, dy), real_y(dx, dy));
            (
                check_wall(-1, 1),
                check_wall(0, 1),
                check_wall(1, 1),
                check_wall(1, 0),
                check_wall(1, -1),
                check_wall(0, -1),
            )
        });

        if run_dx != 0 && run_dy != 0 {
            // Check the walls on either side, i.e. moving up-right, 1 and 2 below:
            //
            // ```
            // 11.
            // .@2
            // ..2
            // ```
            let left_wall = if up_left && up {
                true
            } else if !up_left && !up {
                false
            } else {
                // Don't run against a partial left wall.
                return None;
            };

            let right_wall = if right && down_right {
                true
            } else if !right && !down_right {
                false
            } else {
                // Don't run against a partial right wall.
                return None;
            };

            match (left_wall, right_wall) {
                (true, true) => None, // This should be handled by corridor running instead.
                (true, false) => Some(AutoRunWallSide::Left),
                (false, true) => Some(AutoRunWallSide::Right),
                (false, false) => Some(AutoRunWallSide::Neither),
            }
        } else {
            // Check the walls on either side, i.e. moving right, 1 and 2 below:
            //
            // ```
            // .11
            // .@.
            // .22
            // ```
            let left_wall = if up && up_right {
                true
            } else if !up && !up_right {
                false
            } else {
                // Don't run against a partial left wall.
                return None;
            };

            let right_wall = if down && down_right {
                true
            } else if !down && !down_right {
                false
            } else {
                // Don't run against a partial right wall.
                return None;
            };

            match (left_wall, right_wall) {
                (true, true) => None, // This should be handled by corridor running instead.
                (true, false) => Some(AutoRunWallSide::Left),
                (false, true) => Some(AutoRunWallSide::Right),
                (false, false) => Some(AutoRunWallSide::Neither),
            }
        }
    }
}

/// If the player is currently auto running, calculate and return the direction that auto running
/// should continue in, or `None` to stop auto running.
fn auto_run_next_step(world: &World) -> Option<(i32, i32)> {
    let auto_run = world.run(|player_id: UniqueView<PlayerId>, players: View<Player>| {
        let player = players.get(player_id.0);
        player
            .auto_run
            .as_ref()
            .map(|ar| (ar.run_type, ar.dir.0, ar.dir.1))
    });

    if let Some((run_type, dx, dy)) = auto_run {
        match run_type {
            AutoRunType::Corridor => {
                if let Some(new_dir) = auto_run_corridor_check(world, dx, dy) {
                    // Adjust facing to follow the corridor.
                    world.run(
                        |player_id: UniqueView<PlayerId>, mut players: ViewMut<Player>| {
                            let player = (&mut players).get(player_id.0);
                            if let Some(ar) = &mut player.auto_run {
                                ar.dir = new_dir;
                            }
                        },
                    );
                    Some(new_dir)
                } else {
                    None
                }
            }
            AutoRunType::Straight { expect_wall } => {
                if let Some(actual_wall) = auto_run_straight_check(world, dx, dy) {
                    // Ensure whatever wall we expect is still there.
                    if actual_wall == expect_wall {
                        Some((dx, dy))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    } else {
        None
    }
}

pub fn try_move_player(world: &World, dx: i32, dy: i32, start_run: bool) -> PlayerInputResult {
    if start_run && world.run(player_sees_foes) {
        world.run(|mut msgs: UniqueViewMut<Messages>| {
            msgs.add("You cannot run while foes are near.".into())
        });
        return PlayerInputResult::NoResult;
    }

    let mut melee_queue = Vec::new();
    let (took_time, moved) = world.run(
        |mut map: UniqueViewMut<Map>,
         combat_stats: View<CombatStats>,
         mut fovs: ViewMut<FieldOfView>,
         players: View<Player>,
         mut positions: ViewMut<Position>| {
            let mut took_time = false;
            let mut moved = false;

            for (id, (_, pos, fov)) in (&players, &mut positions, &mut fovs).iter().with_id() {
                let new_x = pos.x + dx;
                let new_y = pos.y + dy;

                if new_x >= 0 && new_y >= 0 && new_x < map.width && new_y < map.height {
                    let melee_target = map
                        .iter_entities_at(new_x, new_y)
                        .find(|e| combat_stats.contains(*e));

                    if let Some(melee_target) = melee_target {
                        melee_queue.push((id, melee_target));
                        took_time = true;
                    } else if !map.is_blocked(new_x, new_y) {
                        map.move_entity(id, pos.into(), (new_x, new_y), false);
                        pos.x = new_x;
                        pos.y = new_y;
                        fov.dirty = true;
                        took_time = true;
                        moved = true;
                    }
                }
            }

            (took_time, moved)
        },
    );

    for (attacker, defender) in melee_queue {
        damage::melee_attack(world, attacker, defender);
    }

    if start_run && moved {
        if auto_run_corridor_check(world, dx, dy).is_some() {
            // Start corridor auto run.
            world.run(
                |player_id: UniqueView<PlayerId>, mut players: ViewMut<Player>| {
                    (&mut players).get(player_id.0).auto_run = Some(AutoRun {
                        limit: 200,
                        dir: (dx, dy),
                        run_type: AutoRunType::Corridor,
                    });
                },
            );
        } else if let Some(expect_wall) = auto_run_straight_check(world, dx, dy) {
            // Start straight auto run.
            world.run(
                |player_id: UniqueView<PlayerId>, mut players: ViewMut<Player>| {
                    (&mut players).get(player_id.0).auto_run = Some(AutoRun {
                        limit: 200,
                        dir: (dx, dy),
                        run_type: AutoRunType::Straight { expect_wall },
                    });
                },
            );
        }
    }

    if took_time {
        PlayerInputResult::TurnDone
    } else {
        PlayerInputResult::NoResult
    }
}

pub fn player_pick_up_item(world: &World, item_id: EntityId) {
    let player_id = world.run(|player_id: UniqueView<PlayerId>| player_id.0);

    item::remove_item_from_map(world, item_id);
    item::add_item_to_inventory(world, player_id, item_id);
    world.run(|mut msgs: UniqueViewMut<Messages>, names: View<Name>| {
        msgs.add(format!(
            "{} picks up {}.",
            names.get(player_id).0,
            names.get(item_id).0
        ));
    });
}

pub fn player_drop_item(world: &World, item_id: EntityId) {
    let player_id = world.run(|player_id: UniqueView<PlayerId>| player_id.0);
    let player_pos: (i32, i32) =
        world.run(|positions: View<Position>| positions.get(player_id).into());

    item::remove_item_from_inventory(world, player_id, item_id);
    item::add_item_to_map(world, item_id, player_pos);
    world.run(|mut msgs: UniqueViewMut<Messages>, names: View<Name>| {
        msgs.add(format!(
            "{} drops {}.",
            names.get(player_id).0,
            names.get(item_id).0
        ));
    });
}

pub fn player_input(world: &World, inputs: &mut InputBuffer) -> PlayerInputResult {
    let player_id = world.borrow::<UniqueView<PlayerId>>();

    inputs.prepare_input();

    if item::is_asleep(world, player_id.0) {
        if let Some(InputEvent::Press(Button::Keyboard(key))) = inputs.get_input() {
            match key.into() {
                GameKey::Cancel => PlayerInputResult::ShowExitPrompt,
                _ => {
                    world.run(|mut msgs: UniqueViewMut<Messages>, names: View<Name>| {
                        msgs.add(format!("{} is sleeping.", names.get(player_id.0).0));
                    });
                    item::handle_sleep_turn(world, player_id.0);
                    PlayerInputResult::TurnDone
                }
            }
        } else {
            PlayerInputResult::NoResult
        }
    } else if world.run(player_is_auto_running) {
        if matches!(
            inputs.get_input(),
            Some(InputEvent::Press(Button::Keyboard(_)))
        ) || world.run(player_check_frontier)
            || world.run(player_sees_foes)
        {
            world.run(player_stop_auto_run);
            PlayerInputResult::NoResult
        } else {
            let limit_reached = world.run(|mut players: ViewMut<Player>| {
                let player = (&mut players).get(player_id.0);
                if let Some(auto_run) = &mut player.auto_run {
                    if auto_run.limit > 0 {
                        auto_run.limit -= 1;
                    }
                    auto_run.limit <= 0
                } else {
                    true
                }
            });

            if limit_reached {
                world.run(player_stop_auto_run);
                PlayerInputResult::NoResult
            } else if let Some((dx, dy)) = auto_run_next_step(world) {
                // Do one step of auto running.
                try_move_player(world, dx, dy, false)
            } else {
                world.run(player_stop_auto_run);
                PlayerInputResult::NoResult
            }
        }
    } else if let Some(InputEvent::Press(Button::Keyboard(key))) = inputs.get_input() {
        let shift = inputs.get_mods(KeyMods::SHIFT);

        match key.into() {
            GameKey::Left => try_move_player(world, -1, 0, shift),
            GameKey::Down => try_move_player(world, 0, 1, shift),
            GameKey::Up => try_move_player(world, 0, -1, shift),
            GameKey::Right => try_move_player(world, 1, 0, shift),
            GameKey::UpLeft => try_move_player(world, -1, -1, shift),
            GameKey::UpRight => try_move_player(world, 1, -1, shift),
            GameKey::DownLeft => try_move_player(world, -1, 1, shift),
            GameKey::DownRight => try_move_player(world, 1, 1, shift),
            GameKey::Wait => PlayerInputResult::TurnDone,
            GameKey::Cancel => PlayerInputResult::ShowExitPrompt,
            GameKey::PickUp => PlayerInputResult::ShowPickUpMenu,
            GameKey::Inventory | GameKey::Confirm => PlayerInputResult::ShowInventory,
            _ => PlayerInputResult::NoResult,
        }
    } else {
        PlayerInputResult::NoResult
    }
}

pub fn player_is_alive(player_alive: UniqueView<PlayerAlive>) -> bool {
    player_alive.0
}

pub fn player_is_dead_input(inputs: &mut InputBuffer) -> bool {
    inputs.prepare_input();

    matches!(
        inputs.get_input(),
        Some(InputEvent::Press(Button::Keyboard(Key::Space)))
    )
}
