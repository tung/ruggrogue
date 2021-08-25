use shipyard::{AllStoragesViewMut, UniqueView, UniqueViewMut, World};

use crate::{
    experience::Difficulty,
    gamekey::{self, GameKey},
    gamesym::GameSym,
    map::{self, Map},
    menu_memory::MenuMemory,
    message::Messages,
    player::{self, PlayerAlive, PlayerId},
    saveload, spawn,
    ui::{self, Options},
    vision, GameSeed, TurnCount,
};
use ruggle::{
    util::{Color, Size},
    InputBuffer, InputEvent, KeyMods, TileGrid, Tileset,
};

use super::{
    dungeon::DungeonMode,
    yes_no_dialog::{YesNoDialogMode, YesNoDialogModeResult},
    ModeControl, ModeResult, ModeUpdate,
};

pub enum TitleModeResult {
    AppQuit,
}

pub enum TitleAction {
    NewGame,
    LoadGame,
    Quit,
}

impl TitleAction {
    fn label(&self) -> &'static str {
        match self {
            TitleAction::NewGame => "New Game",
            TitleAction::LoadGame => "Load Game",
            TitleAction::Quit => "Quit",
        }
    }
}

fn print_game_seed(game_seed: UniqueView<GameSeed>) {
    println!("Game seed: {}", game_seed.0);
}

fn new_game_setup(world: &World) {
    world.run(print_game_seed);

    world.borrow::<UniqueViewMut<MenuMemory>>().reset();
    world.borrow::<UniqueViewMut<Messages>>().reset();
    world.borrow::<UniqueViewMut<TurnCount>>().0 = 1;

    // Replace old difficulty tracker with a fresh one.
    {
        let difficulty_id = world.borrow::<UniqueView<Difficulty>>().id;
        spawn::despawn_entity(&mut world.borrow::<AllStoragesViewMut>(), difficulty_id);
        let new_difficulty = Difficulty::new(world.run(spawn::spawn_difficulty));
        world
            .borrow::<UniqueViewMut<Difficulty>>()
            .replace(new_difficulty);
    }

    // Reset the map state.
    {
        let mut map = world.borrow::<UniqueViewMut<Map>>();
        map.clear();
        map.depth = 1;
    }

    // Replace the old player with a fresh one.
    {
        let player_id = world.borrow::<UniqueView<PlayerId>>().0;
        spawn::despawn_entity(&mut world.borrow::<AllStoragesViewMut>(), player_id);
        let new_player_id = world.run(spawn::spawn_player);
        world.borrow::<UniqueViewMut<PlayerId>>().0 = new_player_id;
    }

    world.borrow::<UniqueViewMut<PlayerAlive>>().0 = true;

    world.run(map::generate_rooms_and_corridors);
    world.run(player::add_coords_to_players);
    world.run(map::place_player_in_first_room);
    spawn::fill_rooms_with_spawns(world);
    world.run(vision::recalculate_fields_of_view);

    world
        .borrow::<UniqueViewMut<Messages>>()
        .add("Welcome to Ruggle!".into());
    player::describe_player_pos(world);
}

pub fn post_game_cleanup(world: &World) {
    world.run(player::remove_coords_from_players);
    world.run(spawn::despawn_coord_entities);

    // Ensure the next game uses a new seed.
    world.borrow::<UniqueViewMut<GameSeed>>().0 = rand::random();
}

pub struct TitleMode {
    actions: Vec<TitleAction>,
    inner_width: u32,
    selection: usize,
}

/// Show the title screen of the game with a menu that leads into the game proper.
impl TitleMode {
    pub fn new() -> Self {
        let mut actions = vec![TitleAction::NewGame];
        if saveload::save_file_exists() {
            actions.push(TitleAction::LoadGame);
        }
        actions.push(TitleAction::Quit);

        let inner_width = actions.iter().map(|a| a.label().len()).max().unwrap_or(0) as u32;
        let selection = actions
            .iter()
            .position(|a| matches!(*a, TitleAction::LoadGame))
            .unwrap_or(0);

        Self {
            actions,
            inner_width,
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
        let new_grid_size = Size {
            w: 4 + self.inner_width,
            h: 4 + self.actions.len() as u32,
        };

        if !grids.is_empty() {
            grids[0].resize(new_grid_size);
        } else {
            grids.push(TileGrid::new(new_grid_size, tilesets, font as usize));
            grids[0].view.clear_color = None;
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
        pop_result: &Option<ModeResult>,
    ) -> (ModeControl, ModeUpdate) {
        if let Some(result) = pop_result {
            return match result {
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
                                    new_game_setup(world);
                                    inputs.clear_input();
                                    return (
                                        ModeControl::Switch(DungeonMode::new().into()),
                                        ModeUpdate::Immediate,
                                    );
                                }
                            }
                            TitleAction::LoadGame => {
                                saveload::load_game(world).unwrap();
                                world.run(print_game_seed);
                                inputs.clear_input();
                                return (
                                    ModeControl::Switch(DungeonMode::new().into()),
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
        let grid = &mut grids[0];
        let fg = Color::WHITE;
        let bg = Color::BLACK;
        let selected_bg = ui::SELECTED_BG;

        grid.view.color_mod = if active { Color::WHITE } else { Color::GRAY };

        grid.draw_box((0, 0), (grid.width(), grid.height()), fg, bg);

        for (i, action) in self.actions.iter().enumerate() {
            grid.print_color(
                (2, 2 + i as i32),
                action.label(),
                true,
                fg,
                if i == self.selection { selected_bg } else { bg },
            );
        }
    }
}
