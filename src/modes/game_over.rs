use shipyard::{Get, UniqueView, View, World};

use crate::{
    components::{CombatStats, Equipment, Experience, HurtBy, Inventory, Name, Tally},
    gamekey::{self, GameKey},
    gamesym::GameSym,
    map::Map,
    player::{PlayerAlive, PlayerId},
    ui::Options,
    TurnCount, Wins,
};
use ruggrogue::{
    util::{Color, Size},
    InputBuffer, InputEvent, KeyMods, TileGrid, Tileset,
};

use super::{
    dungeon::DungeonMode,
    title::{self, TitleMode},
    ModeControl, ModeResult, ModeUpdate,
};

pub enum GameOverModeResult {
    AppQuit,
}

pub struct GameOverMode;

impl GameOverMode {
    pub fn new() -> Self {
        Self {}
    }

    pub fn prepare_grids(
        &self,
        world: &World,
        grids: &mut Vec<TileGrid<GameSym>>,
        tilesets: &[Tileset<GameSym>],
        window_size: Size,
    ) {
        let Options {
            font, text_zoom, ..
        } = *world.borrow::<UniqueView<Options>>();
        let new_grid_size = Size { w: 34, h: 19 };

        if !grids.is_empty() {
            grids[0].resize(new_grid_size);
        } else {
            grids.push(TileGrid::new(new_grid_size, tilesets, font as usize));
            grids[0].view.clear_color = Some(Color::BLACK);
        }

        grids[0].set_tileset(tilesets, font as usize);
        grids[0].view_centered(tilesets, text_zoom, (0, 0).into(), window_size);
        grids[0].view.zoom = text_zoom;
    }

    pub fn update(
        &mut self,
        world: &World,
        inputs: &mut InputBuffer,
        _grids: &[TileGrid<GameSym>],
        _pop_result: &Option<ModeResult>,
    ) -> (ModeControl, ModeUpdate) {
        inputs.prepare_input();

        if let Some(InputEvent::AppQuit) = inputs.get_input() {
            return (
                ModeControl::Pop(GameOverModeResult::AppQuit.into()),
                ModeUpdate::Immediate,
            );
        } else if let Some(InputEvent::Press(keycode)) = inputs.get_input() {
            let key = gamekey::from_keycode(keycode, inputs.get_mods(KeyMods::SHIFT));
            if matches!(key, GameKey::Confirm | GameKey::Cancel) {
                let player_alive = world.borrow::<UniqueView<PlayerAlive>>().0;

                title::post_game_cleanup(world, !player_alive);
                if player_alive {
                    title::new_game_setup(world, true);
                }

                inputs.clear_input();
                return (
                    ModeControl::Switch(if player_alive {
                        // Jump straight into new game plus.
                        DungeonMode::new().into()
                    } else {
                        TitleMode::new().into()
                    }),
                    ModeUpdate::Immediate,
                );
            }
        }

        (ModeControl::Stay, ModeUpdate::WaitForEvent)
    }

    pub fn draw(&self, world: &World, grids: &mut [TileGrid<GameSym>], active: bool) {
        const DATA_X: i32 = 15;
        let grid = &mut grids[0];
        let data_fg = Color::YELLOW;
        let bg = Color::BLACK;
        let player_alive = world.borrow::<UniqueView<PlayerAlive>>().0;

        grid.view.color_mod = if active { Color::WHITE } else { Color::GRAY };

        if player_alive {
            grid.print_color(
                (1, 0),
                "* * *  Y O U   W I N !  * * *",
                true,
                Color::GREEN,
                bg,
            );
        } else {
            grid.print_color(
                (0, 0),
                "* * * You have been defeated * * *",
                true,
                Color::MAGENTA,
                bg,
            );
        }

        let player_id = world.borrow::<UniqueView<PlayerId>>();

        if player_alive {
            let wins = world.borrow::<UniqueView<Wins>>().0;

            if wins < 2 {
                grid.print((0, 2), "Your birthday present is saved!");
            } else {
                grid.print((9, 2), "Wins:");
                grid.print_color((DATA_X, 2), wins.to_string().as_str(), true, data_fg, bg);
            }
        } else {
            let names = world.borrow::<View<Name>>();
            let hurt_bys = world.borrow::<View<HurtBy>>();
            let defeated_by = match hurt_bys.try_get(player_id.0) {
                Ok(HurtBy::Someone(hurter)) => {
                    if *hurter == player_id.0 {
                        "an overinflated ego"
                    } else {
                        names.get(*hurter).0.as_str()
                    }
                }
                Ok(HurtBy::Starvation) => "starvation",
                Err(_) => "perfectly natural causes",
            };

            grid.print((2, 2), "Defeated by:");
            grid.print_color((DATA_X, 2), defeated_by, true, data_fg, bg);
        }

        {
            let exps = world.borrow::<View<Experience>>();
            let player_exp = exps.get(player_id.0);

            grid.print((8, 4), "Level:");
            grid.print_color(
                (DATA_X, 4),
                player_exp.level.to_string().as_str(),
                true,
                data_fg,
                bg,
            );
            grid.print((3, 5), "Experience:");
            grid.print_color(
                (DATA_X, 5),
                (player_exp.base + player_exp.exp).to_string().as_str(),
                true,
                data_fg,
                bg,
            );
        }

        {
            let combat_stats = world.borrow::<View<CombatStats>>();
            let player_stats = combat_stats.get(player_id.0);

            grid.print((7, 6), "Health:");
            grid.print_color(
                (DATA_X, 6),
                &format!("{} / {}", player_stats.hp, player_stats.max_hp),
                true,
                data_fg,
                bg,
            );
            grid.print((7, 7), "Attack:");
            grid.print_color(
                (DATA_X, 7),
                &format!("{:+.0}", player_stats.attack),
                true,
                data_fg,
                bg,
            );
            grid.print((6, 8), "Defense:");
            grid.print_color(
                (DATA_X, 8),
                &format!("{:+.0}", player_stats.defense),
                true,
                data_fg,
                bg,
            );
        }

        grid.print((8, 9), "Depth:");
        grid.print_color(
            (DATA_X, 9),
            world.borrow::<UniqueView<Map>>().depth.to_string().as_str(),
            true,
            data_fg,
            bg,
        );

        grid.print((8, 10), "Turns:");
        grid.print_color(
            (DATA_X, 10),
            world
                .borrow::<UniqueView<TurnCount>>()
                .0
                .to_string()
                .as_str(),
            true,
            data_fg,
            bg,
        );

        grid.print((0, 12), "Items carried:");
        grid.print_color(
            (DATA_X, 12),
            world
                .borrow::<View<Inventory>>()
                .get(player_id.0)
                .items
                .len()
                .to_string()
                .as_str(),
            true,
            data_fg,
            bg,
        );

        {
            let equipments = world.borrow::<View<Equipment>>();
            let names = world.borrow::<View<Name>>();
            let player_equipment = equipments.get(player_id.0);

            grid.print((7, 13), "Weapon:");
            grid.print_color(
                (DATA_X, 13),
                player_equipment
                    .weapon
                    .map(|w| names.get(w).0.as_str())
                    .unwrap_or("nothing"),
                true,
                data_fg,
                bg,
            );
            grid.print((8, 14), "Armor:");
            grid.print_color(
                (DATA_X, 14),
                player_equipment
                    .armor
                    .map(|a| names.get(a).0.as_str())
                    .unwrap_or("nothing"),
                true,
                data_fg,
                bg,
            );
        }

        {
            let tallies = world.borrow::<View<Tally>>();
            let player_tally = tallies.get(player_id.0);

            grid.print((1, 16), "Damage dealt:");
            grid.print_color(
                (DATA_X, 16),
                player_tally.damage_dealt.to_string().as_str(),
                true,
                data_fg,
                bg,
            );
            grid.print((1, 17), "Damage taken:");
            grid.print_color(
                (DATA_X, 17),
                player_tally.damage_taken.to_string().as_str(),
                true,
                data_fg,
                bg,
            );
            grid.print((0, 18), "Foes defeated:");
            grid.print_color(
                (DATA_X, 18),
                player_tally.kills.to_string().as_str(),
                true,
                data_fg,
                bg,
            );
        }
    }
}
