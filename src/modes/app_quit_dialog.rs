use shipyard::World;

use crate::gamesym::GameSym;
use ruggrogue::{util::Size, InputBuffer, TileGrid, Tileset};

use super::{
    yes_no_dialog::{YesNoDialogMode, YesNoDialogModeResult},
    ModeControl, ModeResult, ModeUpdate,
};

pub enum AppQuitDialogModeResult {
    Cancelled,
    Confirmed,
}

pub struct AppQuitDialogMode(YesNoDialogMode);

/// A yes-or-no dialog box that appears when the use requests that the app be closed.
impl AppQuitDialogMode {
    pub fn new() -> Self {
        Self(YesNoDialogMode::new(
            "Really quit RuggRogue?".to_string(),
            false,
        ))
    }

    pub fn prepare_grids(
        &self,
        world: &World,
        grids: &mut Vec<TileGrid<GameSym>>,
        tilesets: &[Tileset<GameSym>],
        window_size: Size,
    ) {
        self.0.prepare_grids(world, grids, tilesets, window_size);
    }

    pub fn update(
        &mut self,
        world: &World,
        inputs: &mut InputBuffer,
        grids: &[TileGrid<GameSym>],
        pop_result: &Option<ModeResult>,
    ) -> (ModeControl, ModeUpdate) {
        match self.0.update(world, inputs, grids, pop_result) {
            (ModeControl::Pop(ModeResult::YesNoDialogModeResult(result)), mode_update) => {
                match result {
                    YesNoDialogModeResult::AppQuit => (ModeControl::Stay, ModeUpdate::WaitForEvent),
                    YesNoDialogModeResult::Yes => (
                        ModeControl::Pop(AppQuitDialogModeResult::Confirmed.into()),
                        mode_update,
                    ),
                    YesNoDialogModeResult::No => (
                        ModeControl::Pop(AppQuitDialogModeResult::Cancelled.into()),
                        mode_update,
                    ),
                }
            }
            result => result,
        }
    }

    pub fn draw(&self, world: &World, grids: &mut [TileGrid<GameSym>], active: bool) {
        self.0.draw(world, grids, active);
    }
}
