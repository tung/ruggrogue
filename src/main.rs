mod components;
mod damage;
mod map;
mod monster;
mod player;
mod rect;
mod vision;

use rand::{thread_rng, Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;
use shipyard::{
    EntitiesViewMut, EntityId, Get, IntoIter, Shiperator, UniqueView, UniqueViewMut, View, ViewMut,
    World,
};
use std::path::PathBuf;

use crate::{
    components::{
        BlocksTile, CombatStats, FieldOfView, Monster, Name, Player, PlayerId, Position, Renderable,
    },
    damage::{
        delete_dead_entities, inflict_damage, melee_combat, DamageQueue, DeadEntities, MeleeQueue,
    },
    map::{draw_map, Map},
    monster::{do_monster_turns, enqueue_monster_turns, monster_turns_empty, MonsterTurns},
    player::{player_input, player_is_dead_input},
    vision::recalculate_fields_of_view,
};
use ruggle::{CharGrid, RunSettings};

pub struct PlayerAlive(bool);

pub struct RuggleRng(Pcg64Mcg);

fn player_is_alive(player_alive: UniqueView<PlayerAlive>) -> bool {
    player_alive.0
}

#[allow(clippy::too_many_arguments)]
fn spawn_player(
    mut map: UniqueViewMut<Map>,
    mut entities: EntitiesViewMut,
    mut combat_stats: ViewMut<CombatStats>,
    mut fovs: ViewMut<FieldOfView>,
    mut names: ViewMut<Name>,
    mut players: ViewMut<Player>,
    mut positions: ViewMut<Position>,
    mut renderables: ViewMut<Renderable>,
) -> EntityId {
    let player_id = entities.add_entity(
        (
            &mut players,
            &mut combat_stats,
            &mut fovs,
            &mut names,
            &mut positions,
            &mut renderables,
        ),
        (
            Player {},
            CombatStats {
                max_hp: 30,
                hp: 30,
                defense: 2,
                power: 5,
            },
            FieldOfView::new(8),
            Name("Player".into()),
            Position { x: 0, y: 0 },
            Renderable {
                ch: '@',
                fg: [1., 1., 0., 1.],
                bg: [0., 0., 0., 1.],
            },
        ),
    );

    map.place_entity(player_id, (0, 0), false);

    player_id
}

#[allow(clippy::too_many_arguments)]
fn spawn_monsters_in_rooms(
    mut map: UniqueViewMut<Map>,
    mut rng: UniqueViewMut<RuggleRng>,
    mut entities: EntitiesViewMut,
    mut blocks: ViewMut<BlocksTile>,
    mut combat_stats: ViewMut<CombatStats>,
    mut fovs: ViewMut<FieldOfView>,
    mut monsters: ViewMut<Monster>,
    mut names: ViewMut<Name>,
    mut positions: ViewMut<Position>,
    mut renderables: ViewMut<Renderable>,
) {
    for room in map.rooms.iter().skip(1) {
        let (ch, name, fg) = match rng.0.gen_range(0, 2) {
            0 => ('g', "Goblin", [0.5, 0.9, 0.2, 1.]),
            1 => ('o', "Orc", [0.9, 0.3, 0.2, 1.]),
            _ => ('X', "???", [1., 0., 0., 1.]),
        };

        entities.add_entity(
            (
                &mut monsters,
                &mut blocks,
                &mut combat_stats,
                &mut fovs,
                &mut names,
                &mut positions,
                &mut renderables,
            ),
            (
                Monster {},
                BlocksTile {},
                CombatStats {
                    max_hp: 16,
                    hp: 16,
                    defense: 1,
                    power: 4,
                },
                FieldOfView::new(8),
                Name(name.into()),
                room.center().into(),
                Renderable {
                    ch,
                    fg,
                    bg: [0., 0., 0., 1.],
                },
            ),
        );
    }

    for (id, (_, _, pos)) in (&monsters, &blocks, &positions).iter().with_id() {
        map.place_entity(id, pos.into(), true);
    }
}

fn draw_renderables(world: &World, grid: &mut CharGrid) {
    world.run(
        |player: UniqueView<PlayerId>,
         fovs: View<FieldOfView>,
         positions: View<Position>,
         renderables: View<Renderable>| {
            let (x, y) = positions.get(player.0).into();
            let fov = fovs.get(player.0);

            for (pos, render) in (&positions, &renderables).iter() {
                let gx = pos.x - x + 40;
                let gy = pos.y - y + 18;

                if gx >= 0 && gy >= 0 && gx < 80 && gy < 36 && fov.get(pos.into()) {
                    grid.put_color([gx, gy], Some(render.fg), Some(render.bg), render.ch);
                }
            }
        },
    );
}

fn main() {
    let world = World::new();

    world.add_unique(RuggleRng(Pcg64Mcg::from_rng(thread_rng()).unwrap()));

    world.add_unique(Map::new(80, 50));
    world.run(map::generate_rooms_and_corridors);

    world.add_unique(PlayerId(world.run(spawn_player)));
    world.add_unique(PlayerAlive(true));
    world.run(map::place_player_in_first_room);

    world.run(spawn_monsters_in_rooms);

    world.add_unique(MonsterTurns::new());

    world.add_unique(MeleeQueue::new());
    world.add_unique(DamageQueue::new());
    world.add_unique(DeadEntities::new());

    world.run(recalculate_fields_of_view);

    let settings = RunSettings {
        title: "Ruggle".to_string(),
        grid_size: [80, 36],
        font_path: PathBuf::from("assets/gohufont-uni-14.ttf"),
        font_size: 14.0,
        min_fps: 30,
        max_fps: 60,
    };

    ruggle::run(settings, |mut inputs, mut grid| {
        if world.run(player_is_alive) {
            let (time_passed, player_turn_done) = if !world.run(monster_turns_empty) {
                world.run(do_monster_turns);
                (true, false)
            } else if player_input(&world, &mut inputs) {
                (true, true)
            } else {
                (false, false)
            };

            if time_passed {
                world.run(melee_combat);
                world.run(inflict_damage);
                world.run(delete_dead_entities);
                world.run(recalculate_fields_of_view);
                if player_turn_done {
                    world.run(enqueue_monster_turns);
                }
            }
        } else if player_is_dead_input(&mut inputs) {
            return (false, false);
        }

        grid.clear();
        draw_map(&world, &mut grid);
        draw_renderables(&world, &mut grid);

        (
            true,
            world.run(player_is_alive) && !world.run(monster_turns_empty),
        )
    });
}
