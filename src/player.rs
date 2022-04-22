use sdl2::keyboard::Keycode;
use serde::{Deserialize, Serialize};
use shipyard::{
    EntitiesView, EntityId, Get, IntoIter, Shiperator, UniqueView, UniqueViewMut, View, ViewMut,
    World,
};

use crate::{
    components::{CombatStats, Coord, FieldOfView, Item, Monster, Name, Player},
    damage, experience,
    gamekey::{self, GameKey},
    hunger::{self, CanRegenResult},
    item::{self, PickUpHint},
    map::{self, Map, Tile},
    message::Messages,
    spawn, vision,
};
use ruggrogue::{util::Position, InputBuffer, InputEvent, KeyMods, PathableMap};

#[derive(Deserialize, Serialize)]
pub struct PlayerId(pub EntityId);

#[derive(Deserialize, Serialize)]
pub struct PlayerAlive(pub bool);

#[derive(Clone, Copy, PartialEq)]
enum AutoRunWallSide {
    Neither,
    Left,
    Right,
}

#[derive(Clone, Copy)]
enum AutoRunType {
    RestInPlace,
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
    ViewMap,
    ShowPickUpMenu,
    ShowInventory,
    ShowInventoryShortcut(GameKey),
    ShowEquipmentShortcut(GameKey),
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

/// Check current and newly-adjacent tiles to the player for things worth stopping for during auto
/// run.
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

    if matches!(run_type, AutoRunType::RestInPlace) {
        // Interrupting resting is handled elsewhere.
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
        stop_for(0, 0)
            || stop_for(-1, 1)
            || stop_for(0, 1)
            || stop_for(1, 1)
            || stop_for(1, 0)
            || stop_for(1, -1)
    } else {
        // There are three newly-adjacent tiles after a cardinal move.
        stop_for(0, 0) || stop_for(1, 1) || stop_for(1, 0) || stop_for(1, -1)
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

    const UP_LEFT: u16 = 1;
    const UP: u16 = 1 << 1;
    const UP_RIGHT: u16 = 1 << 2;
    const LEFT: u16 = 1 << 3;
    const RIGHT: u16 = 1 << 4;
    const DOWN_LEFT: u16 = 1 << 5;
    const DOWN: u16 = 1 << 6;
    const DOWN_RIGHT: u16 = 1 << 7;

    const UP_UP_LEFT: u16 = 1 << 8;
    const UP_UP: u16 = 1 << 9;
    const UP_UP_RIGHT: u16 = 1 << 10;
    const UP_RIGHT_RIGHT: u16 = 1 << 11;
    const RIGHT_RIGHT: u16 = 1 << 12;
    const DOWN_RIGHT_RIGHT: u16 = 1 << 13;
    const DOWN_DOWN_LEFT: u16 = 1 << 14;
    const DOWN_DOWN: u16 = 1 << 15;

    let mut nearby_walls: u16 = 0;

    // Check nearby tiles for walls.
    world.run(
        |map: UniqueView<Map>, player_id: UniqueView<PlayerId>, fovs: View<FieldOfView>| {
            let player_fov = fovs.get(player_id.0);
            let check_unknown_or_wall = |dx, dy| {
                !player_fov.get((real_x(dx, dy), real_y(dx, dy)))
                    || map.wall_or_oob(real_x(dx, dy), real_y(dx, dy))
            };

            // (dx, dy, wall bit to set in nearby_walls)
            const NEARBY_TILES: [(i32, i32, u16); 16] = [
                (-1, 1, UP_LEFT),
                (0, 1, UP),
                (1, 1, UP_RIGHT),
                (-1, 0, LEFT),
                (1, 0, RIGHT),
                (-1, -1, DOWN_LEFT),
                (0, -1, DOWN),
                (1, -1, DOWN_RIGHT),
                (-1, 2, UP_UP_LEFT),
                (0, 2, UP_UP),
                (1, 2, UP_UP_RIGHT),
                (2, 1, UP_RIGHT_RIGHT),
                (2, 0, RIGHT_RIGHT),
                (2, -1, DOWN_RIGHT_RIGHT),
                (-1, -2, DOWN_DOWN_LEFT),
                (0, -2, DOWN_DOWN),
            ];

            for (dx, dy, wall_bit) in NEARBY_TILES {
                if check_unknown_or_wall(dx, dy) {
                    nearby_walls |= wall_bit;
                }
            }
        },
    );

    if run_dx != 0 && run_dy != 0 {
        // Arrived here diagonally (moved up-right post-transform).
        const UP_RIGHT_ARC: u16 = UP_LEFT | UP | UP_RIGHT | RIGHT | DOWN_RIGHT;

        // (move dx, move dy, tiles that must be open, tiles to check)
        const DIRS_AND_CHECKS: [(i32, i32, u16, u16); 9] = [
            // Check for a single space bordered with walls to advance forward or turn up to 90
            // degrees left or right.
            //
            // ```
            // 1##  #2#  ##3  ###  ###
            // #@#  .@#  .@#  .@4  .@#
            // ..#  ..#  ..#  ..#  .#5
            // ```
            (-1, 1, UP_LEFT, UP_RIGHT_ARC | LEFT),
            (0, 1, UP, UP_RIGHT_ARC),
            (1, 1, UP_RIGHT, UP_RIGHT_ARC),
            (1, 0, RIGHT, UP_RIGHT_ARC),
            (1, -1, DOWN_RIGHT, UP_RIGHT_ARC | DOWN),
            // Cardinal directions may lead into the corner of a corridor.
            //
            // ```
            //  ##  ##
            // 66#  #77  #8   ###
            // #@#   @#  @8#  @9#
            //            ##  #9
            // ```
            (
                0,
                1,
                UP_LEFT | UP,
                UP_UP | UP_UP_RIGHT | UP_LEFT | UP | UP_RIGHT | LEFT | RIGHT,
            ),
            (
                0,
                1,
                UP | UP_RIGHT,
                UP_UP_LEFT | UP_UP | UP_LEFT | UP | UP_RIGHT | RIGHT,
            ),
            (
                1,
                0,
                UP_RIGHT | RIGHT,
                UP | UP_RIGHT | RIGHT | RIGHT_RIGHT | DOWN_RIGHT | DOWN_RIGHT_RIGHT,
            ),
            (
                1,
                0,
                RIGHT | DOWN_RIGHT,
                UP | UP_RIGHT | UP_RIGHT_RIGHT | RIGHT | RIGHT_RIGHT | DOWN | DOWN_RIGHT,
            ),
        ];

        for (move_dx, move_dy, open_bits, mask_bits) in DIRS_AND_CHECKS {
            if nearby_walls & mask_bits == mask_bits & !open_bits {
                return Some((
                    move_dx * real_x_from_x + move_dy * real_x_from_y,
                    move_dx * real_y_from_x + move_dy * real_y_from_y,
                ));
            }
        }

        // No obvious corridor to follow.
        None
    } else {
        // Arrived here cardinally (moved right post-transform).
        const RIGHT_ARC: u16 = UP | UP_RIGHT | RIGHT | DOWN | DOWN_RIGHT;

        // (move dx, move dy, tiles that must be open, tiles to check)
        const DIRS_AND_CHECKS: [(i32, i32, u16, u16); 9] = [
            // Check for a single space bordered with walls to advance forward or turn up to 90
            // degrees left or right.
            //
            // ```
            // #1#  .#2  .##  .##  .##
            // .@#  .@#  .@3  .@#  .@#
            // .##  .##  .##  .#4  #5#
            // ```
            (0, 1, UP, RIGHT_ARC | UP_LEFT),
            (1, 1, UP_RIGHT, RIGHT_ARC),
            (1, 0, RIGHT, RIGHT_ARC),
            (1, -1, DOWN_RIGHT, RIGHT_ARC),
            (0, -1, DOWN, RIGHT_ARC | DOWN_LEFT),
            // Cardinal directions may lead into the corner of a corridor.
            //
            // ```
            // ##
            // #66  #7   ###   ##
            //  @#  @7#  @8#   @#
            //  ##  ###  #8   #99
            //                ##
            // ```
            (
                0,
                1,
                UP | UP_RIGHT,
                UP_UP_LEFT | UP_UP | UP_LEFT | UP | UP_RIGHT | RIGHT | DOWN | DOWN_RIGHT,
            ),
            (
                1,
                0,
                UP_RIGHT | RIGHT,
                UP | UP_RIGHT | RIGHT | RIGHT_RIGHT | DOWN | DOWN_RIGHT | DOWN_RIGHT_RIGHT,
            ),
            (
                1,
                0,
                RIGHT | DOWN_RIGHT,
                UP | UP_RIGHT | UP_RIGHT_RIGHT | RIGHT | RIGHT_RIGHT | DOWN | DOWN_RIGHT,
            ),
            (
                0,
                -1,
                DOWN | DOWN_RIGHT,
                UP | UP_RIGHT | RIGHT | DOWN_LEFT | DOWN | DOWN_RIGHT | DOWN_DOWN_LEFT | DOWN_DOWN,
            ),
        ];

        for (move_dx, move_dy, open_bits, mask_bits) in DIRS_AND_CHECKS {
            if nearby_walls & mask_bits == mask_bits & !open_bits {
                return Some((
                    move_dx * real_x_from_x + move_dy * real_x_from_y,
                    move_dx * real_y_from_x + move_dy * real_y_from_y,
                ));
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
            AutoRunType::RestInPlace => {
                let player_id = world.borrow::<UniqueView<PlayerId>>();

                // Rest while player can regenerate hit points.
                if matches!(
                    hunger::can_regen(world, player_id.0),
                    CanRegenResult::CanRegen
                ) {
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

fn wait_player(world: &World, rest_in_place: bool) -> PlayerInputResult {
    let foes_seen = world.run(player_sees_foes);
    let (player_id, mut players) = world.borrow::<(UniqueView<PlayerId>, ViewMut<Player>)>();
    let player_can_regen = hunger::can_regen(world, player_id.0);
    let mut msgs = world.borrow::<UniqueViewMut<Messages>>();

    if rest_in_place {
        if foes_seen {
            msgs.add("You cannot rest while foes are near.".into());
            return PlayerInputResult::NoResult;
        } else if !matches!(player_can_regen, CanRegenResult::CanRegen) {
            match player_can_regen {
                CanRegenResult::CanRegen => unreachable!(),
                CanRegenResult::NoRegen => msgs.add("You cannot rest to heal.".into()),
                CanRegenResult::FullyRested => msgs.add("You are already fully rested.".into()),
                CanRegenResult::TooHungry => msgs.add("You are too hungry to rest.".into()),
            }
            return PlayerInputResult::NoResult;
        }

        // Rest in place if requested.
        if matches!(player_can_regen, CanRegenResult::CanRegen) {
            msgs.add("You tend to your wounds.".into());
            (&mut players).get(player_id.0).auto_run = Some(AutoRun {
                limit: 400,
                dir: (0, 0),
                run_type: AutoRunType::RestInPlace,
            });
        }
    }

    PlayerInputResult::TurnDone
}

pub fn add_coords_to_players(
    entities: EntitiesView,
    mut coords: ViewMut<Coord>,
    players: View<Player>,
) {
    let player_ids = players.iter().with_id().map(|(id, _)| id);

    for id in player_ids {
        entities.add_component((&mut coords,), (Coord((0, 0).into()),), id);
    }
}

pub fn remove_coords_from_players(mut coords: ViewMut<Coord>, players: View<Player>) {
    let player_ids = players.iter().with_id().map(|(id, _)| id);

    for id in player_ids {
        coords.remove(id);
    }
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
    world.run(remove_coords_from_players);
    world.run(spawn::despawn_coord_entities);
    world.run(add_coords_to_players);

    world.run(|mut map: UniqueViewMut<Map>| {
        map.clear();
        map.depth += 1;
    });
    if let Some(victory_pos) = world.run(map::generate_rooms_and_corridors) {
        spawn::spawn_present(world, victory_pos);
    }
    world.run(map::place_player_in_first_room);

    world.run(experience::redeem_exp_for_next_depth);
    world.run(experience::gain_levels);
    spawn::fill_rooms_with_spawns(world);
    world.run(experience::calc_exp_for_next_depth);

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

/// Describe contents of the tile the player is on.
pub fn describe_player_pos(world: &World) {
    let Position { x, y } = {
        let player_id = world.borrow::<UniqueView<PlayerId>>();
        let coords = world.borrow::<View<Coord>>();
        coords.get(player_id.0).0
    };
    let map = world.borrow::<UniqueView<Map>>();
    let more_than_player = map.iter_entities_at(x, y).nth(1).is_some();
    let pick_up_hint = world.borrow::<UniqueView<PickUpHint>>().0
        && map
            .iter_entities_at(x, y)
            .any(|id| world.borrow::<View<Item>>().contains(id));
    let tile = map.get_tile(x, y);

    if more_than_player || !matches!(tile, Tile::Floor | Tile::Wall) {
        let (desc, recalled) = map.describe_pos(world, x, y, false, true, true);
        let downstairs = matches!(tile, Tile::DownStairs) && map.depth == 1;

        world.borrow::<UniqueViewMut<Messages>>().add(format!(
            "You {} {} here.{}",
            if recalled { "recall" } else { "see" },
            desc,
            match (pick_up_hint, downstairs) {
                (true, true) => " (Press 'g' to pick up, 'Enter' to descend.)",
                (true, false) => " (Press 'g' to pick up.)",
                (false, true) => " (Press 'Enter' to descend.)",
                _ => "",
            },
        ));
    }
}

pub fn player_input(world: &World, inputs: &mut InputBuffer) -> PlayerInputResult {
    let player_id = world.borrow::<UniqueView<PlayerId>>();

    inputs.prepare_input();

    if item::is_asleep(world, player_id.0) {
        if let Some(InputEvent::AppQuit) = inputs.get_input() {
            PlayerInputResult::AppQuit
        } else if let Some(InputEvent::Press(keycode)) = inputs.get_input() {
            let shift = inputs.get_mods(KeyMods::SHIFT);
            let key = gamekey::from_keycode(keycode, shift);

            if !matches!(key, GameKey::Unmapped) {
                world.borrow::<UniqueViewMut<Messages>>().reset_highlight();
            }

            match key {
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
        let key = gamekey::from_keycode(keycode, shift);

        if !matches!(key, GameKey::Unmapped) {
            world.borrow::<UniqueViewMut<Messages>>().reset_highlight();
        }

        match key {
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
            GameKey::ViewMap => PlayerInputResult::ViewMap,
            GameKey::Descend | GameKey::Confirm => PlayerInputResult::TryDescend,
            GameKey::PickUp => PlayerInputResult::ShowPickUpMenu,
            GameKey::Inventory => PlayerInputResult::ShowInventory,
            key @ GameKey::UseItem | key @ GameKey::EquipItem | key @ GameKey::DropItem => {
                PlayerInputResult::ShowInventoryShortcut(key)
            }
            key @ GameKey::RemoveItem => PlayerInputResult::ShowEquipmentShortcut(key),
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
