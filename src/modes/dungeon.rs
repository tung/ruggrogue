use shipyard::{Get, IntoIter, UniqueView, UniqueViewMut, View, World};

use crate::{
    components::PlayerId, damage, map, message, monster, player, spawn, ui, vision, PlayerAlive,
};

use super::{ModeControl, ModeResult, ModeUpdate};

pub enum DungeonModeResult {
    Done,
}

pub struct DungeonMode;

fn player_is_alive(player_alive: UniqueView<PlayerAlive>) -> bool {
    player_alive.0
}

fn draw_renderables(world: &World, grid: &mut ruggle::CharGrid) {
    use crate::components::{FieldOfView, Position, Renderable};

    world.run(
        |player: UniqueView<PlayerId>,
         fovs: View<FieldOfView>,
         positions: View<Position>,
         renderables: View<Renderable>| {
            let (x, y) = positions.get(player.0).into();
            let fov = fovs.get(player.0);
            let w = grid.size_cells()[0];
            let h = grid.size_cells()[1] - ui::HUD_LINES;
            let cx = w / 2;
            let cy = h / 2;

            for (pos, render) in (&positions, &renderables).iter() {
                let gx = pos.x - x + cx;
                let gy = pos.y - y + cy;

                if gx >= 0 && gy >= 0 && gx < w && gy < h && fov.get(pos.into()) {
                    grid.put_color([gx, gy], Some(render.fg), Some(render.bg), render.ch);
                }
            }
        },
    );
}

/// The main gameplay mode.  The player can move around and explore the map, fight monsters and
/// perform other actions while alive, directly or indirectly.
impl DungeonMode {
    pub fn new(world: &World) -> Self {
        world.run(|mut msgs: UniqueViewMut<message::Messages>| {
            msgs.add("Welcome to Ruggle!".into())
        });
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
        inputs: &mut ruggle::InputBuffer,
        _pop_result: &Option<ModeResult>,
    ) -> (ModeControl, ModeUpdate) {
        if world.run(player_is_alive) {
            let (time_passed, player_turn_done) = if !world.run(monster::monster_turns_empty) {
                world.run(monster::do_monster_turns);
                (true, false)
            } else if player::player_input(world, inputs) {
                (true, true)
            } else {
                (false, false)
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

            if world.run(player_is_alive) && !world.run(monster::monster_turns_empty) {
                (ModeControl::Stay, ModeUpdate::Update)
            } else {
                (ModeControl::Stay, ModeUpdate::WaitForEvent)
            }
        } else if player::player_is_dead_input(inputs) {
            (
                ModeControl::Pop(DungeonModeResult::Done.into()),
                ModeUpdate::Update,
            )
        } else {
            (ModeControl::Stay, ModeUpdate::WaitForEvent)
        }
    }

    pub fn draw(&self, world: &World, grid: &mut ruggle::CharGrid) {
        map::draw_map(world, grid);
        draw_renderables(world, grid);
        ui::draw_ui(world, grid);
    }
}
