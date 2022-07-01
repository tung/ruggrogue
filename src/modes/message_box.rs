use shipyard::{UniqueView, World};

use crate::{
    gamekey::{self, GameKey},
    gamesym::GameSym,
    ui::Options,
};
use ruggrogue::{
    util::{Color, Size},
    InputBuffer, InputEvent, KeyMods, TileGrid, Tileset,
};

use super::{ModeControl, ModeResult, ModeUpdate};

pub enum MessageBoxModeResult {
    AppQuit,
    Done,
}

pub struct MessageBoxMode {
    msg: Vec<String>,
    inner_width: u32,
}

/// Show a multi-line message box.
impl MessageBoxMode {
    pub fn new(msg: Vec<String>) -> Self {
        let inner_width = msg.iter().map(|m| m.chars().count()).max().unwrap_or(0) as u32;

        Self { msg, inner_width }
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
        let new_size = Size {
            w: self.inner_width + 4,
            h: self.msg.len() as u32 + 4,
        };

        if !grids.is_empty() {
            grids[0].resize(new_size);
        } else {
            grids.push(TileGrid::new(new_size, tilesets, font as usize));
            grids[0].view.clear_color = None;
        }

        grids[0].set_tileset(tilesets, font as usize);
        grids[0].view_centered(tilesets, text_zoom, (0, 0).into(), window_size);
        grids[0].view.zoom = text_zoom;
    }

    pub fn update(
        &mut self,
        _world: &World,
        inputs: &mut InputBuffer,
        _grids: &[TileGrid<GameSym>],
        _pop_result: &Option<ModeResult>,
    ) -> (ModeControl, ModeUpdate) {
        inputs.prepare_input();

        if let Some(InputEvent::AppQuit) = inputs.get_input() {
            return (
                ModeControl::Pop(MessageBoxModeResult::AppQuit.into()),
                ModeUpdate::Immediate,
            );
        } else if let Some(InputEvent::Press(keycode)) = inputs.get_input() {
            let key = gamekey::from_keycode(keycode, inputs.get_mods(KeyMods::SHIFT));
            if matches!(key, GameKey::Confirm | GameKey::Cancel) {
                inputs.clear_input();
                return (
                    ModeControl::Pop(MessageBoxModeResult::Done.into()),
                    ModeUpdate::Immediate,
                );
            }
        }

        (ModeControl::Stay, ModeUpdate::WaitForEvent)
    }

    pub fn draw(&self, _world: &World, grids: &mut [TileGrid<GameSym>], active: bool) {
        let grid = &mut grids[0];

        grid.view.color_mod = if active { Color::WHITE } else { Color::GRAY };

        grid.draw_box(
            (0, 0),
            (grid.width(), grid.height()),
            Color::WHITE,
            Color::BLACK,
        );

        for (y, msg) in self.msg.iter().enumerate() {
            grid.print((2, 2 + y as i32), msg);
        }
    }
}
