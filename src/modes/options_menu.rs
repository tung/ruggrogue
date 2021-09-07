use shipyard::{UniqueView, UniqueViewMut, World};

use crate::{
    gamekey::{self, GameKey},
    gamesym::GameSym,
    ui::{self, Options},
};
use ruggrogue::{
    util::{Color, Size},
    InputBuffer, InputEvent, KeyMods, TileGrid, Tileset,
};

use super::{
    yes_no_dialog::{YesNoDialogMode, YesNoDialogModeResult},
    ModeControl, ModeResult, ModeUpdate,
};

const TILESET_LABEL: &str = "  Tileset:";
const FONT_LABEL: &str = "     Font:";
const NUM_FONTS: u32 = 2;
const TILESET_NAMES: [&str; 3] = ["GohuFont", "Terminal", "Urizen"];
const UNKNOWN_TILESET_NAME: &str = "???";
const MAP_ZOOM_LABEL: &str = " Map zoom:";
const TEXT_ZOOM_LABEL: &str = "Text zoom:";
const ZOOM_1X_ON: &str = "[1x]";
const ZOOM_1X_OFF: &str = " 1x ";
const ZOOM_2X_ON: &str = "[2x]";
const ZOOM_2X_OFF: &str = " 2x ";
const QUIT: &str = "[ Save and exit ]";
const BACK: &str = "[ Back ]";

pub enum OptionsMenuModeResult {
    AppQuit,
    Closed,
    ReallyQuit,
}

enum Selection {
    Tileset,
    Font,
    MapZoom,
    TextZoom,
    Quit,
}

pub struct OptionsMenuMode {
    prompt_to_save: bool,
    selection: Selection,
}

/// A menu of general game options that the player can choose amongst.
impl OptionsMenuMode {
    pub fn new(prompt_to_save: bool) -> Self {
        Self {
            prompt_to_save,
            selection: Selection::Tileset,
        }
    }

    pub fn prepare_grids(
        &self,
        world: &World,
        grids: &mut Vec<TileGrid<GameSym>>,
        tilesets: &[Tileset<GameSym>],
        window_size: Size,
    ) {
        let tileset_width = 7
            + TILESET_LABEL.len()
            + TILESET_NAMES
                .iter()
                .map(|n| n.len())
                .max()
                .unwrap_or_else(|| UNKNOWN_TILESET_NAME.len());
        let font_width = 7
            + FONT_LABEL.len()
            + TILESET_NAMES
                .iter()
                .take(NUM_FONTS as usize)
                .map(|n| n.len())
                .max()
                .unwrap_or_else(|| UNKNOWN_TILESET_NAME.len());
        let map_zoom_width = 2 + MAP_ZOOM_LABEL.len() + ZOOM_1X_ON.len() + ZOOM_2X_ON.len();
        let text_zoom_width = 2 + TEXT_ZOOM_LABEL.len() + ZOOM_1X_ON.len() + ZOOM_2X_ON.len();
        let new_grid_size = Size {
            w: 4 + tileset_width
                .max(font_width)
                .max(map_zoom_width)
                .max(text_zoom_width)
                .max(QUIT.len()) as u32,
            h: 10,
        };
        let Options {
            font, text_zoom, ..
        } = *world.borrow::<UniqueView<Options>>();

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
                (Selection::Tileset, GameKey::Up) => self.selection = Selection::Quit,
                (Selection::Tileset, GameKey::Down) => self.selection = Selection::Font,
                (Selection::Tileset, GameKey::Left) => {
                    if options.tileset > 0 {
                        options.tileset -= 1;
                        inputs.clear_input();
                        return (ModeControl::Stay, ModeUpdate::Immediate);
                    }
                }
                (Selection::Tileset, GameKey::Right) => {
                    if options.tileset as usize + 1 < TILESET_NAMES.len() {
                        options.tileset += 1;
                        inputs.clear_input();
                        return (ModeControl::Stay, ModeUpdate::Immediate);
                    }
                }

                (Selection::Font, GameKey::Up) => self.selection = Selection::Tileset,
                (Selection::Font, GameKey::Down) => self.selection = Selection::MapZoom,
                (Selection::Font, GameKey::Left) => {
                    if options.font > 0 {
                        options.font -= 1;
                        inputs.clear_input();
                        return (ModeControl::Stay, ModeUpdate::Immediate);
                    }
                }
                (Selection::Font, GameKey::Right) => {
                    if options.font + 1 < NUM_FONTS {
                        options.font += 1;
                        inputs.clear_input();
                        return (ModeControl::Stay, ModeUpdate::Immediate);
                    }
                }

                (Selection::MapZoom, GameKey::Up) => self.selection = Selection::Font,
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
                (Selection::Quit, GameKey::Down) => self.selection = Selection::Tileset,
                (Selection::Quit, GameKey::Confirm) => {
                    inputs.clear_input();
                    return (
                        if self.prompt_to_save {
                            ModeControl::Push(
                                YesNoDialogMode::new(
                                    "Save and return to title screen?".into(),
                                    false,
                                )
                                .into(),
                            )
                        } else {
                            ModeControl::Pop(OptionsMenuModeResult::Closed.into())
                        },
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

    fn draw_tileset(
        &self,
        world: &World,
        grid: &mut TileGrid<GameSym>,
        fg: Color,
        bg: Color,
        selected_bg: Color,
    ) {
        let tileset_left_x = 3 + TILESET_LABEL.len() as i32;
        let tileset_name_x = 3 + tileset_left_x;
        let tileset_right_x = 1
            + tileset_name_x
            + TILESET_NAMES
                .iter()
                .map(|n| n.len())
                .max()
                .unwrap_or_else(|| UNKNOWN_TILESET_NAME.len()) as i32;
        let tileset_y = 2;
        let tileset = world.borrow::<UniqueView<Options>>().tileset;

        grid.print((2, tileset_y), TILESET_LABEL);
        if tileset > 0 {
            grid.print_color((tileset_left_x, tileset_y), "<<", true, fg, bg);
        }
        grid.print_color(
            (tileset_name_x, tileset_y),
            TILESET_NAMES
                .get(tileset as usize)
                .unwrap_or(&UNKNOWN_TILESET_NAME),
            true,
            fg,
            if matches!(self.selection, Selection::Tileset) {
                selected_bg
            } else {
                bg
            },
        );
        if tileset as usize + 1 < TILESET_NAMES.len() {
            grid.print_color((tileset_right_x, tileset_y), ">>", true, fg, bg);
        }
    }

    fn draw_font(
        &self,
        world: &World,
        grid: &mut TileGrid<GameSym>,
        fg: Color,
        bg: Color,
        selected_bg: Color,
    ) {
        let font_left_x = 3 + FONT_LABEL.len() as i32;
        let font_name_x = 3 + font_left_x;
        let font_right_x = 1
            + font_name_x
            + TILESET_NAMES
                .iter()
                .map(|n| n.len())
                .max()
                .unwrap_or_else(|| UNKNOWN_TILESET_NAME.len()) as i32;
        let font_y = 3;
        let font = world.borrow::<UniqueView<Options>>().font;

        grid.print((2, font_y), FONT_LABEL);
        if font > 0 {
            grid.print_color((font_left_x, font_y), "<<", true, fg, bg);
        }
        grid.print_color(
            (font_name_x, font_y),
            TILESET_NAMES
                .get(font as usize)
                .unwrap_or(&UNKNOWN_TILESET_NAME),
            true,
            fg,
            if matches!(self.selection, Selection::Font) {
                selected_bg
            } else {
                bg
            },
        );
        if font + 1 < NUM_FONTS {
            grid.print_color((font_right_x, font_y), ">>", true, fg, bg);
        }
    }

    fn draw_map_zoom(
        &self,
        world: &World,
        grid: &mut TileGrid<GameSym>,
        fg: Color,
        bg: Color,
        selected_bg: Color,
    ) {
        let map_zoom_1x_x = 3 + MAP_ZOOM_LABEL.len() as i32;
        let map_zoom_2x_x = 4 + (MAP_ZOOM_LABEL.len() + ZOOM_1X_OFF.len()) as i32;
        let map_zoom_y = 4;
        let map_zoom = world.borrow::<UniqueView<Options>>().map_zoom;

        grid.print((2, map_zoom_y), MAP_ZOOM_LABEL);
        grid.print_color(
            (map_zoom_1x_x, map_zoom_y),
            if map_zoom == 1 {
                ZOOM_1X_ON
            } else {
                ZOOM_1X_OFF
            },
            true,
            fg,
            if map_zoom == 1 && matches!(self.selection, Selection::MapZoom) {
                selected_bg
            } else {
                bg
            },
        );
        grid.print_color(
            (map_zoom_2x_x, map_zoom_y),
            if map_zoom == 2 {
                ZOOM_2X_ON
            } else {
                ZOOM_2X_OFF
            },
            true,
            fg,
            if map_zoom == 2 && matches!(self.selection, Selection::MapZoom) {
                selected_bg
            } else {
                bg
            },
        );
    }

    fn draw_text_zoom(
        &self,
        world: &World,
        grid: &mut TileGrid<GameSym>,
        fg: Color,
        bg: Color,
        selected_bg: Color,
    ) {
        let text_zoom_1x_x = 3 + TEXT_ZOOM_LABEL.len() as i32;
        let text_zoom_2x_x = 4 + (TEXT_ZOOM_LABEL.len() + ZOOM_1X_OFF.len()) as i32;
        let text_zoom_y = 5;
        let text_zoom = world.borrow::<UniqueView<Options>>().text_zoom;

        grid.print((2, text_zoom_y), TEXT_ZOOM_LABEL);
        grid.print_color(
            (text_zoom_1x_x, text_zoom_y),
            if text_zoom == 1 {
                ZOOM_1X_ON
            } else {
                ZOOM_1X_OFF
            },
            true,
            fg,
            if text_zoom == 1 && matches!(self.selection, Selection::TextZoom) {
                selected_bg
            } else {
                bg
            },
        );
        grid.print_color(
            (text_zoom_2x_x, text_zoom_y),
            if text_zoom == 2 {
                ZOOM_2X_ON
            } else {
                ZOOM_2X_OFF
            },
            true,
            fg,
            if text_zoom == 2 && matches!(self.selection, Selection::TextZoom) {
                selected_bg
            } else {
                bg
            },
        );
    }

    pub fn draw(&self, world: &World, grids: &mut [TileGrid<GameSym>], active: bool) {
        let grid = &mut grids[0];
        let fg = Color::WHITE;
        let bg = Color::BLACK;
        let selected_bg = ui::SELECTED_BG;

        grid.view.color_mod = if active { Color::WHITE } else { Color::GRAY };

        grid.draw_box((0, 0), (grid.width(), grid.height()), fg, bg);
        grid.print_color((2, 0), "< Options >", true, Color::YELLOW, bg);

        self.draw_tileset(world, grid, fg, bg, selected_bg);
        self.draw_font(world, grid, fg, bg, selected_bg);
        self.draw_map_zoom(world, grid, fg, bg, selected_bg);
        self.draw_text_zoom(world, grid, fg, bg, selected_bg);

        grid.print_color(
            (2, 7),
            if self.prompt_to_save { QUIT } else { BACK },
            true,
            fg,
            if matches!(self.selection, Selection::Quit) {
                selected_bg
            } else {
                bg
            },
        );
    }
}
