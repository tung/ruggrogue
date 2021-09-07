use shipyard::{EntityId, Get, UniqueView, View, World};

use crate::{
    components::{AreaOfEffect, Consumable, EquipSlot, Name, Ranged, Renderable, Victory},
    gamekey::{self, GameKey},
    gamesym::GameSym,
    ui::{self, Options},
};
use ruggrogue::{
    util::{Color, Size},
    InputBuffer, InputEvent, KeyMods, TileGrid, Tileset,
};

use super::{
    target::{TargetMode, TargetModeResult},
    ModeControl, ModeResult, ModeUpdate,
};

const CANCEL: &str = "[ Cancel ]";

pub enum InventoryActionModeResult {
    AppQuit,
    Cancelled,
    EquipItem(EntityId),
    UseItem(EntityId, Option<(i32, i32)>),
    DropItem(EntityId),
}

enum SubSection {
    Actions,
    Cancel,
}

#[allow(clippy::enum_variant_names)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum InventoryAction {
    EquipItem,
    UseItem,
    DropItem,
}

impl InventoryAction {
    pub fn from_key(key: GameKey) -> Option<Self> {
        match key {
            GameKey::EquipItem => Some(InventoryAction::EquipItem),
            GameKey::UseItem => Some(InventoryAction::UseItem),
            GameKey::DropItem => Some(InventoryAction::DropItem),
            _ => None,
        }
    }

    pub fn item_supports_action(world: &World, item_id: EntityId, action: InventoryAction) -> bool {
        match action {
            InventoryAction::EquipItem => world.borrow::<View<EquipSlot>>().contains(item_id),
            InventoryAction::UseItem => {
                world.borrow::<View<Consumable>>().contains(item_id)
                    | world.borrow::<View<Victory>>().contains(item_id)
            }
            InventoryAction::DropItem => true,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            InventoryAction::EquipItem => "Equip",
            InventoryAction::UseItem => "Apply",
            InventoryAction::DropItem => "Drop",
        }
    }

    fn label(&self) -> &'static str {
        match self {
            InventoryAction::EquipItem => "[ Equip ]",
            InventoryAction::UseItem => "[ Apply ]",
            InventoryAction::DropItem => "[ Drop ]",
        }
    }
}

pub struct InventoryActionMode {
    item_id: EntityId,
    inner_width: i32,
    actions: Vec<InventoryAction>,
    subsection: SubSection,
    selection: i32,
}

/// Show a menu of actions for a single item in the player's inventory.
impl InventoryActionMode {
    pub fn new(world: &World, item_id: EntityId, default_action: Option<InventoryAction>) -> Self {
        let actions = [
            InventoryAction::EquipItem,
            InventoryAction::UseItem,
            InventoryAction::DropItem,
        ]
        .iter()
        .filter(|action| InventoryAction::item_supports_action(world, item_id, **action))
        .copied()
        .collect::<Vec<_>>();
        let subsection = if actions.is_empty() {
            SubSection::Cancel
        } else {
            SubSection::Actions
        };
        let selection = default_action
            .and_then(|d_act| actions.iter().position(|a| *a == d_act))
            .unwrap_or(0);
        let item_width = world.borrow::<View<Name>>().get(item_id).0.len();
        let inner_width = 2 + item_width
            .max(CANCEL.len())
            .max(actions.iter().map(|a| a.label().len()).max().unwrap_or(0));

        Self {
            item_id,
            inner_width: inner_width as i32,
            actions,
            subsection,
            selection: selection as i32,
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

    fn confirm_action(&self, world: &World, inputs: &mut InputBuffer) -> (ModeControl, ModeUpdate) {
        let result = match self.subsection {
            SubSection::Actions => match self.actions[self.selection as usize] {
                InventoryAction::EquipItem => InventoryActionModeResult::EquipItem(self.item_id),
                InventoryAction::UseItem => {
                    if let Some(Ranged { range }) =
                        &world.borrow::<View<Ranged>>().try_get(self.item_id).ok()
                    {
                        let item_name = world.borrow::<View<Name>>().get(self.item_id).0.clone();
                        let radius = world
                            .borrow::<View<AreaOfEffect>>()
                            .try_get(self.item_id)
                            .map_or(0, |aoe| aoe.radius);

                        inputs.clear_input();
                        return (
                            ModeControl::Push(
                                TargetMode::new(world, item_name, *range, radius, true).into(),
                            ),
                            ModeUpdate::Immediate,
                        );
                    } else {
                        InventoryActionModeResult::UseItem(self.item_id, None)
                    }
                }
                InventoryAction::DropItem => InventoryActionModeResult::DropItem(self.item_id),
            },
            SubSection::Cancel => InventoryActionModeResult::Cancelled,
        };

        (ModeControl::Pop(result.into()), ModeUpdate::Immediate)
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
                GameKey::Confirm => return self.confirm_action(world, inputs),
                key @ GameKey::EquipItem | key @ GameKey::UseItem | key @ GameKey::DropItem => {
                    if let Some(inv_action) = InventoryAction::from_key(key) {
                        if let Some(action_pos) = self.actions.iter().position(|a| *a == inv_action)
                        {
                            if matches!(self.subsection, SubSection::Actions)
                                && self.selection == action_pos as i32
                            {
                                return self.confirm_action(world, inputs);
                            } else {
                                self.subsection = SubSection::Actions;
                                self.selection = action_pos as i32;
                            }
                        }
                    }
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

        world.run(|names: View<Name>, renderables: View<Renderable>| {
            let render = renderables.get(self.item_id);

            grid.put_sym_color((2, 2), render.sym, render.fg, render.bg);
            grid.print_color((4, 2), &names.get(self.item_id).0, true, fg, bg);
        });

        for (i, action) in self.actions.iter().enumerate() {
            grid.print_color(
                (4, 4 + i as i32),
                action.label(),
                true,
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
            true,
            fg,
            if matches!(self.subsection, SubSection::Cancel) {
                selected_bg
            } else {
                bg
            },
        );
    }
}
