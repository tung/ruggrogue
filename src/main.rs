mod components;
mod damage;
mod map;
mod message;
mod monster;
mod player;
mod rect;
mod spawn;
mod ui;
mod vision;

use rand::{thread_rng, SeedableRng};
use rand_pcg::Pcg64Mcg;
use shipyard::{Get, IntoIter, UniqueView, View, World};
use std::path::PathBuf;

use crate::{
    components::{FieldOfView, PlayerId, Position, Renderable},
    damage::{
        delete_dead_entities, inflict_damage, melee_combat, DamageQueue, DeadEntities, MeleeQueue,
    },
    map::{draw_map, Map},
    message::Messages,
    monster::{do_monster_turns, enqueue_monster_turns, monster_turns_empty, MonsterTurns},
    player::{player_input, player_is_dead_input},
    ui::draw_ui,
    vision::recalculate_fields_of_view,
};
use ruggle::{CharGrid, RunControl, RunSettings};

pub struct PlayerAlive(bool);

pub struct RuggleRng(Pcg64Mcg);

fn player_is_alive(player_alive: UniqueView<PlayerAlive>) -> bool {
    player_alive.0
}

fn draw_renderables(world: &World, grid: &mut CharGrid) {
    world.run(
        |player: UniqueView<PlayerId>,
         fovs: View<FieldOfView>,
         positions: View<Position>,
         renderables: View<Renderable>| {
            let (x, y) = positions.get(player.0).into();
            let fov = fovs.get(player.0);
            let w = grid.size_cells()[0];
            let h = grid.size_cells()[1] - ui::HUD_LINES;
            let cx = w / 2;
            let cy = h / 2;

            for (pos, render) in (&positions, &renderables).iter() {
                let gx = pos.x - x + cx;
                let gy = pos.y - y + cy;

                if gx >= 0 && gy >= 0 && gx < w && gy < h && fov.get(pos.into()) {
                    grid.put_color([gx, gy], Some(render.fg), Some(render.bg), render.ch);
                }
            }
        },
    );
}

fn main() {
    let world = World::new();

    world.add_unique(RuggleRng(Pcg64Mcg::from_rng(thread_rng()).unwrap()));

    let mut messages = Messages::new(4);
    messages.add("Welcome to Ruggle!".into());
    world.add_unique(messages);

    world.add_unique(Map::new(80, 50));
    world.run(map::generate_rooms_and_corridors);

    world.add_unique(PlayerId(world.run(spawn::spawn_player)));
    world.add_unique(PlayerAlive(true));
    world.run(map::place_player_in_first_room);

    world.run(spawn::spawn_monsters_in_rooms);

    world.add_unique(MonsterTurns::new());

    world.add_unique(MeleeQueue::new());
    world.add_unique(DamageQueue::new());
    world.add_unique(DeadEntities::new());

    world.run(recalculate_fields_of_view);

    let settings = RunSettings {
        title: "Ruggle".to_string(),
        grid_size: [80, 48],
        font_path: PathBuf::from("assets/terminal-8x8.png"),
        min_fps: 30,
        max_fps: 60,
    };

    ruggle::run(
        settings,
        |mut inputs| {
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

                if world.run(player_is_alive) && !world.run(monster_turns_empty) {
                    RunControl::Update
                } else {
                    RunControl::WaitForEvent
                }
            } else if player_is_dead_input(&mut inputs) {
                RunControl::Quit
            } else {
                RunControl::WaitForEvent
            }
        },
        |mut grid| {
            grid.clear();
            draw_map(&world, &mut grid);
            draw_renderables(&world, &mut grid);
            draw_ui(&world, &mut grid);
        },
    );
}
