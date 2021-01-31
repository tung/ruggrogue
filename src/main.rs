mod components;
mod damage;
mod map;
mod message;
mod modes;
mod monster;
mod player;
mod rect;
mod spawn;
mod ui;
mod vision;

use rand::{thread_rng, SeedableRng};
use rand_pcg::Pcg64Mcg;
use shipyard::World;
use std::{cell::RefCell, path::PathBuf};

use crate::modes::{dungeon::DungeonMode, ModeStack};
use ruggle::RunSettings;

pub struct PlayerAlive(bool);

pub struct RuggleRng(Pcg64Mcg);

fn main() {
    let world = World::new();

    world.add_unique(RuggleRng(Pcg64Mcg::from_rng(thread_rng()).unwrap()));
    world.add_unique(message::Messages::new(4));
    world.add_unique(map::Map::new(80, 50));
    world.add_unique(components::PlayerId(world.run(spawn::spawn_player)));
    world.add_unique(PlayerAlive(true));
    world.add_unique(monster::MonsterTurns::new());
    world.add_unique(damage::MeleeQueue::new());
    world.add_unique(damage::DamageQueue::new());
    world.add_unique(damage::DeadEntities::new());

    let mode_stack = RefCell::new(ModeStack::new(vec![DungeonMode::new(&world).into()]));

    let settings = RunSettings {
        title: "Ruggle".to_string(),
        grid_size: [80, 48],
        font_path: PathBuf::from("assets/terminal-8x8.png"),
        min_fps: 30,
        max_fps: 60,
    };

    ruggle::run(
        settings,
        |inputs| mode_stack.borrow_mut().update(&world, inputs),
        |grid| {
            grid.clear();
            mode_stack.borrow().draw(&world, grid);
        },
    );
}
