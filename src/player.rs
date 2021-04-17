use sdl2::keyboard::Keycode;
use shipyard::{
    EntityId, Get, IntoIter, Shiperator, UniqueView, UniqueViewMut, View, ViewMut, World,
};
use std::collections::HashSet;

use crate::{
    components::{CombatStats, Coord, FieldOfView, Inventory, Item, Monster, Name, Player},
    damage,
    gamekey::{self, GameKey},
    item,
    map::{self, Map, Tile},
    message::Messages,
    spawn, vision,
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
    RestUntilHealed,
    Corridor,
    Straight { expect_wall: AutoRunWallSide },
}

pub struct AutoRun {
    limit: i32,
    dir: (i32, i32),
    run_type: AutoRunType,
}

pub enum PlayerInputResult {
    AppQuit,
    NoResult,
    TurnDone,
    TryDescend,
    ShowOptionsMenu,
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
    let (player_id, coords, fovs) =
        world.borrow::<(UniqueView<PlayerId>, View<Coord>, View<FieldOfView>)>();

    if let Ok(fov) = fovs.try_get(who) {
        let player_coord = coords.get(player_id.0);

        fov.get(player_coord.0.into())
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
    coords: View<Coord>,
    items: View<Item>,
    players: View<Player>,
) -> bool {
    let player = players.get(player_id.0);
    let AutoRun {
        dir: (auto_run_dx, auto_run_dy),
        run_type,
        ..
    } = *player.auto_run.as_ref().unwrap();

    if matches!(run_type, AutoRunType::RestUntilHealed) {
        // Interrupting resting until healed is handled elsewhere.
        return false;
    }

    let player_coord = coords.get(player_id.0);
    let (real_x_from_x, real_x_from_y, real_y_from_x, real_y_from_y) =
        rotate_view(auto_run_dx, auto_run_dy);
    let real_x = |dx, dy| player_coord.0.x + dx * real_x_from_x + dy * real_x_from_y;
    let real_y = |dx, dy| player_coord.0.y + dx * real_y_from_x + dy * real_y_from_y;
    let stop_for = |dx, dy| {
        let (map_x, map_y) = (real_x(dx, dy), real_y(dx, dy));

        // Stop for unusual dungeon features.
        if !matches!(map.get_tile(map_x, map_y), Tile::Floor | Tile::Wall) {
            return true;
        }

        // Stop for items.
        if map
            .iter_entities_at(map_x, map_y)
            .any(|id| items.contains(id))
        {
            return true;
        }

        false
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
    let (player_x, player_y): (i32, i32) =
        world.run(|player_id: UniqueView<PlayerId>, coords: View<Coord>| {
            coords.get(player_id.0).0.into()
        });
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
        |map: UniqueView<Map>, player_id: UniqueView<PlayerId>, coords: View<Coord>| {
            let coord = coords.get(player_id.0);
            (
                coord.0.x,
                coord.0.y,
                map.wall_or_oob(coord.0.x + run_dx, coord.0.y + run_dy),
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
            AutoRunType::RestUntilHealed => {
                let (player_id, combat_stats) =
                    world.borrow::<(UniqueView<PlayerId>, View<CombatStats>)>();
                let CombatStats { hp, max_hp, .. } = combat_stats.get(player_id.0);

                // Rest until player is healed.
                if hp < max_hp {
                    Some((0, 0))
                } else {
                    None
                }
            }
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
         mut coords: ViewMut<Coord>,
         mut fovs: ViewMut<FieldOfView>,
         players: View<Player>| {
            let mut took_time = false;
            let mut moved = false;

            for (id, (_, coord, fov)) in (&players, &mut coords, &mut fovs).iter().with_id() {
                let new_x = coord.0.x + dx;
                let new_y = coord.0.y + dy;

                if new_x >= 0 && new_y >= 0 && new_x < map.width && new_y < map.height {
                    let melee_target = map
                        .iter_entities_at(new_x, new_y)
                        .find(|e| combat_stats.contains(*e));

                    if let Some(melee_target) = melee_target {
                        melee_queue.push((id, melee_target));
                        took_time = true;
                    } else if !map.is_blocked(new_x, new_y) {
                        map.move_entity(id, coord.0.into(), (new_x, new_y), false);
                        coord.0 = (new_x, new_y).into();
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

fn wait_player(world: &World, rest_until_healed: bool) -> PlayerInputResult {
    let foes_seen = world.run(player_sees_foes);
    let (player_id, mut combat_stats, mut players) =
        world.borrow::<(UniqueView<PlayerId>, ViewMut<CombatStats>, ViewMut<Player>)>();
    let mut player_stats = (&mut combat_stats).get(player_id.0);
    let mut msgs = world.borrow::<UniqueViewMut<Messages>>();

    if rest_until_healed {
        if foes_seen {
            msgs.add("You cannot rest while foes are near.".into());
            return PlayerInputResult::NoResult;
        } else if player_stats.hp >= player_stats.max_hp {
            msgs.add("You are already fully rested.".into());
            return PlayerInputResult::NoResult;
        }
    }

    // Regain hit points when waiting without foes in field of view.
    if !foes_seen && player_stats.hp < player_stats.max_hp {
        player_stats.hp += 1;
        if rest_until_healed {
            msgs.add("You tend to your wounds.".into());
        }
    }

    // Continue resting until healed if requested.
    if rest_until_healed && player_stats.hp < player_stats.max_hp {
        (&mut players).get(player_id.0).auto_run = Some(AutoRun {
            limit: 200,
            dir: (0, 0),
            run_type: AutoRunType::RestUntilHealed,
        });
    }

    PlayerInputResult::TurnDone
}

pub fn all_player_associated_ids(
    inventories: View<Inventory>,
    players: View<Player>,
) -> HashSet<EntityId> {
    let mut ids = HashSet::new();

    for (id, _) in players.iter().with_id() {
        // Add the player.
        ids.insert(id);

        // Add the player's inventory items.
        if let Ok(inv) = inventories.try_get(id) {
            ids.extend(inv.items.iter());
        }
    }

    ids
}

pub fn player_try_descend(
    map: UniqueView<Map>,
    mut msgs: UniqueViewMut<Messages>,
    player_id: UniqueView<PlayerId>,
    coords: View<Coord>,
) -> bool {
    let player_coord = coords.get(player_id.0);

    if matches!(
        map.get_tile(player_coord.0.x, player_coord.0.y),
        Tile::DownStairs
    ) {
        true
    } else {
        msgs.add("There is no way down here.".into());
        false
    }
}

pub fn player_do_descend(world: &World) {
    spawn::despawn_all_but_player(world);
    world.run(|mut map: UniqueViewMut<Map>| {
        map.clear();
        map.depth += 1;
    });
    world.run(map::generate_rooms_and_corridors);
    world.run(map::place_player_in_first_room);
    spawn::fill_rooms_with_spawns(world);

    world.run(|mut fovs: ViewMut<FieldOfView>, players: View<Player>| {
        for (fov, _) in (&mut fovs, &players).iter() {
            fov.dirty = true;
        }
    });
    world.run(vision::recalculate_fields_of_view);

    world.run(
        |map: UniqueView<Map>,
         mut msgs: UniqueViewMut<Messages>,
         player_id: UniqueView<PlayerId>,
         names: View<Name>| {
            msgs.add(format!(
                "{} descends to depth {}.",
                names.get(player_id.0).0,
                map.depth,
            ));
        },
    );
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
    let player_pos: (i32, i32) = world.run(|coords: View<Coord>| coords.get(player_id).0.into());

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
        if let Some(InputEvent::AppQuit) = inputs.get_input() {
            PlayerInputResult::AppQuit
        } else if let Some(InputEvent::Press(keycode)) = inputs.get_input() {
            match gamekey::from_keycode(keycode, inputs.get_mods(KeyMods::SHIFT)) {
                GameKey::Cancel => PlayerInputResult::ShowOptionsMenu,
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
        if matches!(inputs.get_input(), Some(InputEvent::AppQuit)) {
            world.run(player_stop_auto_run);
            PlayerInputResult::AppQuit
        } else if matches!(inputs.get_input(), Some(InputEvent::Press(_)))
            || world.run(player_check_frontier)
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
                if dx == 0 && dy == 0 {
                    wait_player(world, false)
                } else {
                    try_move_player(world, dx, dy, false)
                }
            } else {
                world.run(player_stop_auto_run);
                PlayerInputResult::NoResult
            }
        }
    } else if let Some(InputEvent::AppQuit) = inputs.get_input() {
        PlayerInputResult::AppQuit
    } else if let Some(InputEvent::Press(keycode)) = inputs.get_input() {
        let shift = inputs.get_mods(KeyMods::SHIFT);

        match gamekey::from_keycode(keycode, shift) {
            GameKey::Left => try_move_player(world, -1, 0, shift),
            GameKey::Down => try_move_player(world, 0, 1, shift),
            GameKey::Up => try_move_player(world, 0, -1, shift),
            GameKey::Right => try_move_player(world, 1, 0, shift),
            GameKey::UpLeft => try_move_player(world, -1, -1, shift),
            GameKey::UpRight => try_move_player(world, 1, -1, shift),
            GameKey::DownLeft => try_move_player(world, -1, 1, shift),
            GameKey::DownRight => try_move_player(world, 1, 1, shift),
            GameKey::Wait => wait_player(world, shift),
            GameKey::Cancel => PlayerInputResult::ShowOptionsMenu,
            GameKey::Descend | GameKey::Confirm => PlayerInputResult::TryDescend,
            GameKey::PickUp => PlayerInputResult::ShowPickUpMenu,
            GameKey::Inventory => PlayerInputResult::ShowInventory,
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
        Some(InputEvent::Press(Keycode::Space)) | Some(InputEvent::AppQuit)
    )
}
