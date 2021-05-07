mod bitgrid;
mod chunked;
mod components;
mod damage;
mod experience;
mod gamekey;
mod gamesym;
mod hunger;
mod item;
mod magicnum;
mod map;
mod message;
mod modes;
mod monster;
mod player;
mod render;
mod spawn;
mod ui;
mod vision;

use shipyard::World;
use std::{collections::HashMap, path::PathBuf};

use crate::{
    gamesym::GameSym,
    map::Map,
    message::Messages,
    modes::{dungeon::DungeonMode, ModeStack},
    monster::MonsterTurns,
    player::{PlayerAlive, PlayerId},
    ui::Options,
};
use ruggle::{RunSettings, TilesetInfo};

pub struct GameSeed(u64);

pub struct TurnCount(u64);

fn main() {
    let world = World::new();
    let game_seed = std::env::args()
        .nth(1)
        .and_then(|arg| arg.as_str().parse().ok())
        .unwrap_or_else(rand::random);

    println!("Game seed: {}", game_seed);

    world.add_unique(Options {
        tileset: 2,
        font: 0,
        map_zoom: 1,
        text_zoom: 1,
    });
    world.add_unique(GameSeed(game_seed));
    world.add_unique(TurnCount(0));
    world.add_unique(Messages::new(4));
    world.add_unique(Map::new(80, 50));
    world.add_unique(PlayerId(world.run(spawn::spawn_player)));
    world.add_unique(PlayerAlive(true));
    world.add_unique(MonsterTurns::new());

    let mut mode_stack = ModeStack::new(vec![DungeonMode::new(&world).into()]);

    let settings = RunSettings {
        title: "Ruggle".into(),
        window_size: (1000, 600).into(),
        min_window_size: (640, 192).into(),
        fps: 30,
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
