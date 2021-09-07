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
mod menu_memory;
mod message;
mod modes;
mod monster;
mod player;
mod render;
mod saveload;
mod spawn;
mod ui;
mod vision;

use serde::{Deserialize, Serialize};
use shipyard::World;
use std::{collections::HashMap, path::PathBuf};

use crate::{
    chunked::Camera,
    experience::Difficulty,
    gamesym::GameSym,
    item::PickUpHint,
    map::Map,
    menu_memory::MenuMemory,
    message::Messages,
    modes::{title::TitleMode, ModeStack},
    monster::MonsterTurns,
    player::{PlayerAlive, PlayerId},
    ui::Options,
};
use ruggrogue::{RunSettings, TilesetInfo};

#[derive(Deserialize, Serialize)]
pub struct GameSeed(u64);

#[derive(Deserialize, Serialize)]
pub struct TurnCount(u64);

#[derive(Deserialize, Serialize)]
pub struct Wins(u32);

#[derive(Deserialize, Serialize)]
pub struct BaseEquipmentLevel(i32);

#[cfg(target_os = "emscripten")]
extern "C" {
    pub fn ruggrogue_sync_idbfs();
}

fn main() {
    let world = World::new();
    let game_seed = std::env::args()
        .nth(1)
        .and_then(|arg| arg.as_str().parse().ok())
        .unwrap_or_else(rand::random);

    world.add_unique(Options {
        tileset: 2,
        font: 0,
        map_zoom: 1,
        text_zoom: 1,
    });
    world.add_unique(GameSeed(game_seed));
    world.add_unique(TurnCount(0));
    world.add_unique(Wins(0));
    world.add_unique(BaseEquipmentLevel(0));
    world.add_unique(Camera::new());
    world.add_unique(Difficulty::new(world.run(spawn::spawn_difficulty)));
    world.add_unique(MenuMemory::new());
    world.add_unique(Messages::new(100));
    world.add_unique(Map::new(80, 50));
    world.add_unique(PickUpHint(true));
    world.add_unique(PlayerId(world.run(spawn::spawn_player)));
    world.add_unique(PlayerAlive(true));
    world.add_unique(MonsterTurns::new());

    let mut mode_stack = ModeStack::new(vec![TitleMode::new().into()]);

    let settings = RunSettings {
        title: "RuggRogue".into(),
        window_size: (896, 560).into(),
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

    ruggrogue::run(settings, |inputs, layers, tilesets, window_size| {
        mode_stack.update(&world, inputs, layers, tilesets, window_size)
    });

    #[cfg(target_os = "emscripten")]
    unsafe {
        ruggrogue_sync_idbfs();
    }
}
