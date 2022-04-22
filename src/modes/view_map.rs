use shipyard::{Get, UniqueView, UniqueViewMut, View, World};

use crate::{
    chunked::{Camera, ChunkedMapGrid},
    components::Coord,
    gamekey::{self, GameKey},
    gamesym::GameSym,
    map::Map,
    player::PlayerId,
    render, ui,
};
use ruggrogue::{
    util::{Color, Position, Size},
    InputBuffer, InputEvent, KeyMods, TileGrid, Tileset,
};

use super::{ModeControl, ModeResult, ModeUpdate};

const SHIFT_STEP: i32 = 5;

pub enum ViewMapModeResult {
    AppQuit,
    Done,
}

pub struct ViewMapMode {
    chunked_map_grid: ChunkedMapGrid,
    old_msg_frame_size: Size,
    redraw_msg_frame_grid: bool,
    center: Position,
    range: i32,
}

fn reset_camera(
    mut camera: UniqueViewMut<Camera>,
    player_id: UniqueView<PlayerId>,
    coords: View<Coord>,
) {
    camera.0 = coords.get(player_id.0).0;
}

/// Show a movable cursor that describes seen and recalled map tiles and any occupying entities.
impl ViewMapMode {
    pub fn new(world: &World) -> Self {
        Self {
            chunked_map_grid: ChunkedMapGrid::new(),
            old_msg_frame_size: (0, 0).into(),
            redraw_msg_frame_grid: true,
            center: world.borrow::<UniqueView<Camera>>().0,
            range: 80,
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
        _pop_result: &Option<ModeResult>,
    ) -> (ModeControl, ModeUpdate) {
        inputs.prepare_input();

        if let Some(InputEvent::AppQuit) = inputs.get_input() {
            world.run(reset_camera);

            return (
                ModeControl::Pop(ViewMapModeResult::AppQuit.into()),
                ModeUpdate::Immediate,
            );
        } else if let Some(InputEvent::Press(keycode)) = inputs.get_input() {
            let shift = inputs.get_mods(KeyMods::SHIFT);
            let move_amount = if shift { SHIFT_STEP } else { 1 };
            let mut move_x = 0;
            let mut move_y = 0;

            match gamekey::from_keycode(keycode, shift) {
                GameKey::Up => move_y = -move_amount,
                GameKey::Down => move_y = move_amount,
                GameKey::Left => move_x = -move_amount,
                GameKey::Right => move_x = move_amount,
                GameKey::UpLeft => {
                    move_x = -move_amount;
                    move_y = -move_amount;
                }
                GameKey::UpRight => {
                    move_x = move_amount;
                    move_y = -move_amount;
                }
                GameKey::DownLeft => {
                    move_x = -move_amount;
                    move_y = move_amount;
                }
                GameKey::DownRight => {
                    move_x = move_amount;
                    move_y = move_amount;
                }
                GameKey::Home => {
                    let player_pos = {
                        let player_id = world.borrow::<UniqueView<PlayerId>>();
                        let coords = world.borrow::<View<Coord>>();
                        coords.get(player_id.0).0
                    };
                    let camera = world.borrow::<UniqueView<Camera>>();

                    move_x = player_pos.x - camera.0.x;
                    move_y = player_pos.y - camera.0.y;
                }
                GameKey::Confirm | GameKey::Cancel | GameKey::ViewMap => {
                    world.run(reset_camera);
                    return (
                        ModeControl::Pop(ViewMapModeResult::Done.into()),
                        ModeUpdate::Immediate,
                    );
                }
                _ => {}
            }

            if move_x != 0 || move_y != 0 {
                let min_x = self.center.x - self.range;
                let max_x = self.center.x + self.range;
                let min_y = self.center.y - self.range;
                let max_y = self.center.y + self.range;
                let mut camera = world.borrow::<UniqueViewMut<Camera>>();

                // Keep the camera within range of the center.
                if move_x < 0 && camera.0.x + move_x < min_x {
                    move_x = min_x - camera.0.x;
                    move_y = move_y.signum() * move_y.abs().min(move_x.abs());
                }
                if move_x > 0 && camera.0.x + move_x > max_x {
                    move_x = max_x - camera.0.x;
                    move_y = move_y.signum() * move_y.abs().min(move_x.abs());
                }
                if move_y < 0 && camera.0.y + move_y < min_y {
                    move_y = min_y - camera.0.y;
                    move_x = move_x.signum() * move_x.abs().min(move_y.abs());
                }
                if move_y > 0 && camera.0.y + move_y > max_y {
                    move_y = max_y - camera.0.y;
                    move_x = move_x.signum() * move_x.abs().min(move_y.abs());
                }

                if move_x != 0 || move_y != 0 {
                    let old_camera_pos = camera.0;

                    camera.0.x += move_x;
                    camera.0.y += move_y;

                    self.chunked_map_grid
                        .mark_dirty(old_camera_pos, Size { w: 1, h: 1 });
                    self.chunked_map_grid
                        .mark_dirty(camera.0, Size { w: 1, h: 1 });
                }
            }
        }

        (ModeControl::Stay, ModeUpdate::WaitForEvent)
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

        let camera = world.borrow::<UniqueView<Camera>>();
        let map = world.borrow::<UniqueView<Map>>();
        let player_pos = {
            let player_id = world.borrow::<UniqueView<PlayerId>>();
            let coords = world.borrow::<View<Coord>>();
            coords.get(player_id.0).0
        };

        // Highlight cursor position.
        if let Some(pos) = self.chunked_map_grid.map_to_grid_pos(world, camera.0) {
            map_grid.recolor_pos(pos, None, Color::MAGENTA);
        }

        // Describe the location that the camera is positioned at.
        let (desc, recalled) = map.describe_pos(world, camera.0.x, camera.0.y, false, false, false);

        if self.redraw_msg_frame_grid {
            ui::draw_msg_frame(msg_frame_grid, true);
        }

        msg_grid.clear();
        ui::draw_ui(
            world,
            status_grid,
            item_grid,
            msg_grid,
            Some(&format!(
                "You {} [{:+},{:+}]: {}",
                if recalled { "recall" } else { "see" },
                camera.0.x - player_pos.x,
                camera.0.y - player_pos.y,
                desc,
            )),
        );
    }
}
