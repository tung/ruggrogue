use shipyard::{EntityId, Get, UniqueView, UniqueViewMut, View, World};

use crate::{
    components::{Equipment, Inventory, Name, Renderable},
    gamekey::{self, GameKey},
    gamesym::GameSym,
    item,
    menu_memory::MenuMemory,
    player::PlayerId,
    ui::{self, Options},
};
use ruggrogue::{
    util::{Color, Size},
    InputBuffer, InputEvent, KeyMods, TileGrid, Tileset,
};

use super::{
    equipment_action::{EquipmentAction, EquipmentActionMode, EquipmentActionModeResult},
    inventory_action::{InventoryAction, InventoryActionMode, InventoryActionModeResult},
    yes_no_dialog::{YesNoDialogMode, YesNoDialogModeResult},
    ModeControl, ModeResult, ModeUpdate,
};

const EQUIP_GRID: usize = 0;
const INV_GRID: usize = 1;

pub enum InventoryModeResult {
    AppQuit,
    DoNothing,
    RemoveEquipment(EntityId),
    DropEquipment(EntityId),
    EquipItem(EntityId),
    UseItem(EntityId, Option<(i32, i32)>),
    DropItem(EntityId),
}

enum SubSection {
    EquipWeapon,
    EquipArmor,
    SortAll,
    Inventory,
}

pub struct InventoryMode {
    main_width: i32,
    subsection: SubSection,
    inv_selection: i32,
}

/// Show a screen with items carried by the player, and allow them to be manipulated.
impl InventoryMode {
    pub fn new(world: &World) -> Self {
        let player_id = world.borrow::<UniqueView<PlayerId>>();
        let inventories = world.borrow::<View<Inventory>>();
        let names = world.borrow::<View<Name>>();
        let player_inventory = inventories.get(player_id.0);
        let inv_min_width = player_inventory
            .items
            .iter()
            .map(|it| names.get(*it).0.len() + 2)
            .max()
            .unwrap_or(0);
        let inv_selection = world.borrow::<UniqueView<MenuMemory>>()[MenuMemory::INVENTORY]
            .min(player_inventory.items.len().saturating_sub(1) as i32);

        Self {
            main_width: std::cmp::max(30, inv_min_width as i32),
            subsection: SubSection::Inventory,
            inv_selection,
        }
    }

    /// The height of the item list in the inventory grid as an i32 for convenience.
    fn inv_item_list_height(grid: &TileGrid<GameSym>) -> i32 {
        grid.height().saturating_sub(6).max(1).min(i32::MAX as u32) as i32
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

        // Equip grid on top.
        let new_equip_size = Size {
            w: 4 + self.main_width as u32,
            h: 5,
        };
        // Inventory grid occupies the majority center bottom-right.
        let inv_len = world.run(
            |player_id: UniqueView<PlayerId>, inventories: View<Inventory>| {
                inventories.get(player_id.0).items.len() as u32
            },
        );
        let new_inv_size = Size {
            w: new_equip_size.w,
            h: (inv_len + 6)
                .max(13)
                .min(
                    (window_size.h / (tileset.tile_height() * text_zoom))
                        .saturating_sub(new_equip_size.h),
                )
                .max(7),
        };

        if !grids.is_empty() {
            grids[EQUIP_GRID].resize(new_equip_size);
            grids[INV_GRID].resize(new_inv_size);
        } else {
            grids.push(TileGrid::new(new_equip_size, tilesets, font as usize));
            grids.push(TileGrid::new(new_inv_size, tilesets, font as usize));
            grids[EQUIP_GRID].view.clear_color = None;
            grids[INV_GRID].view.clear_color = None;
        }

        let (equip_grid, grids) = grids.split_first_mut().unwrap(); // EQUIP_GRID
        let (inv_grid, _) = grids.split_first_mut().unwrap(); // INV_GRID

        // Set fonts.
        equip_grid.set_tileset(tilesets, font as usize);
        inv_grid.set_tileset(tilesets, font as usize);

        // Calculate equip grid x and width.
        equip_grid.view.size.w = new_equip_size.w * tileset.tile_width() * text_zoom;
        equip_grid.view.pos.x = (window_size.w - equip_grid.view.size.w) as i32 / 2;

        // Calculate inventory grid x and width.
        inv_grid.view.size.w = new_inv_size.w * tileset.tile_width() * text_zoom;
        inv_grid.view.pos.x = equip_grid.view.pos.x;

        // Calculate equip grid y and height.
        let combined_px_height =
            (new_inv_size.h + new_equip_size.h) * tileset.tile_height() * text_zoom;
        if combined_px_height <= window_size.h {
            equip_grid.view.pos.y = (window_size.h - combined_px_height) as i32 / 2;
            equip_grid.view.size.h = new_equip_size.h * tileset.tile_height() * text_zoom;
            equip_grid.view.visible = true;
        } else if new_inv_size.h * tileset.tile_height() * text_zoom < window_size.h {
            equip_grid.view.pos.y = 0;
            equip_grid.view.size.h =
                window_size.h - new_inv_size.h * tileset.tile_height() * text_zoom;
            equip_grid.view.visible = true;
        } else {
            equip_grid.view.pos.y = 0;
            equip_grid.view.size.h = 0;
            equip_grid.view.visible = false;
        }

        // Calculate inventory grid y and height.
        inv_grid.view.pos.y = equip_grid.view.pos.y + equip_grid.view.size.h as i32;
        inv_grid.view.size.h = new_inv_size.h * tileset.tile_height() * text_zoom;

        // Set all grids to current text zoom.
        equip_grid.view.zoom = text_zoom;
        inv_grid.view.zoom = text_zoom;
    }

    pub fn update(
        &mut self,
        world: &World,
        inputs: &mut InputBuffer,
        grids: &[TileGrid<GameSym>],
        pop_result: &Option<ModeResult>,
    ) -> (ModeControl, ModeUpdate) {
        if let Some(result) = pop_result {
            return match result {
                ModeResult::EquipmentActionModeResult(result) => match result {
                    EquipmentActionModeResult::AppQuit => (
                        ModeControl::Pop(InventoryModeResult::AppQuit.into()),
                        ModeUpdate::Immediate,
                    ),
                    EquipmentActionModeResult::Cancelled => {
                        (ModeControl::Stay, ModeUpdate::WaitForEvent)
                    }
                    EquipmentActionModeResult::RemoveEquipment(item_id) => (
                        ModeControl::Pop(InventoryModeResult::RemoveEquipment(*item_id).into()),
                        ModeUpdate::Immediate,
                    ),
                    EquipmentActionModeResult::DropEquipment(item_id) => (
                        ModeControl::Pop(InventoryModeResult::DropEquipment(*item_id).into()),
                        ModeUpdate::Immediate,
                    ),
                },

                ModeResult::InventoryActionModeResult(result) => match result {
                    InventoryActionModeResult::AppQuit => (
                        ModeControl::Pop(InventoryModeResult::AppQuit.into()),
                        ModeUpdate::Immediate,
                    ),
                    InventoryActionModeResult::Cancelled => {
                        (ModeControl::Stay, ModeUpdate::WaitForEvent)
                    }
                    InventoryActionModeResult::EquipItem(item_id) => (
                        ModeControl::Pop(InventoryModeResult::EquipItem(*item_id).into()),
                        ModeUpdate::Immediate,
                    ),
                    InventoryActionModeResult::UseItem(item_id, target) => (
                        ModeControl::Pop(InventoryModeResult::UseItem(*item_id, *target).into()),
                        ModeUpdate::Immediate,
                    ),
                    InventoryActionModeResult::DropItem(item_id) => (
                        ModeControl::Pop(InventoryModeResult::DropItem(*item_id).into()),
                        ModeUpdate::Immediate,
                    ),
                },

                ModeResult::YesNoDialogModeResult(result) => match result {
                    YesNoDialogModeResult::AppQuit => (
                        ModeControl::Pop(InventoryModeResult::AppQuit.into()),
                        ModeUpdate::Immediate,
                    ),
                    YesNoDialogModeResult::Yes => {
                        let player_id = world.borrow::<UniqueView<PlayerId>>();
                        item::sort_inventory(world, player_id.0);

                        // Reset menu memory for inventory-related shortcut menus.
                        let mut menu_memory = world.borrow::<UniqueViewMut<MenuMemory>>();
                        menu_memory[MenuMemory::INVENTORY_SHORTCUT_EQUIP] = 0;
                        menu_memory[MenuMemory::INVENTORY_SHORTCUT_USE] = 0;
                        menu_memory[MenuMemory::INVENTORY_SHORTCUT_DROP] = 0;

                        (ModeControl::Stay, ModeUpdate::WaitForEvent)
                    }
                    YesNoDialogModeResult::No => (ModeControl::Stay, ModeUpdate::WaitForEvent),
                },

                _ => unreachable!(),
            };
        }

        inputs.prepare_input();

        if let Some(InputEvent::AppQuit) = inputs.get_input() {
            (
                ModeControl::Pop(InventoryModeResult::AppQuit.into()),
                ModeUpdate::Immediate,
            )
        } else if let Some(InputEvent::Press(keycode)) = inputs.get_input() {
            let player_id = world.borrow::<UniqueView<PlayerId>>();
            let equipments = world.borrow::<View<Equipment>>();
            let inventories = world.borrow::<View<Inventory>>();
            let player_equipment = equipments.get(player_id.0);
            let player_inv = inventories.get(player_id.0);
            let shift = inputs.get_mods(KeyMods::SHIFT);

            match (&self.subsection, gamekey::from_keycode(keycode, shift)) {
                (SubSection::EquipWeapon, GameKey::Up) => {
                    self.subsection = SubSection::Inventory;
                    self.inv_selection = if player_inv.items.is_empty() {
                        0
                    } else {
                        player_inv.items.len() as i32 - 1
                    }
                }
                (SubSection::EquipWeapon, GameKey::Down) => {
                    self.subsection = SubSection::EquipArmor;
                }
                (SubSection::EquipWeapon, GameKey::Confirm) => {
                    if let Some(weapon) = player_equipment.weapon {
                        inputs.clear_input();
                        return (
                            ModeControl::Push(EquipmentActionMode::new(world, weapon, None).into()),
                            ModeUpdate::Immediate,
                        );
                    }
                }
                (SubSection::EquipWeapon, key)
                    if matches!(key, GameKey::RemoveItem | GameKey::DropItem) =>
                {
                    if let Some(weapon) = player_equipment.weapon {
                        if let Some(equip_action) = EquipmentAction::from_key(key) {
                            inputs.clear_input();
                            return (
                                ModeControl::Push(
                                    EquipmentActionMode::new(world, weapon, Some(equip_action))
                                        .into(),
                                ),
                                ModeUpdate::Immediate,
                            );
                        }
                    }
                }

                (SubSection::EquipArmor, GameKey::Up) => {
                    self.subsection = SubSection::EquipWeapon;
                }
                (SubSection::EquipArmor, GameKey::Down) => {
                    self.subsection = SubSection::SortAll;
                }
                (SubSection::EquipArmor, GameKey::Confirm) => {
                    if let Some(armor) = player_equipment.armor {
                        inputs.clear_input();
                        return (
                            ModeControl::Push(EquipmentActionMode::new(world, armor, None).into()),
                            ModeUpdate::Immediate,
                        );
                    }
                }
                (SubSection::EquipArmor, key)
                    if matches!(key, GameKey::RemoveItem | GameKey::DropItem) =>
                {
                    if let Some(armor) = player_equipment.armor {
                        if let Some(equip_action) = EquipmentAction::from_key(key) {
                            inputs.clear_input();
                            return (
                                ModeControl::Push(
                                    EquipmentActionMode::new(world, armor, Some(equip_action))
                                        .into(),
                                ),
                                ModeUpdate::Immediate,
                            );
                        }
                    }
                }

                (SubSection::SortAll, GameKey::Up) => {
                    self.subsection = SubSection::EquipArmor;
                }
                (SubSection::SortAll, GameKey::Down) => {
                    self.subsection = SubSection::Inventory;
                    self.inv_selection = 0;
                }
                (SubSection::SortAll, GameKey::Confirm) => {
                    inputs.clear_input();
                    return (
                        ModeControl::Push(
                            YesNoDialogMode::new("Sort all inventory items?".to_string(), true)
                                .into(),
                        ),
                        ModeUpdate::Immediate,
                    );
                }

                (SubSection::Inventory, GameKey::Up) => {
                    if self.inv_selection > 0 {
                        self.inv_selection -= 1;
                    } else {
                        self.subsection = SubSection::SortAll;
                    }
                }
                (SubSection::Inventory, GameKey::Down) => {
                    if !player_inv.items.is_empty()
                        && self.inv_selection < player_inv.items.len() as i32 - 1
                    {
                        self.inv_selection += 1;
                    } else {
                        self.subsection = SubSection::EquipWeapon;
                    }
                }
                (SubSection::Inventory, GameKey::PageUp) => {
                    if let Some(inv_grid) = grids.get(INV_GRID) {
                        self.inv_selection = self
                            .inv_selection
                            .saturating_sub(Self::inv_item_list_height(inv_grid))
                            .max(0);
                    }
                }
                (SubSection::Inventory, GameKey::PageDown) => {
                    if let Some(inv_grid) = grids.get(INV_GRID) {
                        let max_selection = (player_inv.items.len() as i32 - 1).max(0);

                        self.inv_selection = self
                            .inv_selection
                            .saturating_add(Self::inv_item_list_height(inv_grid))
                            .min(max_selection);
                    }
                }
                (SubSection::Inventory, GameKey::Home) => {
                    self.inv_selection = 0;
                }
                (SubSection::Inventory, GameKey::End) => {
                    self.inv_selection = (player_inv.items.len() as i32 - 1).max(0);
                }
                (SubSection::Inventory, GameKey::Confirm) => {
                    if !player_inv.items.is_empty() {
                        inputs.clear_input();
                        return (
                            ModeControl::Push(
                                InventoryActionMode::new(
                                    world,
                                    player_inv.items[self.inv_selection as usize],
                                    None,
                                )
                                .into(),
                            ),
                            ModeUpdate::Immediate,
                        );
                    }
                }
                (SubSection::Inventory, key)
                    if matches!(
                        key,
                        GameKey::EquipItem | GameKey::UseItem | GameKey::DropItem
                    ) =>
                {
                    if let Some(item_id) = player_inv.items.get(self.inv_selection as usize) {
                        if let Some(inv_action) = InventoryAction::from_key(key) {
                            if InventoryAction::item_supports_action(world, *item_id, inv_action) {
                                inputs.clear_input();
                                return (
                                    ModeControl::Push(
                                        InventoryActionMode::new(world, *item_id, Some(inv_action))
                                            .into(),
                                    ),
                                    ModeUpdate::Immediate,
                                );
                            }
                        }
                    }
                }

                (_, GameKey::Cancel) | (_, GameKey::Inventory) => {
                    return (
                        ModeControl::Pop(InventoryModeResult::DoNothing.into()),
                        ModeUpdate::Immediate,
                    )
                }
                _ => {}
            }

            world.borrow::<UniqueViewMut<MenuMemory>>()[MenuMemory::INVENTORY] = self.inv_selection;

            (ModeControl::Stay, ModeUpdate::WaitForEvent)
        } else {
            (ModeControl::Stay, ModeUpdate::WaitForEvent)
        }
    }

    fn draw_equip(
        &self,
        world: &World,
        grid: &mut TileGrid<GameSym>,
        fg: Color,
        bg: Color,
        selected_bg: Color,
    ) {
        let equipments = world.borrow::<View<Equipment>>();
        let names = world.borrow::<View<Name>>();
        let renderables = world.borrow::<View<Renderable>>();
        let player_equipment = equipments.get(world.borrow::<UniqueView<PlayerId>>().0);
        let weapon_bg = if matches!(self.subsection, SubSection::EquipWeapon) {
            selected_bg
        } else {
            bg
        };
        let armor_bg = if matches!(self.subsection, SubSection::EquipArmor) {
            selected_bg
        } else {
            bg
        };

        // Draw box with bottom edge off-grid.
        grid.draw_box((0, 0), (grid.width(), grid.height() + 1), fg, bg);
        grid.print_color((2, 0), "< Equipment >", true, Color::YELLOW, bg);

        grid.print((2, 2), "Weapon:");
        if let Some(weapon) = player_equipment.weapon {
            let render = renderables.get(weapon);
            grid.put_sym_color((10, 2), render.sym, render.fg, render.bg);
            grid.print_color((12, 2), &names.get(weapon).0, true, fg, weapon_bg);
        } else {
            grid.print_color((10, 2), "-- nothing --", true, fg, weapon_bg);
        }

        grid.print((2, 3), "Armor:");
        if let Some(armor) = player_equipment.armor {
            let render = renderables.get(armor);
            grid.put_sym_color((10, 3), render.sym, render.fg, render.bg);
            grid.print_color((12, 3), &names.get(armor).0, true, fg, armor_bg);
        } else {
            grid.print_color((10, 3), "-- nothing --", true, fg, armor_bg);
        }
    }

    fn draw_inventory(
        &self,
        world: &World,
        grid: &mut TileGrid<GameSym>,
        fg: Color,
        bg: Color,
        selected_bg: Color,
    ) {
        grid.draw_box((0, 0), (grid.width(), grid.height()), fg, bg);
        grid.put_char_color((0, 0), '├', fg, bg);
        grid.put_char_color((grid.width() as i32 - 1, 0), '┤', fg, bg);
        grid.print_color((2, 0), "< Inventory >", true, Color::YELLOW, bg);

        grid.print_color(
            (2, 2),
            "[ Sort all items ]",
            true,
            fg,
            if matches!(self.subsection, SubSection::SortAll) {
                selected_bg
            } else {
                bg
            },
        );

        world.run(
            |player_id: UniqueView<PlayerId>,
             inventories: View<Inventory>,
             names: View<Name>,
             renderables: View<Renderable>| {
                let player_inv = inventories.get(player_id.0);
                let item_x = 2;
                let item_y = 4;

                if player_inv.items.is_empty() {
                    grid.print_color(
                        (item_x, item_y),
                        "-- nothing --",
                        true,
                        fg,
                        if matches!(self.subsection, SubSection::Inventory) {
                            selected_bg
                        } else {
                            bg
                        },
                    );
                } else {
                    let item_height = Self::inv_item_list_height(grid);
                    let item_offset = std::cmp::max(
                        0,
                        std::cmp::min(
                            player_inv.items.len() as i32 - item_height,
                            self.inv_selection - (item_height - 1) / 2,
                        ),
                    );

                    if player_inv.items.len() as i32 > item_height {
                        grid.draw_bar(
                            true,
                            (grid.width() as i32 - 1, item_y),
                            item_height,
                            item_offset,
                            item_height,
                            player_inv.items.len() as i32,
                            fg,
                            bg,
                        );
                    }

                    for (i, item_id) in player_inv
                        .items
                        .iter()
                        .enumerate()
                        .skip(item_offset as usize)
                        .take(item_height as usize)
                    {
                        let render = renderables.get(*item_id);

                        grid.put_sym_color(
                            (item_x, item_y + i as i32 - item_offset),
                            render.sym,
                            render.fg,
                            render.bg,
                        );

                        grid.print_color(
                            (item_x + 2, item_y + i as i32 - item_offset),
                            &names.get(*item_id).0,
                            true,
                            fg,
                            if matches!(self.subsection, SubSection::Inventory)
                                && i as i32 == self.inv_selection
                            {
                                selected_bg
                            } else {
                                bg
                            },
                        );
                    }
                }
            },
        );
    }

    pub fn draw(&self, world: &World, grids: &mut [TileGrid<GameSym>], active: bool) {
        let (equip_grid, grids) = grids.split_first_mut().unwrap(); // EQUIP_GRID
        let (inv_grid, _) = grids.split_first_mut().unwrap(); // INV_GRID
        let fg = Color::WHITE;
        let bg = Color::BLACK;
        let selected_bg = ui::SELECTED_BG;

        if active {
            equip_grid.view.color_mod = Color::WHITE;
            inv_grid.view.color_mod = Color::WHITE;
        } else {
            equip_grid.view.color_mod = Color::GRAY;
            inv_grid.view.color_mod = Color::GRAY;
        }

        if equip_grid.view.visible {
            self.draw_equip(world, equip_grid, fg, bg, selected_bg);
        }
        self.draw_inventory(world, inv_grid, fg, bg, selected_bg);
    }
}
