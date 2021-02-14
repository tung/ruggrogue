use shipyard::{Get, IntoIter, UniqueView, UniqueViewMut, View, World};

use crate::{
    damage, item, map,
    message::Messages,
    monster,
    player::{self, PlayerAlive, PlayerId, PlayerInputResult},
    spawn, ui, vision,
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

fn player_is_alive(player_alive: UniqueView<PlayerAlive>) -> bool {
    player_alive.0
}

fn draw_renderables(world: &World, grid: &mut CharGrid, active: bool) {
    use crate::components::{FieldOfView, Position, RenderOnFloor, RenderOnMap, Renderable};

    world.run(
        |player_id: UniqueView<PlayerId>,
         fovs: View<FieldOfView>,
         positions: View<Position>,
         render_on_floors: View<RenderOnFloor>,
         render_on_maps: View<RenderOnMap>,
         renderables: View<Renderable>| {
            let (x, y) = positions.get(player_id.0).into();
            let fov = fovs.get(player_id.0);
            let w = grid.size_cells()[0];
            let h = grid.size_cells()[1] - ui::HUD_LINES;
            let cx = w / 2;
            let cy = h / 2;
            let mut render_entity = |pos: &Position, render: &Renderable| {
                let gx = pos.x - x + cx;
                let gy = pos.y - y + cy;
                if gx >= 0 && gy >= 0 && gx < w && gy < h && fov.get(pos.into()) {
                    grid.put_color(
                        [gx, gy],
                        Some(ui::recolor(render.fg, active)),
                        Some(ui::recolor(render.bg, active)),
                        render.ch,
                    );
                }
            };

            // Draw floor entities first.
            for (pos, render, _) in (&positions, &renderables, &render_on_floors).iter() {
                render_entity(pos, render);
            }

            // Draw normal map entities.
            for (pos, render, _) in (&positions, &renderables, &render_on_maps).iter() {
                render_entity(pos, render);
            }
        },
    );
}

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
        if world.run(player_is_alive) {
            let (time_passed, player_turn_done) = if let Some(result) = pop_result {
                match result {
                    // "Really exit Ruggle?" prompt result.
                    ModeResult::YesNoDialogModeResult(result) => match result {
                        YesNoDialogModeResult::Yes => {
                            return (
                                ModeControl::Pop(DungeonModeResult::Done.into()),
                                ModeUpdate::Immediate,
                            )
                        }
                        YesNoDialogModeResult::No => (false, false),
                    },

                    ModeResult::PickUpMenuModeResult(result) => match result {
                        PickUpMenuModeResult::PickedItem(item_id) => {
                            player::player_pick_up_item(world, *item_id);
                            (true, true)
                        }
                        PickUpMenuModeResult::Cancelled => (false, false),
                    },

                    ModeResult::InventoryModeResult(result) => match result {
                        InventoryModeResult::DoNothing => (false, false),
                        InventoryModeResult::UseItem(item_id) => {
                            let player_id =
                                world.run(|player_id: UniqueView<PlayerId>| player_id.0);
                            item::use_item(world, player_id, *item_id);
                            (true, true)
                        }
                        InventoryModeResult::DropItem(item_id) => {
                            player::player_drop_item(world, *item_id);
                            (true, true)
                        }
                    },

                    _ => (false, false),
                }
            } else if !world.run(monster::monster_turns_empty) {
                world.run(monster::do_monster_turns);
                (true, false)
            } else {
                match player::player_input(world, inputs) {
                    PlayerInputResult::NoResult => (false, false),
                    PlayerInputResult::TurnDone => (true, true),
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
                world.run(damage::melee_combat);
                world.run(damage::inflict_damage);
                world.run(damage::delete_dead_entities);
                world.run(vision::recalculate_fields_of_view);
                if player_turn_done {
                    world.run(monster::enqueue_monster_turns);
                }
            }

            let update = world.run(player_is_alive)
                && (!world.run(monster::monster_turns_empty)
                    || world.run(player::player_is_auto_running));

            (
                ModeControl::Stay,
                if update {
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
        map::draw_map(world, grid, active);
        draw_renderables(world, grid, active);
        ui::draw_ui(world, grid, active);
    }
}
