use shipyard::{EntityId, Get, UniqueView, UniqueViewMut, View, World};

use crate::{
    components::{AreaOfEffect, Inventory, Name, Ranged, Renderable},
    gamekey::{self, GameKey},
    gamesym::GameSym,
    menu_memory::MenuMemory,
    message::Messages,
    player::PlayerId,
    ui::{self, Options},
};
use ruggrogue::{
    util::{Color, Size},
    InputBuffer, InputEvent, KeyMods, TileGrid, Tileset,
};

use super::{
    inventory_action::InventoryAction,
    target::{TargetMode, TargetModeResult},
    ModeControl, ModeResult, ModeUpdate,
};

const CANCEL: &str = "[ Cancel ]";

pub enum InventoryShortcutModeResult {
    AppQuit,
    Cancelled,
    EquipItem(EntityId),
    UseItem(EntityId, Option<(i32, i32)>),
    DropItem(EntityId),
}

enum SubSection {
    Items,
    Cancel,
}

pub struct InventoryShortcutMode {
    action: InventoryAction,
    title: String,
    prompt: String,
    items: Vec<EntityId>,
    inner_width: i32,
    subsection: SubSection,
    selection: i32,
}

/// Show inventory items for which a given action can be performed, shortcutting the inventory and
/// inventory action modes.
impl InventoryShortcutMode {
    pub fn new(world: &World, action: InventoryAction) -> Self {
        let menu_memory = world.borrow::<UniqueView<MenuMemory>>();
        let player_id = world.borrow::<UniqueView<PlayerId>>();
        let inventories = world.borrow::<View<Inventory>>();
        let names = world.borrow::<View<Name>>();
        let player_inv = inventories.get(player_id.0);
        let items = player_inv
            .items
            .iter()
            .filter(|it| InventoryAction::item_supports_action(world, **it, action))
            .copied()
            .collect::<Vec<EntityId>>();
        let title = format!("< {} Item >", action.name());
        let prompt = format!("{} which item?", action.name());
        let inner_width = title.len().max(prompt.len()).max(CANCEL.len()).max(
            items
                .iter()
                .map(|it| names.get(*it).0.len() + 2)
                .max()
                .unwrap_or(2),
        );
        let selection = match action {
            InventoryAction::EquipItem => menu_memory[MenuMemory::INVENTORY_SHORTCUT_EQUIP],
            InventoryAction::UseItem => menu_memory[MenuMemory::INVENTORY_SHORTCUT_USE],
            InventoryAction::DropItem => menu_memory[MenuMemory::INVENTORY_SHORTCUT_DROP],
        };
        let selection = selection.min(items.len().saturating_sub(1) as i32);

        Self {
            action,
            title,
            prompt,
            items,
            inner_width: inner_width as i32,
            subsection: SubSection::Items,
            selection,
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
        let tileset = &tilesets.get(font as usize).unwrap_or(&tilesets[0]);
        let new_grid_size = Size {
            w: self.inner_width as u32 + 4,
            h: (8 + self.items.len() as u32)
                .min(window_size.h / (tileset.tile_height() * text_zoom))
                .max(9),
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

    /// The height of the item list as an i32 for convenience.
    fn item_list_height(grid: &TileGrid<GameSym>) -> i32 {
        grid.height().saturating_sub(8).max(1).min(i32::MAX as u32) as i32
    }

    fn confirm_action(&self, world: &World, inputs: &mut InputBuffer) -> (ModeControl, ModeUpdate) {
        let item_id = self.items[self.selection as usize];
        let result = match self.subsection {
            SubSection::Items => match self.action {
                InventoryAction::EquipItem => InventoryShortcutModeResult::EquipItem(item_id),
                InventoryAction::UseItem => {
                    if let Some(Ranged { range }) =
                        &world.borrow::<View<Ranged>>().try_get(item_id).ok()
                    {
                        let item_name = world.borrow::<View<Name>>().get(item_id).0.clone();
                        let radius = world
                            .borrow::<View<AreaOfEffect>>()
                            .try_get(item_id)
                            .map_or(0, |aoe| aoe.radius);

                        inputs.clear_input();
                        return (
                            ModeControl::Push(
                                TargetMode::new(world, item_name, *range, radius, true).into(),
                            ),
                            ModeUpdate::Immediate,
                        );
                    } else {
                        InventoryShortcutModeResult::UseItem(item_id, None)
                    }
                }
                InventoryAction::DropItem => InventoryShortcutModeResult::DropItem(item_id),
            },
            SubSection::Cancel => InventoryShortcutModeResult::Cancelled,
        };

        (ModeControl::Pop(result.into()), ModeUpdate::Immediate)
    }

    pub fn update(
        &mut self,
        world: &World,
        inputs: &mut InputBuffer,
        grids: &[TileGrid<GameSym>],
        pop_result: &Option<ModeResult>,
    ) -> (ModeControl, ModeUpdate) {
        if self.items.is_empty() {
            world.borrow::<UniqueViewMut<Messages>>().add(format!(
                "You have no items in your inventory to {}.",
                self.action.name().to_lowercase(),
            ));

            (
                ModeControl::Pop(InventoryShortcutModeResult::Cancelled.into()),
                ModeUpdate::Immediate,
            )
        } else if let Some(result) = pop_result {
            match result {
                ModeResult::TargetModeResult(result) => match result {
                    TargetModeResult::AppQuit => (
                        ModeControl::Pop(InventoryShortcutModeResult::AppQuit.into()),
                        ModeUpdate::Immediate,
                    ),
                    TargetModeResult::Cancelled => (ModeControl::Stay, ModeUpdate::WaitForEvent),
                    TargetModeResult::Target { x, y } => (
                        ModeControl::Pop(
                            InventoryShortcutModeResult::UseItem(
                                self.items[self.selection as usize],
                                Some((*x, *y)),
                            )
                            .into(),
                        ),
                        ModeUpdate::Immediate,
                    ),
                },
                _ => unreachable!(),
            }
        } else {
            inputs.prepare_input();

            if let Some(InputEvent::AppQuit) = inputs.get_input() {
                return (
                    ModeControl::Pop(InventoryShortcutModeResult::AppQuit.into()),
                    ModeUpdate::Immediate,
                );
            } else if let Some(InputEvent::Press(keycode)) = inputs.get_input() {
                match gamekey::from_keycode(keycode, inputs.get_mods(KeyMods::SHIFT)) {
                    GameKey::Up => match self.subsection {
                        SubSection::Items => {
                            if self.selection > 0 {
                                self.selection -= 1;
                            } else {
                                self.subsection = SubSection::Cancel;
                            }
                        }
                        SubSection::Cancel => {
                            self.subsection = SubSection::Items;
                            self.selection = self.items.len() as i32 - 1;
                        }
                    },
                    GameKey::Down => match self.subsection {
                        SubSection::Items => {
                            if self.selection < self.items.len() as i32 - 1 {
                                self.selection += 1;
                            } else {
                                self.subsection = SubSection::Cancel;
                            }
                        }
                        SubSection::Cancel => {
                            self.subsection = SubSection::Items;
                            self.selection = 0;
                        }
                    },
                    GameKey::PageUp => {
                        if matches!(self.subsection, SubSection::Items) {
                            if let Some(grid) = grids.get(0) {
                                self.selection = self
                                    .selection
                                    .saturating_sub(Self::item_list_height(grid))
                                    .max(0);
                            }
                        }
                    }
                    GameKey::PageDown => {
                        if matches!(self.subsection, SubSection::Items) {
                            if let Some(grid) = grids.get(0) {
                                let max_selection = (self.items.len() as i32 - 1).max(0);

                                self.selection = self
                                    .selection
                                    .saturating_add(Self::item_list_height(grid))
                                    .min(max_selection);
                            }
                        }
                    }
                    GameKey::Home => {
                        if matches!(self.subsection, SubSection::Items) {
                            self.selection = 0;
                        }
                    }
                    GameKey::End => {
                        if matches!(self.subsection, SubSection::Items) {
                            self.selection = (self.items.len() as i32 - 1).max(0);
                        }
                    }
                    GameKey::Cancel => {
                        return (
                            ModeControl::Pop(InventoryShortcutModeResult::Cancelled.into()),
                            ModeUpdate::Immediate,
                        )
                    }
                    GameKey::Confirm => return self.confirm_action(world, inputs),
                    key => {
                        if let Some(action) = InventoryAction::from_key(key) {
                            if action == self.action && matches!(self.subsection, SubSection::Items)
                            {
                                return self.confirm_action(world, inputs);
                            }
                        }
                    }
                }

                // Update inventory shortcut menu memory for the matching action.
                {
                    let mut menu_memory = world.borrow::<UniqueViewMut<MenuMemory>>();
                    let menu_memory = match self.action {
                        InventoryAction::EquipItem => {
                            &mut menu_memory[MenuMemory::INVENTORY_SHORTCUT_EQUIP]
                        }
                        InventoryAction::UseItem => {
                            &mut menu_memory[MenuMemory::INVENTORY_SHORTCUT_USE]
                        }
                        InventoryAction::DropItem => {
                            &mut menu_memory[MenuMemory::INVENTORY_SHORTCUT_DROP]
                        }
                    };

                    *menu_memory = self.selection;
                }
            }

            (ModeControl::Stay, ModeUpdate::WaitForEvent)
        }
    }

    pub fn draw(&self, world: &World, grids: &mut [TileGrid<GameSym>], active: bool) {
        let grid = &mut grids[0];
        let width = grid.width();
        let height = grid.height();
        let fg = Color::WHITE;
        let bg = Color::BLACK;
        let selected_bg = ui::SELECTED_BG;

        grid.view.color_mod = if active { Color::WHITE } else { Color::GRAY };

        grid.draw_box((0, 0), (width, height), fg, bg);
        grid.print_color((2, 0), &self.title, true, Color::YELLOW, bg);
        grid.print((2, 2), &self.prompt);

        let list_height = Self::item_list_height(grid);
        let list_offset = (self.selection - (list_height - 1) / 2)
            .min(self.items.len() as i32 - list_height)
            .max(0);

        if self.items.len() as i32 > list_height {
            grid.draw_bar(
                true,
                (width as i32 - 1, 4),
                list_height,
                list_offset,
                list_height,
                self.items.len() as i32,
                fg,
                bg,
            );
        }

        {
            let names = world.borrow::<View<Name>>();
            let renderables = world.borrow::<View<Renderable>>();

            for (i, item_id) in self
                .items
                .iter()
                .enumerate()
                .skip(list_offset as usize)
                .take(list_height as usize)
            {
                let render = renderables.get(*item_id);

                grid.put_sym_color(
                    (2, 4 + i as i32 - list_offset),
                    render.sym,
                    render.fg,
                    render.bg,
                );

                grid.print_color(
                    (4, 4 + i as i32 - list_offset),
                    &names.get(*item_id).0,
                    true,
                    fg,
                    if matches!(self.subsection, SubSection::Items) && i as i32 == self.selection {
                        selected_bg
                    } else {
                        bg
                    },
                );
            }
        }

        grid.print_color(
            (4, height as i32 - 3),
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
