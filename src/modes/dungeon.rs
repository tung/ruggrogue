use shipyard::{UniqueView, UniqueViewMut, World};

use crate::{
    damage, item, map,
    message::Messages,
    monster,
    player::{self, PlayerAlive, PlayerId, PlayerInputResult},
    render, spawn, ui, vision,
};
use ruggle::{CharGrid, InputBuffer};

use super::{
    inventory::{InventoryMode, InventoryModeResult},
    pick_up_menu::{PickUpMenuMode, PickUpMenuModeResult},
    yes_no_dialog::{YesNoDialogMode, YesNoDialogModeResult},
    ModeControl, ModeResult, ModeUpdate,
};

pub enum DungeonModeResult {
    Done,
}

pub struct DungeonMode;

/// The main gameplay mode.  The player can move around and explore the map, fight monsters and
/// perform other actions while alive, directly or indirectly.
impl DungeonMode {
    pub fn new(world: &World) -> Self {
        world.run(|mut msgs: UniqueViewMut<Messages>| msgs.add("Welcome to Ruggle!".into()));
        world.run(map::generate_rooms_and_corridors);
        world.run(|mut alive: UniqueViewMut<PlayerAlive>| alive.0 = true);
        world.run(map::place_player_in_first_room);
        spawn::fill_rooms_with_spawns(world);
        world.run(vision::recalculate_fields_of_view);

        Self {}
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
                    // "Really exit Ruggle?" prompt result.
                    ModeResult::YesNoDialogModeResult(result) => match result {
                        YesNoDialogModeResult::Yes => {
                            return (
                                ModeControl::Pop(DungeonModeResult::Done.into()),
                                ModeUpdate::Immediate,
                            )
                        }
                        YesNoDialogModeResult::No => false,
                    },

                    ModeResult::PickUpMenuModeResult(result) => match result {
                        PickUpMenuModeResult::PickedItem(item_id) => {
                            player::player_pick_up_item(world, *item_id);
                            true
                        }
                        PickUpMenuModeResult::Cancelled => false,
                    },

                    ModeResult::InventoryModeResult(result) => match result {
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

                    _ => false,
                }
            } else {
                match player::player_input(world, inputs) {
                    PlayerInputResult::NoResult => false,
                    PlayerInputResult::TurnDone => true,
                    PlayerInputResult::ShowExitPrompt => {
                        inputs.clear_input();
                        return (
                            ModeControl::Push(
                                YesNoDialogMode::new("Really exit Ruggle?".to_string(), false)
                                    .into(),
                            ),
                            ModeUpdate::Immediate,
                        );
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

    pub fn draw(&self, world: &World, grid: &mut CharGrid, active: bool) {
        render::draw_map(world, grid, active);
        render::draw_renderables(world, grid, active);
        ui::draw_ui(world, grid, active, None);
    }
}
