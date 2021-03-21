use shipyard::{UniqueView, UniqueViewMut, World};

use crate::{
    gamekey::{self, GameKey},
    ui::{self, Options},
};
use ruggle::{util::Size, Font, InputBuffer, InputEvent, KeyMods, TileGrid};

use super::{
    yes_no_dialog::{YesNoDialogMode, YesNoDialogModeResult},
    ModeControl, ModeResult, ModeUpdate,
};

const MAP_ZOOM_LABEL: &str = " Map zoom:";
const TEXT_ZOOM_LABEL: &str = "Text zoom:";
const ZOOM_1X_ON: &str = "[1x]";
const ZOOM_1X_OFF: &str = " 1x ";
const ZOOM_2X_ON: &str = "[2x]";
const ZOOM_2X_OFF: &str = " 2x ";
const QUIT: &str = "[ Quit ]";

pub enum OptionsMenuModeResult {
    AppQuit,
    Closed,
    ReallyQuit,
}

enum Selection {
    MapZoom,
    TextZoom,
    Quit,
}

pub struct OptionsMenuMode {
    selection: Selection,
}

/// A menu of general game options that the player can choose amongst.
impl OptionsMenuMode {
    pub fn new() -> Self {
        Self {
            selection: Selection::MapZoom,
        }
    }

    pub fn prepare_grids(
        &self,
        world: &World,
        grids: &mut Vec<TileGrid>,
        fonts: &[Font],
        window_size: Size,
    ) {
        let text_zoom = world.borrow::<UniqueView<Options>>().text_zoom;
        let new_grid_size = Size {
            w: 4 + (2 + MAP_ZOOM_LABEL.len() + ZOOM_1X_ON.len() + ZOOM_2X_ON.len())
                .max(2 + TEXT_ZOOM_LABEL.len() + ZOOM_1X_ON.len() + ZOOM_2X_ON.len())
                .max(QUIT.len()) as u32,
            h: 8,
        };

        if !grids.is_empty() {
            grids[0].resize(new_grid_size);
        } else {
            grids.push(TileGrid::new(new_grid_size, fonts, 0));
            grids[0].view.clear_color = None;
        }

        grids[0].view_centered(fonts, text_zoom, (0, 0).into(), window_size);
        grids[0].view.zoom = text_zoom;
    }

    pub fn update(
        &mut self,
        world: &World,
        inputs: &mut InputBuffer,
        pop_result: &Option<ModeResult>,
    ) -> (ModeControl, ModeUpdate) {
        if let Some(result) = pop_result {
            return match result {
                ModeResult::YesNoDialogModeResult(result) => match result {
                    YesNoDialogModeResult::AppQuit => (
                        ModeControl::Pop(OptionsMenuModeResult::AppQuit.into()),
                        ModeUpdate::Immediate,
                    ),
                    YesNoDialogModeResult::Yes => (
                        ModeControl::Pop(OptionsMenuModeResult::ReallyQuit.into()),
                        ModeUpdate::Immediate,
                    ),
                    YesNoDialogModeResult::No => (ModeControl::Stay, ModeUpdate::WaitForEvent),
                },
                _ => unreachable!(),
            };
        }

        inputs.prepare_input();

        if let Some(InputEvent::AppQuit) = inputs.get_input() {
            return (
                ModeControl::Pop(OptionsMenuModeResult::AppQuit.into()),
                ModeUpdate::Immediate,
            );
        } else if let Some(InputEvent::Press(keycode)) = inputs.get_input() {
            let mut options = world.borrow::<UniqueViewMut<Options>>();
            let gkey = gamekey::from_keycode(keycode, inputs.get_mods(KeyMods::SHIFT));

            match (&self.selection, gkey) {
                (Selection::MapZoom, GameKey::Up) => self.selection = Selection::Quit,
                (Selection::MapZoom, GameKey::Down) => self.selection = Selection::TextZoom,
                (Selection::MapZoom, GameKey::Left) => {
                    options.map_zoom = 1;
                    inputs.clear_input();
                    return (ModeControl::Stay, ModeUpdate::Immediate);
                }
                (Selection::MapZoom, GameKey::Right) => {
                    options.map_zoom = 2;
                    inputs.clear_input();
                    return (ModeControl::Stay, ModeUpdate::Immediate);
                }

                (Selection::TextZoom, GameKey::Up) => self.selection = Selection::MapZoom,
                (Selection::TextZoom, GameKey::Down) => self.selection = Selection::Quit,
                (Selection::TextZoom, GameKey::Left) => {
                    options.text_zoom = 1;
                    inputs.clear_input();
                    return (ModeControl::Stay, ModeUpdate::Immediate);
                }
                (Selection::TextZoom, GameKey::Right) => {
                    options.text_zoom = 2;
                    inputs.clear_input();
                    return (ModeControl::Stay, ModeUpdate::Immediate);
                }

                (Selection::Quit, GameKey::Up) => self.selection = Selection::TextZoom,
                (Selection::Quit, GameKey::Down) => self.selection = Selection::MapZoom,
                (Selection::Quit, GameKey::Confirm) => {
                    inputs.clear_input();
                    return (
                        ModeControl::Push(
                            YesNoDialogMode::new("Really exit Ruggle?".to_string(), false).into(),
                        ),
                        ModeUpdate::Immediate,
                    );
                }

                (_, GameKey::Cancel) => {
                    return (
                        ModeControl::Pop(OptionsMenuModeResult::Closed.into()),
                        ModeUpdate::Immediate,
                    )
                }
                (_, _) => {}
            }
        }

        (ModeControl::Stay, ModeUpdate::WaitForEvent)
    }

    pub fn draw(&self, world: &World, grids: &mut [TileGrid], active: bool) {
        let grid = &mut grids[0];
        let options = world.borrow::<UniqueView<Options>>();

        grid.view.color_mod = if active {
            ui::color::WHITE
        } else {
            ui::color::GRAY
        };

        grid.set_draw_fg(ui::color::WHITE);
        grid.set_draw_bg(ui::color::BLACK);
        grid.draw_box((0, 0), (grid.width(), grid.height()));
        grid.print((2, 0), "< Options >");

        let map_zoom_1x_x = 3 + MAP_ZOOM_LABEL.len() as i32;
        let map_zoom_2x_x = 4 + (MAP_ZOOM_LABEL.len() + ZOOM_1X_OFF.len()) as i32;
        let map_zoom_y = 2;

        grid.print((2, map_zoom_y), MAP_ZOOM_LABEL);
        grid.set_draw_bg(ui::color::SELECTED_BG);
        grid.print_color(
            (map_zoom_1x_x, map_zoom_y),
            false,
            options.map_zoom == 1 && matches!(self.selection, Selection::MapZoom),
            if options.map_zoom == 1 {
                ZOOM_1X_ON
            } else {
                ZOOM_1X_OFF
            },
        );
        grid.print_color(
            (map_zoom_2x_x, map_zoom_y),
            false,
            options.map_zoom == 2 && matches!(self.selection, Selection::MapZoom),
            if options.map_zoom == 2 {
                ZOOM_2X_ON
            } else {
                ZOOM_2X_OFF
            },
        );

        let text_zoom_1x_x = 3 + TEXT_ZOOM_LABEL.len() as i32;
        let text_zoom_2x_x = 4 + (TEXT_ZOOM_LABEL.len() + ZOOM_1X_OFF.len()) as i32;
        let text_zoom_y = 3;

        grid.print((2, text_zoom_y), TEXT_ZOOM_LABEL);
        grid.set_draw_bg(ui::color::SELECTED_BG);
        grid.print_color(
            (text_zoom_1x_x, text_zoom_y),
            false,
            options.text_zoom == 1 && matches!(self.selection, Selection::TextZoom),
            if options.text_zoom == 1 {
                ZOOM_1X_ON
            } else {
                ZOOM_1X_OFF
            },
        );
        grid.print_color(
            (text_zoom_2x_x, text_zoom_y),
            false,
            options.text_zoom == 2 && matches!(self.selection, Selection::TextZoom),
            if options.text_zoom == 2 {
                ZOOM_2X_ON
            } else {
                ZOOM_2X_OFF
            },
        );

        grid.set_draw_bg(ui::color::SELECTED_BG);
        grid.print_color(
            (2, 5),
            false,
            matches!(self.selection, Selection::Quit),
            QUIT,
        );
    }
}
