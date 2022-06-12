use shipyard::{Get, UniqueView, UniqueViewMut, View, World};

use crate::{
    chunked::{Camera, ChunkedMapGrid},
    components::{Coord, FieldOfView},
    damage, experience,
    gamesym::GameSym,
    hunger, item,
    map::Map,
    message::Messages,
    monster,
    player::{self, PlayerId, PlayerInputResult},
    render, saveload, ui, vision, TurnCount,
};
use ruggrogue::{
    util::{Color, Position, Size},
    InputBuffer, TileGrid, Tileset,
};

use super::{
    app_quit_dialog::{AppQuitDialogMode, AppQuitDialogModeResult},
    equipment_action::EquipmentAction,
    equipment_shortcut::{EquipmentShortcutMode, EquipmentShortcutModeResult},
    game_over::GameOverMode,
    inventory::{InventoryMode, InventoryModeResult},
    inventory_action::InventoryAction,
    inventory_shortcut::{InventoryShortcutMode, InventoryShortcutModeResult},
    options_menu::{OptionsMenuMode, OptionsMenuModeResult},
    pick_up_menu::{PickUpMenuMode, PickUpMenuModeResult},
    title::{self, TitleMode},
    view_map::{ViewMapMode, ViewMapModeResult},
    yes_no_dialog::{YesNoDialogMode, YesNoDialogModeResult},
    ModeControl, ModeResult, ModeUpdate,
};

pub enum DungeonModeResult {
    Done,
}

pub struct DungeonMode {
    chunked_map_grid: ChunkedMapGrid,
    old_msg_frame_size: Size,
    redraw_msg_frame_grid: bool,
}

fn app_quit_dialog(inputs: &mut InputBuffer) -> (ModeControl, ModeUpdate) {
    inputs.clear_input();
    (
        ModeControl::Push(AppQuitDialogMode::new().into()),
        ModeUpdate::Immediate,
    )
}

fn get_player_fov(player_id: UniqueView<PlayerId>, fovs: View<FieldOfView>) -> (Position, Size) {
    let player_fov = fovs.get(player_id.0);

    (
        Position {
            x: player_fov.center.0 - player_fov.range - 1,
            y: player_fov.center.1 - player_fov.range - 1,
        },
        Size {
            w: 2 * player_fov.range as u32 + 1 + 2,
            h: 2 * player_fov.range as u32 + 1 + 2,
        },
    )
}

fn get_player_pos(player_id: UniqueView<PlayerId>, coords: View<Coord>) -> Position {
    coords.get(player_id.0).0
}

/// The main gameplay mode.  The player can move around and explore the map, fight monsters and
/// perform other actions while alive, directly or indirectly.
impl DungeonMode {
    pub fn new() -> Self {
        Self {
            chunked_map_grid: ChunkedMapGrid::new(),
            old_msg_frame_size: (0, 0).into(),
            redraw_msg_frame_grid: true,
        }
    }

    pub fn prepare_grids(
        &mut self,
        world: &World,
        grids: &mut Vec<TileGrid<GameSym>>,
        tilesets: &[Tileset<GameSym>],
        window_size: Size,
    ) {
        ui::prepare_main_grids(
            &mut self.chunked_map_grid,
            world,
            grids,
            tilesets,
            window_size,
        );

        // Detect changes to message frame grid size and redraw the grid when it changes.
        self.redraw_msg_frame_grid = grids[ui::MSG_FRAME_GRID].width() != self.old_msg_frame_size.w
            || grids[ui::MSG_FRAME_GRID].height() != self.old_msg_frame_size.h;
        self.old_msg_frame_size.w = grids[ui::MSG_FRAME_GRID].width();
        self.old_msg_frame_size.h = grids[ui::MSG_FRAME_GRID].height();
    }

    pub fn update(
        &mut self,
        world: &World,
        inputs: &mut InputBuffer,
        _grids: &[TileGrid<GameSym>],
        pop_result: &Option<ModeResult>,
    ) -> (ModeControl, ModeUpdate) {
        if world.run(player::player_is_alive) {
            let old_player_fov = world.run(get_player_fov);
            let old_player_pos = world.run(get_player_pos);
            let old_depth = world.borrow::<UniqueView<Map>>().depth;
            let time_passed = if let Some(result) = pop_result {
                match result {
                    ModeResult::AppQuitDialogModeResult(result) => match result {
                        AppQuitDialogModeResult::Confirmed => {
                            if let Err(e) = saveload::save_game(world) {
                                eprintln!("Warning: saveload::save_game: {}", e);
                            }
                            return (
                                ModeControl::Pop(DungeonModeResult::Done.into()),
                                ModeUpdate::Immediate,
                            );
                        }
                        AppQuitDialogModeResult::Cancelled => false,
                    },

                    ModeResult::YesNoDialogModeResult(result) => match result {
                        YesNoDialogModeResult::AppQuit => return app_quit_dialog(inputs),
                        YesNoDialogModeResult::Yes => {
                            player::player_do_descend(world);
                            if let Err(e) = saveload::save_game(world) {
                                eprintln!("Warning: saveload::save_game: {}", e);
                            }
                            false
                        }
                        YesNoDialogModeResult::No => false,
                    },

                    ModeResult::OptionsMenuModeResult(result) => match result {
                        OptionsMenuModeResult::AppQuit => return app_quit_dialog(inputs),
                        OptionsMenuModeResult::Closed => false,
                        OptionsMenuModeResult::ReallyQuit => {
                            if let Err(e) = saveload::save_game(world) {
                                eprintln!("Warning: saveload::save_game: {}", e);
                            }
                            title::post_game_cleanup(world, true);
                            inputs.clear_input();
                            return (
                                ModeControl::Switch(TitleMode::new().into()),
                                ModeUpdate::Immediate,
                            );
                        }
                    },

                    ModeResult::PickUpMenuModeResult(result) => match result {
                        PickUpMenuModeResult::AppQuit => return app_quit_dialog(inputs),
                        PickUpMenuModeResult::PickedItem(item_id) => {
                            player::player_pick_up_item(world, *item_id);
                            true
                        }
                        PickUpMenuModeResult::Cancelled => false,
                    },

                    ModeResult::InventoryModeResult(result) => {
                        let player_id = world.borrow::<UniqueView<PlayerId>>().0;

                        match result {
                            InventoryModeResult::AppQuit => return app_quit_dialog(inputs),
                            InventoryModeResult::DoNothing => false,
                            InventoryModeResult::RemoveEquipment(item_id) => {
                                item::remove_equipment(world, player_id, *item_id);
                                true
                            }
                            InventoryModeResult::DropEquipment(item_id) => {
                                item::drop_equipment(world, player_id, *item_id);
                                true
                            }
                            InventoryModeResult::EquipItem(item_id) => {
                                item::equip_item(world, player_id, *item_id);
                                true
                            }
                            InventoryModeResult::UseItem(item_id, target) => {
                                if item::use_item(world, player_id, *item_id, *target) {
                                    inputs.clear_input();
                                    return (
                                        ModeControl::Switch(GameOverMode::new().into()),
                                        ModeUpdate::Immediate,
                                    );
                                }
                                true
                            }
                            InventoryModeResult::DropItem(item_id) => {
                                player::player_drop_item(world, *item_id);
                                true
                            }
                        }
                    }

                    ModeResult::InventoryShortcutModeResult(result) => {
                        let player_id = world.borrow::<UniqueView<PlayerId>>().0;

                        match result {
                            InventoryShortcutModeResult::AppQuit => return app_quit_dialog(inputs),
                            InventoryShortcutModeResult::Cancelled => false,
                            InventoryShortcutModeResult::EquipItem(item_id) => {
                                item::equip_item(world, player_id, *item_id);
                                true
                            }
                            InventoryShortcutModeResult::UseItem(item_id, target) => {
                                if item::use_item(world, player_id, *item_id, *target) {
                                    inputs.clear_input();
                                    return (
                                        ModeControl::Switch(GameOverMode::new().into()),
                                        ModeUpdate::Immediate,
                                    );
                                }
                                true
                            }
                            InventoryShortcutModeResult::DropItem(item_id) => {
                                player::player_drop_item(world, *item_id);
                                true
                            }
                        }
                    }

                    ModeResult::EquipmentShortcutModeResult(result) => {
                        let player_id = world.borrow::<UniqueView<PlayerId>>().0;

                        match result {
                            EquipmentShortcutModeResult::AppQuit => return app_quit_dialog(inputs),
                            EquipmentShortcutModeResult::Cancelled => false,
                            EquipmentShortcutModeResult::RemoveEquipment(item_id) => {
                                item::remove_equipment(world, player_id, *item_id);
                                true
                            }
                            EquipmentShortcutModeResult::DropEquipment(item_id) => {
                                item::drop_equipment(world, player_id, *item_id);
                                true
                            }
                        }
                    }

                    ModeResult::ViewMapModeResult(result) => match result {
                        ViewMapModeResult::AppQuit => return app_quit_dialog(inputs),
                        ViewMapModeResult::Done => false,
                    },

                    _ => unreachable!(),
                }
            } else {
                match player::player_input(world, inputs) {
                    PlayerInputResult::AppQuit => return app_quit_dialog(inputs),
                    PlayerInputResult::NoResult => false,
                    PlayerInputResult::TurnDone => true,
                    PlayerInputResult::ShowOptionsMenu => {
                        inputs.clear_input();
                        return (
                            ModeControl::Push(OptionsMenuMode::new(true).into()),
                            ModeUpdate::Immediate,
                        );
                    }
                    PlayerInputResult::ViewMap => {
                        inputs.clear_input();
                        return (
                            ModeControl::Push(ViewMapMode::new(world).into()),
                            ModeUpdate::Immediate,
                        );
                    }
                    PlayerInputResult::TryDescend => {
                        if world.run(player::player_try_descend) {
                            inputs.clear_input();
                            return (
                                ModeControl::Push(
                                    YesNoDialogMode::new(
                                        "Descend to the next level?".to_string(),
                                        false,
                                    )
                                    .into(),
                                ),
                                ModeUpdate::Immediate,
                            );
                        } else {
                            false
                        }
                    }
                    PlayerInputResult::ShowPickUpMenu => {
                        inputs.clear_input();
                        return (
                            ModeControl::Push(PickUpMenuMode::new(world).into()),
                            ModeUpdate::Immediate,
                        );
                    }
                    PlayerInputResult::ShowInventory => {
                        inputs.clear_input();
                        return (
                            ModeControl::Push(InventoryMode::new(world).into()),
                            ModeUpdate::Immediate,
                        );
                    }
                    PlayerInputResult::ShowInventoryShortcut(key) => {
                        if let Some(action) = InventoryAction::from_key(key) {
                            inputs.clear_input();
                            return (
                                ModeControl::Push(InventoryShortcutMode::new(world, action).into()),
                                ModeUpdate::Immediate,
                            );
                        } else {
                            false
                        }
                    }
                    PlayerInputResult::ShowEquipmentShortcut(key) => {
                        if let Some(action) = EquipmentAction::from_key(key) {
                            inputs.clear_input();
                            return (
                                ModeControl::Push(EquipmentShortcutMode::new(world, action).into()),
                                ModeUpdate::Immediate,
                            );
                        } else {
                            false
                        }
                    }
                }
            };

            if time_passed {
                world.run(damage::handle_dead_entities);
                world.run(experience::gain_levels);
                world.run(vision::recalculate_fields_of_view);
                world.run(monster::enqueue_monster_turns);

                if world.run(player::player_is_alive) {
                    monster::do_monster_turns(world);
                    world.run(damage::handle_dead_entities);
                    world.run(experience::gain_levels);
                    world.run(vision::recalculate_fields_of_view);

                    if world.run(player::player_is_alive) {
                        world.run(hunger::tick_hunger);
                        world.run(damage::handle_dead_entities);
                        world.run(experience::gain_levels);
                        world.run(vision::recalculate_fields_of_view);

                        if world.run(player::player_is_alive) {
                            world.run(damage::clear_hurt_bys);
                            world.borrow::<UniqueViewMut<TurnCount>>().0 += 1;
                            world.borrow::<UniqueViewMut<Messages>>().separator();
                        }
                    }
                }

                // Redraw map chunks containing the player's old and new fields of view.
                let new_player_fov = world.run(get_player_fov);
                self.chunked_map_grid
                    .mark_dirty(old_player_fov.0, old_player_fov.1);
                self.chunked_map_grid
                    .mark_dirty(new_player_fov.0, new_player_fov.1);
            }

            {
                let new_depth = world.borrow::<UniqueView<Map>>().depth;
                let new_player_pos = world.run(get_player_pos);

                // Redraw all map chunks when changing levels.
                if new_depth != old_depth {
                    self.chunked_map_grid.mark_all_dirty();
                }

                if new_depth != old_depth || new_player_pos != old_player_pos {
                    player::describe_player_pos(world);
                }

                // Make the camera follow the player.
                {
                    let mut camera = world.borrow::<UniqueViewMut<Camera>>();
                    camera.0 = new_player_pos;
                }
            }

            (
                ModeControl::Stay,
                if world.run(player::player_is_alive) && world.run(player::player_is_auto_running) {
                    ModeUpdate::Update
                } else {
                    ModeUpdate::WaitForEvent
                },
            )
        } else if player::player_is_dead_input(inputs) {
            (
                ModeControl::Switch(GameOverMode::new().into()),
                ModeUpdate::Immediate,
            )
        } else {
            (ModeControl::Stay, ModeUpdate::WaitForEvent)
        }
    }

    pub fn draw(&mut self, world: &World, grids: &mut [TileGrid<GameSym>], active: bool) {
        let (map_grid, grids) = grids.split_first_mut().unwrap(); // ui::MAP_GRID
        let (status_grid, grids) = grids.split_first_mut().unwrap(); // ui::STATUS_GRID
        let (item_grid, grids) = grids.split_first_mut().unwrap(); // ui::ITEM_GRID
        let (msg_frame_grid, grids) = grids.split_first_mut().unwrap(); // ui::MSG_FRAME_GRID
        let (msg_grid, _) = grids.split_first_mut().unwrap(); // ui::MSG_GRID

        if active {
            map_grid.view.color_mod = Color::WHITE;
            status_grid.view.color_mod = Color::WHITE;
            item_grid.view.color_mod = Color::WHITE;
            msg_frame_grid.view.color_mod = Color::WHITE;
            msg_grid.view.color_mod = Color::WHITE;
        } else {
            map_grid.view.color_mod = Color::GRAY;
            status_grid.view.color_mod = Color::GRAY;
            item_grid.view.color_mod = Color::GRAY;
            msg_frame_grid.view.color_mod = Color::GRAY;
            msg_grid.view.color_mod = Color::GRAY;
        }

        self.chunked_map_grid.draw(world, map_grid);
        render::draw_renderables(&self.chunked_map_grid, world, map_grid);

        if self.redraw_msg_frame_grid {
            ui::draw_msg_frame(msg_frame_grid, false);
        }

        msg_grid.clear();
        ui::draw_ui(world, status_grid, item_grid, msg_grid, None);
    }
}
