use shipyard::{EntityId, Get, UniqueView, View, World};

use crate::{
    components::{AreaOfEffect, Name, Ranged, Renderable},
    gamekey::{self, GameKey},
    gamesym::GameSym,
    ui::{self, Options},
};
use ruggle::{util::Size, InputBuffer, InputEvent, KeyMods, TileGrid, Tileset};

use super::{
    target::{TargetMode, TargetModeResult},
    ModeControl, ModeResult, ModeUpdate,
};

const CANCEL: &str = "[ Cancel ]";

pub enum InventoryActionModeResult {
    AppQuit,
    Cancelled,
    UseItem(EntityId, Option<(i32, i32)>),
    DropItem(EntityId),
}

enum SubSection {
    Actions,
    Cancel,
}

enum Action {
    UseItem,
    DropItem,
}

impl Action {
    fn name(&self) -> &'static str {
        match self {
            Action::UseItem => "[ Apply ]",
            Action::DropItem => "[ Drop ]",
        }
    }
}

pub struct InventoryActionMode {
    item_id: EntityId,
    inner_width: i32,
    actions: Vec<Action>,
    subsection: SubSection,
    selection: i32,
}

/// Show a menu of actions for a single item in the player's inventory.
impl InventoryActionMode {
    pub fn new(world: &World, item_id: EntityId) -> Self {
        let actions = vec![Action::UseItem, Action::DropItem];
        let subsection = if actions.is_empty() {
            SubSection::Cancel
        } else {
            SubSection::Actions
        };

        let item_width = 2 + world.run(|names: View<Name>| names.get(item_id).0.len());
        let inner_width = std::cmp::max(
            item_width,
            std::cmp::max(
                CANCEL.len(),
                actions
                    .iter()
                    .map(|a| 2 + a.name().len())
                    .max()
                    .unwrap_or(2),
            ),
        );

        Self {
            item_id,
            inner_width: inner_width as i32,
            actions,
            subsection,
            selection: 0,
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
            h: 8 + self.actions.len() as u32,
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
        world: &World,
        inputs: &mut InputBuffer,
        pop_result: &Option<ModeResult>,
    ) -> (ModeControl, ModeUpdate) {
        if let Some(result) = pop_result {
            return match result {
                ModeResult::TargetModeResult(result) => match result {
                    TargetModeResult::AppQuit => (
                        ModeControl::Pop(InventoryActionModeResult::AppQuit.into()),
                        ModeUpdate::Immediate,
                    ),
                    TargetModeResult::Cancelled => (ModeControl::Stay, ModeUpdate::WaitForEvent),
                    TargetModeResult::Target { x, y } => (
                        ModeControl::Pop(
                            InventoryActionModeResult::UseItem(self.item_id, Some((*x, *y))).into(),
                        ),
                        ModeUpdate::Immediate,
                    ),
                },
                _ => (ModeControl::Stay, ModeUpdate::WaitForEvent),
            };
        }

        inputs.prepare_input();

        if let Some(InputEvent::AppQuit) = inputs.get_input() {
            return (
                ModeControl::Pop(InventoryActionModeResult::AppQuit.into()),
                ModeUpdate::Immediate,
            );
        } else if let Some(InputEvent::Press(keycode)) = inputs.get_input() {
            match gamekey::from_keycode(keycode, inputs.get_mods(KeyMods::SHIFT)) {
                GameKey::Down => match self.subsection {
                    SubSection::Actions => {
                        if self.selection < self.actions.len() as i32 - 1 {
                            self.selection += 1;
                        } else {
                            self.subsection = SubSection::Cancel;
                        }
                    }
                    SubSection::Cancel => {
                        if !self.actions.is_empty() {
                            self.subsection = SubSection::Actions;
                            self.selection = 0;
                        }
                    }
                },
                GameKey::Up => match self.subsection {
                    SubSection::Actions => {
                        if self.selection > 0 {
                            self.selection -= 1;
                        } else {
                            self.subsection = SubSection::Cancel;
                        }
                    }
                    SubSection::Cancel => {
                        if !self.actions.is_empty() {
                            self.subsection = SubSection::Actions;
                            self.selection = self.actions.len() as i32 - 1;
                        }
                    }
                },
                GameKey::Cancel => {
                    return (
                        ModeControl::Pop(InventoryActionModeResult::Cancelled.into()),
                        ModeUpdate::Immediate,
                    )
                }
                GameKey::Confirm => {
                    let result = match self.subsection {
                        SubSection::Actions => match self.actions[self.selection as usize] {
                            Action::UseItem => {
                                if let Some(Ranged { range }) =
                                    &world.borrow::<View<Ranged>>().try_get(self.item_id).ok()
                                {
                                    let item_name =
                                        world.borrow::<View<Name>>().get(self.item_id).0.clone();
                                    let radius = world
                                        .borrow::<View<AreaOfEffect>>()
                                        .try_get(self.item_id)
                                        .map_or(0, |aoe| aoe.radius);

                                    inputs.clear_input();
                                    return (
                                        ModeControl::Push(
                                            TargetMode::new(world, item_name, *range, radius, true)
                                                .into(),
                                        ),
                                        ModeUpdate::Immediate,
                                    );
                                } else {
                                    InventoryActionModeResult::UseItem(self.item_id, None)
                                }
                            }
                            Action::DropItem => InventoryActionModeResult::DropItem(self.item_id),
                        },
                        SubSection::Cancel => InventoryActionModeResult::Cancelled,
                    };

                    return (ModeControl::Pop(result.into()), ModeUpdate::Immediate);
                }
                _ => {}
            }
        }

        (ModeControl::Stay, ModeUpdate::WaitForEvent)
    }

    pub fn draw(&self, world: &World, grids: &mut [TileGrid<GameSym>], active: bool) {
        let grid = &mut grids[0];
        let fg = ui::color::WHITE;
        let bg = ui::color::BLACK;
        let selected_bg = ui::color::SELECTED_BG;

        grid.view.color_mod = if active {
            ui::color::WHITE
        } else {
            ui::color::GRAY
        };

        grid.draw_box((0, 0), (grid.width(), grid.height()), fg, bg);

        world.run(|names: View<Name>, renderables: View<Renderable>| {
            let render = renderables.get(self.item_id);

            grid.put_sym_color((2, 2), render.sym, render.fg, render.bg);
            grid.print_color((4, 2), &names.get(self.item_id).0, fg, bg);
        });

        for (i, action) in self.actions.iter().enumerate() {
            grid.print_color(
                (4, 4 + i as i32),
                action.name(),
                fg,
                if matches!(self.subsection, SubSection::Actions) && i as i32 == self.selection {
                    selected_bg
                } else {
                    bg
                },
            );
        }

        grid.print_color(
            (4, grid.height() as i32 - 3),
            CANCEL,
            fg,
            if matches!(self.subsection, SubSection::Cancel) {
                selected_bg
            } else {
                bg
            },
        );
    }
}
