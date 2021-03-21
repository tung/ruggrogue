use shipyard::{UniqueView, UniqueViewMut, World};

use crate::{
    damage, item,
    map::{self, Map},
    message::Messages,
    monster,
    player::{self, PlayerAlive, PlayerId, PlayerInputResult},
    render, spawn, ui, vision,
};
use ruggle::{util::Size, Font, InputBuffer, TileGrid};

use super::{
    app_quit_dialog::{AppQuitDialogMode, AppQuitDialogModeResult},
    inventory::{InventoryMode, InventoryModeResult},
    options_menu::{OptionsMenuMode, OptionsMenuModeResult},
    pick_up_menu::{PickUpMenuMode, PickUpMenuModeResult},
    yes_no_dialog::{YesNoDialogMode, YesNoDialogModeResult},
    ModeControl, ModeResult, ModeUpdate,
};

pub enum DungeonModeResult {
    Done,
}

pub struct DungeonMode;

fn app_quit_dialog(inputs: &mut InputBuffer) -> (ModeControl, ModeUpdate) {
    inputs.clear_input();
    (
        ModeControl::Push(AppQuitDialogMode::new().into()),
        ModeUpdate::Immediate,
    )
}

/// The main gameplay mode.  The player can move around and explore the map, fight monsters and
/// perform other actions while alive, directly or indirectly.
impl DungeonMode {
    pub fn new(world: &World) -> Self {
        world
            .borrow::<UniqueViewMut<Messages>>()
            .add("Welcome to Ruggle!".into());
        world.borrow::<UniqueViewMut<Map>>().depth = 1;
        world.run(map::generate_rooms_and_corridors);
        world.borrow::<UniqueViewMut<PlayerAlive>>().0 = true;
        world.run(map::place_player_in_first_room);
        spawn::fill_rooms_with_spawns(world);
        world.run(vision::recalculate_fields_of_view);

        Self {}
    }

    pub fn prepare_grids(
        &self,
        world: &World,
        grids: &mut Vec<TileGrid>,
        fonts: &[Font],
        window_size: Size,
    ) {
        ui::prepare_main_grids(world, grids, fonts, window_size);
    }

    pub fn update(
        &mut self,
        world: &World,
        inputs: &mut InputBuffer,
        pop_result: &Option<ModeResult>,
    ) -> (ModeControl, ModeUpdate) {
        if world.run(player::player_is_alive) {
            let time_passed = if let Some(result) = pop_result {
                match result {
                    ModeResult::AppQuitDialogModeResult(result) => match result {
                        AppQuitDialogModeResult::Confirmed => {
                            return (
                                ModeControl::Pop(DungeonModeResult::Done.into()),
                                ModeUpdate::Immediate,
                            )
                        }
                        AppQuitDialogModeResult::Cancelled => false,
                    },

                    ModeResult::YesNoDialogModeResult(result) => match result {
                        YesNoDialogModeResult::AppQuit => return app_quit_dialog(inputs),
                        YesNoDialogModeResult::Yes => {
                            player::player_do_descend(world);
                            false
                        }
                        YesNoDialogModeResult::No => false,
                    },

                    ModeResult::OptionsMenuModeResult(result) => match result {
                        OptionsMenuModeResult::AppQuit => return app_quit_dialog(inputs),
                        OptionsMenuModeResult::Closed => false,
                        OptionsMenuModeResult::ReallyQuit => {
                            return (
                                ModeControl::Pop(DungeonModeResult::Done.into()),
                                ModeUpdate::Immediate,
                            )
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

                    ModeResult::InventoryModeResult(result) => match result {
                        InventoryModeResult::AppQuit => return app_quit_dialog(inputs),
                        InventoryModeResult::DoNothing => false,
                        InventoryModeResult::UseItem(item_id, target) => {
                            let player_id =
                                world.run(|player_id: UniqueView<PlayerId>| player_id.0);
                            item::use_item(world, player_id, *item_id, *target);
                            true
                        }
                        InventoryModeResult::DropItem(item_id) => {
                            player::player_drop_item(world, *item_id);
                            true
                        }
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
                            ModeControl::Push(OptionsMenuMode::new().into()),
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
                }
            };

            if time_passed {
                world.run(damage::check_for_dead);
                world.run(damage::delete_dead_entities);
                world.run(vision::recalculate_fields_of_view);
                world.run(monster::enqueue_monster_turns);

                while world.run(player::player_is_alive) && !world.run(monster::monster_turns_empty)
                {
                    monster::do_monster_turns(world);
                    world.run(damage::check_for_dead);
                    world.run(damage::delete_dead_entities);
                    world.run(vision::recalculate_fields_of_view);
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
                ModeControl::Pop(DungeonModeResult::Done.into()),
                ModeUpdate::Update,
            )
        } else {
            (ModeControl::Stay, ModeUpdate::WaitForEvent)
        }
    }

    pub fn draw(&self, world: &World, grids: &mut [TileGrid], active: bool) {
        let (map_grid, grids) = grids.split_first_mut().unwrap(); // ui::MAP_GRID
        let (ui_grid, _) = grids.split_first_mut().unwrap(); // ui::UI_GRID

        if active {
            map_grid.view.color_mod = ui::color::WHITE;
            ui_grid.view.color_mod = ui::color::WHITE;
        } else {
            map_grid.view.color_mod = ui::color::GRAY;
            ui_grid.view.color_mod = ui::color::GRAY;
        }

        map_grid.clear();
        render::draw_map(world, map_grid);
        render::draw_renderables(world, map_grid);

        ui_grid.clear();
        ui::draw_ui(world, ui_grid, None);
    }
}
