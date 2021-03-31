use shipyard::{EntityId, Get, UniqueView, View, World};

use crate::{
    components::{Inventory, Name, Renderable},
    gamekey::{self, GameKey},
    gamesym::GameSym,
    player::PlayerId,
    ui::{self, Options},
};
use ruggle::{
    util::{Color, Size},
    InputBuffer, InputEvent, KeyMods, TileGrid, Tileset,
};

use super::{
    inventory_action::{InventoryActionMode, InventoryActionModeResult},
    ModeControl, ModeResult, ModeUpdate,
};

const EQUIP_GRID: usize = 0;
const INV_GRID: usize = 1;

pub enum InventoryModeResult {
    AppQuit,
    DoNothing,
    UseItem(EntityId, Option<(i32, i32)>),
    DropItem(EntityId),
}

enum SubSection {
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
        let inv_min_width = world.run(
            |player_id: UniqueView<PlayerId>, inventories: View<Inventory>, names: View<Name>| {
                inventories
                    .get(player_id.0)
                    .items
                    .iter()
                    .map(|it| names.get(*it).0.len() + 2)
                    .max()
                    .unwrap_or(0)
            },
        );

        Self {
            main_width: std::cmp::max(30, inv_min_width as i32),
            subsection: SubSection::Inventory,
            inv_selection: 0,
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
        pop_result: &Option<ModeResult>,
    ) -> (ModeControl, ModeUpdate) {
        if let Some(result) = pop_result {
            return match result {
                ModeResult::InventoryActionModeResult(result) => match result {
                    InventoryActionModeResult::AppQuit => (
                        ModeControl::Pop(InventoryModeResult::AppQuit.into()),
                        ModeUpdate::Immediate,
                    ),
                    InventoryActionModeResult::Cancelled => {
                        (ModeControl::Stay, ModeUpdate::WaitForEvent)
                    }
                    InventoryActionModeResult::UseItem(item_id, target) => (
                        ModeControl::Pop(InventoryModeResult::UseItem(*item_id, *target).into()),
                        ModeUpdate::Immediate,
                    ),
                    InventoryActionModeResult::DropItem(item_id) => (
                        ModeControl::Pop(InventoryModeResult::DropItem(*item_id).into()),
                        ModeUpdate::Immediate,
                    ),
                },
                _ => (ModeControl::Stay, ModeUpdate::WaitForEvent),
            };
        }

        inputs.prepare_input();

        if let Some(InputEvent::AppQuit) = inputs.get_input() {
            (
                ModeControl::Pop(InventoryModeResult::AppQuit.into()),
                ModeUpdate::Immediate,
            )
        } else if let Some(InputEvent::Press(keycode)) = inputs.get_input() {
            world.run(
                |player_id: UniqueView<PlayerId>, inventories: View<Inventory>| {
                    let player_inv = inventories.get(player_id.0);

                    match gamekey::from_keycode(keycode, inputs.get_mods(KeyMods::SHIFT)) {
                        GameKey::Down => match self.subsection {
                            SubSection::SortAll => {
                                self.subsection = SubSection::Inventory;
                                self.inv_selection = 0;
                            }
                            SubSection::Inventory => {
                                if !player_inv.items.is_empty()
                                    && self.inv_selection < player_inv.items.len() as i32 - 1
                                {
                                    self.inv_selection += 1;
                                } else {
                                    self.subsection = SubSection::SortAll;
                                }
                            }
                        },
                        GameKey::Up => match self.subsection {
                            SubSection::SortAll => {
                                self.subsection = SubSection::Inventory;
                                self.inv_selection = if player_inv.items.is_empty() {
                                    0
                                } else {
                                    player_inv.items.len() as i32 - 1
                                }
                            }
                            SubSection::Inventory => {
                                if self.inv_selection > 0 {
                                    self.inv_selection -= 1;
                                } else {
                                    self.subsection = SubSection::SortAll;
                                }
                            }
                        },
                        GameKey::Cancel | GameKey::Inventory => {
                            return (
                                ModeControl::Pop(InventoryModeResult::DoNothing.into()),
                                ModeUpdate::Immediate,
                            )
                        }
                        GameKey::Confirm => {
                            match self.subsection {
                                SubSection::SortAll => {} // TODO
                                SubSection::Inventory => {
                                    if !player_inv.items.is_empty() {
                                        inputs.clear_input();
                                        return (
                                            ModeControl::Push(
                                                InventoryActionMode::new(
                                                    world,
                                                    player_inv.items[self.inv_selection as usize],
                                                )
                                                .into(),
                                            ),
                                            ModeUpdate::Immediate,
                                        );
                                    }
                                }
                            }
                        }
                        _ => {}
                    }

                    (ModeControl::Stay, ModeUpdate::WaitForEvent)
                },
            )
        } else {
            (ModeControl::Stay, ModeUpdate::WaitForEvent)
        }
    }

    fn draw_equip(
        &self,
        _world: &World,
        grid: &mut TileGrid<GameSym>,
        fg: Color,
        bg: Color,
        _selected_bg: Color,
    ) {
        // Draw box with bottom edge off-grid.
        grid.draw_box((0, 0), (grid.width(), grid.height() + 1), fg, bg);
        grid.print((2, 0), "< Equipment >");
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
        grid.print((2, 0), "< Inventory >");

        grid.print_color(
            (2, 2),
            "[ Sort all items ]",
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
                        fg,
                        if matches!(self.subsection, SubSection::Inventory) {
                            selected_bg
                        } else {
                            bg
                        },
                    );
                } else {
                    let item_height = (grid.height() as i32 - 6).max(1);
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
