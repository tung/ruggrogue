mod bitgrid;
mod components;
mod damage;
mod gamekey;
mod gamesym;
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
use std::{collections::HashMap, path::PathBuf};

use crate::{
    damage::DeadEntities,
    gamesym::GameSym,
    map::Map,
    message::Messages,
    modes::{dungeon::DungeonMode, ModeStack},
    monster::MonsterTurns,
    player::{PlayerAlive, PlayerId},
    ui::Options,
};
use ruggle::{RunSettings, TilesetInfo};

pub struct RuggleRng(Pcg64Mcg);

fn main() {
    let world = World::new();

    world.add_unique(Options {
        map_zoom: 1,
        text_zoom: 1,
    });
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
        window_size: (800, 600).into(),
        min_window_size: (640, 192).into(),
        fps: 60,
        tileset_infos: vec![
            TilesetInfo::<GameSym> {
                image_path: PathBuf::from("assets/gohufont-8x14.png"),
                tile_size: (8, 14).into(),
                tile_start: (0, 0).into(),
                tile_gap: (0, 0).into(),
                font_map: TilesetInfo::<GameSym>::map_code_page_437(),
                symbol_map: HashMap::new(),
            },
            TilesetInfo::<GameSym> {
                image_path: PathBuf::from("assets/terminal-8x8.png"),
                tile_size: (8, 8).into(),
                tile_start: (0, 0).into(),
                tile_gap: (0, 0).into(),
                font_map: TilesetInfo::<GameSym>::map_code_page_437(),
                symbol_map: HashMap::new(),
            },
            gamesym::urizen_tileset_info(),
        ],
    };

    ruggle::run(settings, |inputs, layers, tilesets, window_size| {
        mode_stack.update(&world, inputs, layers, tilesets, window_size)
    });
}
