mod bitgrid;
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
use std::path::PathBuf;

use crate::{
    damage::DeadEntities,
    map::Map,
    message::Messages,
    modes::{dungeon::DungeonMode, ModeStack},
    monster::MonsterTurns,
    player::{PlayerAlive, PlayerId},
};
use ruggle::{FontInfo, RunSettings};

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

    let mut mode_stack = ModeStack::new(vec![DungeonMode::new(&world).into()]);

    let settings = RunSettings {
        title: "Ruggle".into(),
        window_size: (640, 384).into(),
        min_window_size: (640, 192).into(),
        fps: 60,
        font_info: FontInfo {
            image_path: PathBuf::from("assets/terminal-8x8.png"),
            glyph_size: (8, 8).into(),
            font_map: FontInfo::map_code_page_437(),
        },
    };

    ruggle::run(settings, |inputs, layers, font, window_size| {
        mode_stack.update(&world, inputs, layers, font, window_size)
    });
}
