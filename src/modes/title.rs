use shipyard::{AllStoragesViewMut, Get, UniqueView, UniqueViewMut, View, ViewMut, World};

use crate::{
    components::{CombatStats, Experience, FieldOfView},
    experience::{self, Difficulty},
    gamekey::{self, GameKey},
    gamesym::GameSym,
    item::PickUpHint,
    map::{self, Map},
    menu_memory::MenuMemory,
    message::Messages,
    player::{self, PlayerAlive, PlayerId},
    saveload, spawn,
    ui::{self, Options},
    vision, BaseEquipmentLevel, GameSeed, TurnCount, Wins,
};
use ruggrogue::{
    util::{Color, Size},
    InputBuffer, InputEvent, KeyMods, TileGrid, Tileset,
};

use super::{
    dungeon::DungeonMode,
    message_box::{MessageBoxMode, MessageBoxModeResult},
    options_menu::{OptionsMenuMode, OptionsMenuModeResult},
    yes_no_dialog::{YesNoDialogMode, YesNoDialogModeResult},
    ModeControl, ModeResult, ModeUpdate,
};

const LOGO_GRID: usize = 0;
const VERSION_GRID: usize = 1;
const SOURCE_GRID: usize = 2;
const MENU_GRID: usize = 3;

const SOURCE_STR: &str = "tung.github.io/ruggrogue/";
const VERSION_STR: &str = "v1.0.1";
const LOGO_STR: &str = "░░░░░░  ░░  ░░   ░░░░    ░░░░
 ▒▒  ▒▒ ▒▒  ▒▒  ▒▒  ▒▒  ▒▒  ▒▒
 ▓▓  ▓▓ ▓▓  ▓▓ ▓▓      ▓▓
 █████  ██  ██ ██      ██
 ▓▓▓▓   ▓▓  ▓▓ ▓▓  ▓▓▓ ▓▓  ▓▓▓
 ▒▒ ▒▒  ▒▒  ▒▒  ▒▒  ▒▒  ▒▒  ▒▒
░░░  ░░ ░░░░░░   ░░░░░   ░░░░░

        ░░░░░░    ░░░░     ░░░░  ░░  ░░ ░░░░░░░
         ▒▒  ▒▒  ▒▒  ▒▒   ▒▒  ▒▒ ▒▒  ▒▒  ▒▒   ▒
         ▓▓  ▓▓ ▓▓    ▓▓ ▓▓      ▓▓  ▓▓  ▓▓ ▓
         █████  ██    ██ ██      ██  ██  ████
         ▓▓▓▓   ▓▓    ▓▓ ▓▓  ▓▓▓ ▓▓  ▓▓  ▓▓ ▓
         ▒▒ ▒▒   ▒▒  ▒▒   ▒▒  ▒▒ ▒▒  ▒▒  ▒▒   ▒
        ░░░  ░░   ░░░░     ░░░░░ ░░░░░░ ░░░░░░░
";

pub enum TitleModeResult {
    AppQuit,
}

pub enum TitleAction {
    NewGame,
    LoadGame,
    Options,
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    Quit,
}

impl TitleAction {
    fn label(&self) -> &'static str {
        match self {
            TitleAction::NewGame => "New Game",
            TitleAction::LoadGame => "Load Game",
            TitleAction::Options => "Options",
            TitleAction::Quit => "Quit",
        }
    }
}

const ALL_TITLE_ACTIONS: [TitleAction; 4] = [
    TitleAction::NewGame,
    TitleAction::LoadGame,
    TitleAction::Options,
    TitleAction::Quit,
];

fn print_game_seed(game_seed: UniqueView<GameSeed>) {
    println!("Game seed: {}", game_seed.0);
}

pub fn new_game_setup(world: &World, new_game_plus: bool) {
    world.borrow::<UniqueViewMut<MenuMemory>>().reset();
    world.borrow::<UniqueViewMut<Messages>>().reset();
    world.borrow::<UniqueViewMut<Map>>().clear();
    world.borrow::<UniqueViewMut<PlayerAlive>>().0 = true;

    if new_game_plus {
        // Set base equipment level based on difficulty level at the end of the previous game.
        {
            let difficulty_id = world.borrow::<UniqueView<Difficulty>>().id;
            let experiences = world.borrow::<View<Experience>>();
            let difficulty_exp = experiences.get(difficulty_id);
            world.borrow::<UniqueViewMut<BaseEquipmentLevel>>().0 += difficulty_exp.level - 1;
        }

        // Increment turn count and depth.
        world.borrow::<UniqueViewMut<TurnCount>>().0 += 1;
        world.borrow::<UniqueViewMut<Map>>().depth += 1;

        let player_id = world.borrow::<UniqueView<PlayerId>>().0;

        // Restore player to full health.
        if let Ok(stats) = (&mut world.borrow::<ViewMut<CombatStats>>()).try_get(player_id) {
            stats.hp = stats.max_hp;
        }

        // Mark field of view as dirty.
        if let Ok(fov) = (&mut world.borrow::<ViewMut<FieldOfView>>()).try_get(player_id) {
            fov.dirty = true;
        }

        world
            .borrow::<UniqueViewMut<Messages>>()
            .add("Welcome back to RuggRogue!".into());
    } else {
        world.run(print_game_seed);

        // Reset wins and base equipment level.
        world.borrow::<UniqueViewMut<Wins>>().0 = 0;
        world.borrow::<UniqueViewMut<BaseEquipmentLevel>>().0 = 0;

        // Reset turn count and depth.
        world.borrow::<UniqueViewMut<TurnCount>>().0 = 1;
        world.borrow::<UniqueViewMut<Map>>().depth = 1;

        // Replace the old player with a fresh one.
        let player_id = world.borrow::<UniqueView<PlayerId>>().0;
        spawn::despawn_entity(&mut world.borrow::<AllStoragesViewMut>(), player_id);
        let new_player_id = world.run(spawn::spawn_player);
        world.borrow::<UniqueViewMut<PlayerId>>().0 = new_player_id;

        // Show the hint for the pick up item key.
        world.borrow::<UniqueViewMut<PickUpHint>>().0 = true;

        world
            .borrow::<UniqueViewMut<Messages>>()
            .add("Welcome to RuggRogue!".into());
    }

    // Replace old difficulty tracker with a fresh one.
    {
        let difficulty_id = world.borrow::<UniqueView<Difficulty>>().id;
        spawn::despawn_entity(&mut world.borrow::<AllStoragesViewMut>(), difficulty_id);
        let new_difficulty = Difficulty::new(world.run(spawn::spawn_difficulty));
        world
            .borrow::<UniqueViewMut<Difficulty>>()
            .replace(new_difficulty);
    }

    if let Some(victory_pos) = world.run(map::generate_rooms_and_corridors) {
        spawn::spawn_present(world, victory_pos);
    }
    world.run(player::add_coords_to_players);
    world.run(map::place_player_in_first_room);
    spawn::fill_rooms_with_spawns(world);
    world.run(experience::calc_exp_for_next_depth);
    world.run(vision::recalculate_fields_of_view);

    player::describe_player_pos(world);
}

pub fn post_game_cleanup(world: &World, reset_seed: bool) {
    world.run(player::remove_coords_from_players);
    world.run(spawn::despawn_coord_entities);

    if reset_seed {
        // Ensure the next game uses a new seed.
        world.borrow::<UniqueViewMut<GameSeed>>().0 = rand::random();
    }
}

pub struct TitleMode {
    actions: Vec<TitleAction>,
    menu_width: u32,
    menu_height: u32,
    selection: usize,
}

/// Show the title screen of the game with a menu that leads into the game proper.
impl TitleMode {
    pub fn new() -> Self {
        let mut actions = vec![TitleAction::NewGame];

        // There's no obvious way to get Emscripten to load the IndexedDB filesystem in time to
        // realize that a save file exists, so always include the Load Game option for it and just
        // check if there really is a save file when the option is chosen instead.
        if cfg!(target_os = "emscripten") || saveload::save_file_exists() {
            actions.push(TitleAction::LoadGame);
        }

        actions.push(TitleAction::Options);

        #[cfg(not(target_arch = "wasm32"))]
        actions.push(TitleAction::Quit);

        let selection = if saveload::save_file_exists() {
            actions
                .iter()
                .position(|a| matches!(*a, TitleAction::LoadGame))
                .unwrap_or(0)
        } else {
            0
        };

        Self {
            actions,
            menu_width: ALL_TITLE_ACTIONS
                .iter()
                .map(|a| a.label().len())
                .max()
                .unwrap_or(0) as u32,
            menu_height: ALL_TITLE_ACTIONS.len() as u32,
            selection,
        }
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
        let tileset = &tilesets.get(font as usize).unwrap_or(&tilesets[0]);

        let new_logo_size = Size {
            w: LOGO_STR
                .lines()
                .map(str::chars)
                .map(Iterator::count)
                .max()
                .unwrap_or(1) as u32,
            h: LOGO_STR.chars().filter(|c| *c == '\n').count() as u32,
        };
        let new_version_size = Size {
            w: VERSION_STR.len() as u32,
            h: 1,
        };
        let new_source_size = Size {
            w: SOURCE_STR.len() as u32,
            h: 1,
        };
        let new_menu_size = Size {
            w: self.menu_width,
            h: self.menu_height,
        };

        if !grids.is_empty() {
            grids[LOGO_GRID].resize(new_logo_size);
            grids[VERSION_GRID].resize(new_version_size);
            grids[SOURCE_GRID].resize(new_source_size);
            grids[MENU_GRID].resize(new_menu_size);
        } else {
            grids.push(TileGrid::new(new_logo_size, tilesets, font as usize));
            grids.push(TileGrid::new(new_version_size, tilesets, font as usize));
            grids.push(TileGrid::new(new_source_size, tilesets, font as usize));
            grids.push(TileGrid::new(new_menu_size, tilesets, font as usize));
            grids[LOGO_GRID].view.clear_color = None;
            grids[VERSION_GRID].view.clear_color = None;
            grids[SOURCE_GRID].view.clear_color = None;
            grids[MENU_GRID].view.clear_color = None;
        }

        let (logo_grid, grids) = grids.split_first_mut().unwrap(); // LOGO_GRID
        let (version_grid, grids) = grids.split_first_mut().unwrap(); // VERSION_GRID
        let (source_grid, grids) = grids.split_first_mut().unwrap(); // SOURCE_GRID
        let (menu_grid, _) = grids.split_first_mut().unwrap(); // MENU_GRID

        // Set fonts.
        logo_grid.set_tileset(tilesets, font as usize);
        version_grid.set_tileset(tilesets, font as usize);
        source_grid.set_tileset(tilesets, font as usize);
        menu_grid.set_tileset(tilesets, font as usize);

        let combined_px_height =
            (new_logo_size.h + new_menu_size.h) * tileset.tile_height() * text_zoom;

        // Logo goes in the center top third.
        logo_grid.view.size.w = new_logo_size.w * tileset.tile_width() * text_zoom;
        logo_grid.view.size.h = new_logo_size.h * tileset.tile_height() * text_zoom;
        logo_grid.view.pos.x = (window_size.w - logo_grid.view.size.w) as i32 / 2;
        logo_grid.view.pos.y = (window_size.h.saturating_sub(combined_px_height) / 3) as i32;

        // Menu goes in the left-center bottom third.
        menu_grid.view.size.w = new_menu_size.w * tileset.tile_width() * text_zoom;
        menu_grid.view.size.h = new_menu_size.h * tileset.tile_height() * text_zoom;
        menu_grid.view.pos.x = (window_size.w / 2 - menu_grid.view.size.w) as i32;
        menu_grid.view.pos.y =
            (logo_grid.view.size.h + window_size.h.saturating_sub(combined_px_height) * 2 / 3)
                .min(window_size.h.saturating_sub(menu_grid.view.size.h)) as i32;

        // Version goes in the bottom left corner of the screen.
        version_grid.view.size.w = new_version_size.w * tileset.tile_width() * text_zoom;
        version_grid.view.size.h = new_version_size.h * tileset.tile_height() * text_zoom;
        version_grid.view.pos.x = 0;
        version_grid.view.pos.y = (window_size.h - version_grid.view.size.h) as i32;

        // Source goes in the bottom right corner of the screen.
        source_grid.view.size.w = new_source_size.w * tileset.tile_width() * text_zoom;
        source_grid.view.size.h = new_source_size.h * tileset.tile_height() * text_zoom;
        source_grid.view.pos.x = (window_size.w - source_grid.view.size.w) as i32;
        source_grid.view.pos.y = (window_size.h - source_grid.view.size.h) as i32;

        // Set all grids to current text zoom.
        logo_grid.view.zoom = text_zoom;
        version_grid.view.zoom = text_zoom;
        source_grid.view.zoom = text_zoom;
        menu_grid.view.zoom = text_zoom;
    }

    pub fn update(
        &mut self,
        world: &World,
        inputs: &mut InputBuffer,
        _grids: &[TileGrid<GameSym>],
        pop_result: &Option<ModeResult>,
    ) -> (ModeControl, ModeUpdate) {
        if let Some(result) = pop_result {
            return match result {
                ModeResult::MessageBoxModeResult(result) => match result {
                    MessageBoxModeResult::AppQuit => (
                        ModeControl::Pop(TitleModeResult::AppQuit.into()),
                        ModeUpdate::Immediate,
                    ),
                    MessageBoxModeResult::Done => (ModeControl::Stay, ModeUpdate::WaitForEvent),
                },
                ModeResult::OptionsMenuModeResult(result) => match result {
                    OptionsMenuModeResult::AppQuit => (
                        ModeControl::Pop(TitleModeResult::AppQuit.into()),
                        ModeUpdate::Immediate,
                    ),
                    OptionsMenuModeResult::Closed => (ModeControl::Stay, ModeUpdate::WaitForEvent),
                    OptionsMenuModeResult::ReallyQuit => unreachable!(),
                },
                ModeResult::YesNoDialogModeResult(result) => match result {
                    YesNoDialogModeResult::AppQuit => (
                        ModeControl::Pop(TitleModeResult::AppQuit.into()),
                        ModeUpdate::Immediate,
                    ),
                    YesNoDialogModeResult::Yes => {
                        saveload::delete_save_file();

                        // Remove the load game option.
                        self.actions
                            .retain(|a| !matches!(*a, TitleAction::LoadGame));

                        // Adjust selection if needed.
                        if let Some(pos) = self
                            .actions
                            .iter()
                            .position(|a| matches!(*a, TitleAction::NewGame))
                        {
                            self.selection = pos;
                        } else {
                            self.selection =
                                self.selection.min(self.actions.len().saturating_sub(1));
                        }

                        inputs.clear_input();
                        (ModeControl::Stay, ModeUpdate::Immediate)
                    }
                    YesNoDialogModeResult::No => (ModeControl::Stay, ModeUpdate::WaitForEvent),
                },
                _ => unreachable!(),
            };
        }

        inputs.prepare_input();

        match inputs.get_input() {
            Some(InputEvent::AppQuit) => {
                return (
                    ModeControl::Pop(TitleModeResult::AppQuit.into()),
                    ModeUpdate::Immediate,
                );
            }

            Some(InputEvent::Press(keycode)) => {
                match gamekey::from_keycode(keycode, inputs.get_mods(KeyMods::SHIFT)) {
                    GameKey::Up => {
                        if self.selection > 0 {
                            self.selection -= 1;
                        } else {
                            self.selection = self.actions.len().saturating_sub(1);
                        }
                    }
                    GameKey::Down => {
                        if self.selection < self.actions.len().saturating_sub(1) {
                            self.selection += 1;
                        } else {
                            self.selection = 0;
                        }
                    }
                    GameKey::Cancel => {
                        if let Some(quit_pos) = self
                            .actions
                            .iter()
                            .position(|a| matches!(*a, TitleAction::Quit))
                        {
                            self.selection = quit_pos;
                        }
                    }
                    GameKey::Confirm => {
                        assert!(self.selection < self.actions.len());

                        match self.actions[self.selection] {
                            TitleAction::NewGame => {
                                if saveload::save_file_exists() {
                                    inputs.clear_input();
                                    return (
                                        ModeControl::Push(
                                            YesNoDialogMode::new(
                                                "Save data already exists.  Delete it?".into(),
                                                false,
                                            )
                                            .into(),
                                        ),
                                        ModeUpdate::Immediate,
                                    );
                                } else {
                                    new_game_setup(world, false);
                                    inputs.clear_input();
                                    return (
                                        ModeControl::Switch(DungeonMode::new().into()),
                                        ModeUpdate::Immediate,
                                    );
                                }
                            }
                            TitleAction::LoadGame => {
                                if saveload::save_file_exists() {
                                    match saveload::load_game(world) {
                                        Ok(_) => {
                                            world.run(print_game_seed);

                                            // Don't show pick up key hint to returning players.
                                            world.borrow::<UniqueViewMut<PickUpHint>>().0 = false;

                                            inputs.clear_input();
                                            return (
                                                ModeControl::Switch(DungeonMode::new().into()),
                                                ModeUpdate::Immediate,
                                            );
                                        }
                                        Err(e) => {
                                            let mut msg = vec![
                                                "Failed to load game:".to_string(),
                                                "".to_string(),
                                            ];

                                            msg.extend(
                                                ruggrogue::word_wrap(&format!("{}", e), 78)
                                                    .map(String::from),
                                            );

                                            inputs.clear_input();
                                            return (
                                                ModeControl::Push(MessageBoxMode::new(msg).into()),
                                                ModeUpdate::Immediate,
                                            );
                                        }
                                    }
                                } else {
                                    inputs.clear_input();
                                    return (
                                        ModeControl::Push(
                                            MessageBoxMode::new(vec![
                                                "No save file found.".to_string()
                                            ])
                                            .into(),
                                        ),
                                        ModeUpdate::Immediate,
                                    );
                                }
                            }
                            TitleAction::Options => {
                                inputs.clear_input();
                                return (
                                    ModeControl::Push(OptionsMenuMode::new(false).into()),
                                    ModeUpdate::Immediate,
                                );
                            }
                            TitleAction::Quit => {
                                return (
                                    ModeControl::Pop(TitleModeResult::AppQuit.into()),
                                    ModeUpdate::Immediate,
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }

            _ => {}
        }

        (ModeControl::Stay, ModeUpdate::WaitForEvent)
    }

    pub fn draw(&self, _world: &World, grids: &mut [TileGrid<GameSym>], active: bool) {
        let (logo_grid, grids) = grids.split_first_mut().unwrap(); // LOGO_GRID
        let (version_grid, grids) = grids.split_first_mut().unwrap(); // VERSION_GRID
        let (source_grid, grids) = grids.split_first_mut().unwrap(); // SOURCE_GRID
        let (menu_grid, _) = grids.split_first_mut().unwrap(); // MENU_GRID
        let fg = Color::WHITE;
        let bg = Color::BLACK;
        let selected_bg = ui::SELECTED_BG;

        if active {
            logo_grid.view.color_mod = Color::WHITE;
            version_grid.view.color_mod = Color::WHITE;
            source_grid.view.color_mod = Color::WHITE;
            menu_grid.view.color_mod = Color::WHITE;
        } else {
            logo_grid.view.color_mod = Color::GRAY;
            version_grid.view.color_mod = Color::GRAY;
            source_grid.view.color_mod = Color::GRAY;
            menu_grid.view.color_mod = Color::GRAY;
        }

        for (i, logo_line) in LOGO_STR.lines().enumerate() {
            logo_grid.print_color((0, i as i32), logo_line, true, Color::ORANGE, bg);
        }

        version_grid.print_color((0, 0), VERSION_STR, true, Color::GRAY, bg);

        source_grid.print_color((0, 0), SOURCE_STR, true, Color::GRAY, bg);

        menu_grid.clear();
        for (i, action) in self.actions.iter().enumerate() {
            menu_grid.print_color(
                (0, i as i32),
                action.label(),
                true,
                fg,
                if i == self.selection { selected_bg } else { bg },
            );
        }
    }
}
