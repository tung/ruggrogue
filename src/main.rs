mod components;
mod damage;
mod gamekey;
mod item;
mod map;
mod message;
mod modes;
mod monster;
mod player;
mod render;
mod spawn;
mod ui;
mod vision;

use rand::SeedableRng;
use rand_pcg::Pcg64Mcg;
use shipyard::World;
use std::{cell::RefCell, path::PathBuf};

use crate::{
    damage::DeadEntities,
    map::Map,
    message::Messages,
    modes::{dungeon::DungeonMode, ModeStack},
    monster::MonsterTurns,
    player::{PlayerAlive, PlayerId},
};
use ruggle::RunSettings;

pub struct RuggleRng(Pcg64Mcg);

fn main() {
    let world = World::new();

    world.add_unique(RuggleRng(Pcg64Mcg::from_rng(rand::thread_rng()).unwrap()));
    world.add_unique(Messages::new(4));
    world.add_unique(Map::new(80, 50));
    world.add_unique(PlayerId(world.run(spawn::spawn_player)));
    world.add_unique(PlayerAlive(true));
    world.add_unique(MonsterTurns::new());
    world.add_unique(DeadEntities::new());

    let mode_stack = RefCell::new(ModeStack::new(vec![DungeonMode::new(&world).into()]));

    let settings = RunSettings {
        title: "Ruggle".to_string(),
        grid_size: [80, 48],
        min_grid_size: [80, 24],
        font_path: PathBuf::from("assets/terminal-8x8.png"),
        fps: 60,
    };

    ruggle::run(
        &settings,
        |inputs| mode_stack.borrow_mut().update(&world, inputs),
        |grid| {
            grid.clear();
            mode_stack.borrow().draw(&world, grid);
        },
    );
}
