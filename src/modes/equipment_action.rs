use shipyard::{EntityId, Get, UniqueView, View, World};

use crate::{
    components::{Name, Renderable},
    gamekey::{self, GameKey},
    gamesym::GameSym,
    ui::{self, Options},
};
use ruggle::{
    util::{Color, Size},
    InputBuffer, InputEvent, KeyMods, TileGrid, Tileset,
};

use super::{ModeControl, ModeResult, ModeUpdate};

const REMOVE: &str = "[ Remove ]";
const DROP: &str = "[ Drop ]";
const CANCEL: &str = "[ Cancel ]";

pub enum EquipmentActionModeResult {
    AppQuit,
    Cancelled,
    RemoveEquipment(EntityId),
    DropEquipment(EntityId),
}

enum Selection {
    RemoveEquipment,
    DropEquipment,
    Cancel,
}

pub struct EquipmentActionMode {
    item_id: EntityId,
    inner_width: i32,
    selection: Selection,
}

/// Show a menu of actions for an item currently equipped by the player.
impl EquipmentActionMode {
    pub fn new(world: &World, item_id: EntityId) -> Self {
        let item_width = world.borrow::<View<Name>>().get(item_id).0.len();

        Self {
            item_id,
            inner_width: 2 + item_width
                .max(REMOVE.len())
                .max(DROP.len())
                .max(CANCEL.len()) as i32,
            selection: Selection::RemoveEquipment,
        }
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
        let new_grid_size = Size {
            w: 4 + self.inner_width as u32,
            h: 10,
        };

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
        _world: &World,
        inputs: &mut InputBuffer,
        _pop_result: &Option<ModeResult>,
    ) -> (ModeControl, ModeUpdate) {
        inputs.prepare_input();

        if let Some(InputEvent::AppQuit) = inputs.get_input() {
            return (
                ModeControl::Pop(EquipmentActionModeResult::AppQuit.into()),
                ModeUpdate::Immediate,
            );
        } else if let Some(InputEvent::Press(keycode)) = inputs.get_input() {
            let shift = inputs.get_mods(KeyMods::SHIFT);

            match (&self.selection, gamekey::from_keycode(keycode, shift)) {
                (Selection::RemoveEquipment, GameKey::Up) => {
                    self.selection = Selection::Cancel;
                }
                (Selection::RemoveEquipment, GameKey::Down) => {
                    self.selection = Selection::DropEquipment;
                }
                (Selection::RemoveEquipment, GameKey::Confirm) => {
                    return (
                        ModeControl::Pop(
                            EquipmentActionModeResult::RemoveEquipment(self.item_id).into(),
                        ),
                        ModeUpdate::Immediate,
                    )
                }

                (Selection::DropEquipment, GameKey::Up) => {
                    self.selection = Selection::RemoveEquipment;
                }
                (Selection::DropEquipment, GameKey::Down) => {
                    self.selection = Selection::Cancel;
                }
                (Selection::DropEquipment, GameKey::Confirm) => {
                    return (
                        ModeControl::Pop(
                            EquipmentActionModeResult::DropEquipment(self.item_id).into(),
                        ),
                        ModeUpdate::Immediate,
                    )
                }

                (Selection::Cancel, GameKey::Up) => {
                    self.selection = Selection::DropEquipment;
                }
                (Selection::Cancel, GameKey::Down) => {
                    self.selection = Selection::RemoveEquipment;
                }
                (Selection::Cancel, GameKey::Confirm) | (_, GameKey::Cancel) => {
                    return (
                        ModeControl::Pop(EquipmentActionModeResult::Cancelled.into()),
                        ModeUpdate::Immediate,
                    )
                }

                _ => {}
            }
        }

        (ModeControl::Stay, ModeUpdate::WaitForEvent)
    }

    pub fn draw(&self, world: &World, grids: &mut [TileGrid<GameSym>], active: bool) {
        let grid = &mut grids[0];
        let fg = Color::WHITE;
        let bg = Color::BLACK;
        let selected_bg = ui::SELECTED_BG;

        grid.view.color_mod = if active { Color::WHITE } else { Color::GRAY };

        grid.draw_box((0, 0), (grid.width(), grid.height()), fg, bg);

        {
            let names = world.borrow::<View<Name>>();
            let renderables = world.borrow::<View<Renderable>>();
            let render = renderables.get(self.item_id);

            grid.put_sym_color((2, 2), render.sym, render.fg, render.bg);
            grid.print_color((4, 2), &names.get(self.item_id).0, true, fg, bg);
        }

        grid.print_color(
            (4, 4),
            REMOVE,
            true,
            fg,
            if matches!(self.selection, Selection::RemoveEquipment) {
                selected_bg
            } else {
                bg
            },
        );
        grid.print_color(
            (4, 5),
            DROP,
            true,
            fg,
            if matches!(self.selection, Selection::DropEquipment) {
                selected_bg
            } else {
                bg
            },
        );

        grid.print_color(
            (4, 7),
            CANCEL,
            true,
            fg,
            if matches!(self.selection, Selection::Cancel) {
                selected_bg
            } else {
                bg
            },
        );
    }
}
