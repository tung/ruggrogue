use piston::input::{Button, Key};
use shipyard::{EntityId, Get, UniqueView, View, World};

use crate::{
    components::{Inventory, Name, Renderable},
    player::PlayerId,
    ui,
};
use ruggle::{CharGrid, InputBuffer, InputEvent};

use super::{
    inventory_action::{InventoryActionMode, InventoryActionModeResult},
    ModeControl, ModeResult, ModeUpdate,
};

pub enum InventoryModeResult {
    DoNothing,
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

    pub fn update(
        &mut self,
        world: &World,
        inputs: &mut InputBuffer,
        pop_result: &Option<ModeResult>,
    ) -> (ModeControl, ModeUpdate) {
        if let Some(result) = pop_result {
            return match result {
                ModeResult::InventoryActionModeResult(result) => match result {
                    InventoryActionModeResult::Cancelled => {
                        (ModeControl::Stay, ModeUpdate::WaitForEvent)
                    }
                    InventoryActionModeResult::DropItem(item_id) => (
                        ModeControl::Pop(InventoryModeResult::DropItem(*item_id).into()),
                        ModeUpdate::Immediate,
                    ),
                },
                _ => (ModeControl::Stay, ModeUpdate::WaitForEvent),
            };
        }

        inputs.prepare_input();

        if let Some(InputEvent::Press(Button::Keyboard(key))) = inputs.get_input() {
            world.run(
                |player_id: UniqueView<PlayerId>, inventories: View<Inventory>| {
                    let player_inv = inventories.get(player_id.0);

                    match key {
                        Key::J | Key::NumPad2 | Key::Down => match self.subsection {
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
                        Key::K | Key::NumPad8 | Key::Up => match self.subsection {
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
                        Key::Escape => {
                            return (
                                ModeControl::Pop(InventoryModeResult::DoNothing.into()),
                                ModeUpdate::Immediate,
                            )
                        }
                        Key::Return => {
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

    fn draw_inventory(
        &self,
        world: &World,
        grid: &mut CharGrid,
        active: bool,
        [x, y]: [i32; 2],
        [width, height]: [i32; 2],
    ) {
        let fg = Some(ui::recolor(ui::color::WHITE, active));
        let bg = Some(ui::recolor(ui::color::BLACK, active));
        let selected_bg = Some(ui::recolor(ui::color::SELECTED_BG, active));

        grid.print_color([x + 2, y], fg, bg, "< Inventory >");

        grid.print_color(
            [x + 2, y + 2],
            fg,
            if matches!(self.subsection, SubSection::SortAll) {
                selected_bg
            } else {
                bg
            },
            "[ Sort all items ]",
        );

        world.run(
            |player_id: UniqueView<PlayerId>,
             inventories: View<Inventory>,
             names: View<Name>,
             renderables: View<Renderable>| {
                let player_inv = inventories.get(player_id.0);
                let item_x = x + 2;
                let item_y = y + 4;

                if player_inv.items.is_empty() {
                    grid.print_color(
                        [item_x, item_y],
                        fg,
                        if matches!(self.subsection, SubSection::Inventory) {
                            selected_bg
                        } else {
                            bg
                        },
                        "-- nothing --",
                    );
                } else {
                    let item_height = std::cmp::max(1, height - 4 - 2);
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
                            [x + width - 1, item_y],
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

                        grid.put_color(
                            [item_x, item_y + i as i32 - item_offset],
                            Some(ui::recolor(render.fg, active)),
                            Some(ui::recolor(render.bg, active)),
                            render.ch,
                        );
                        grid.print_color(
                            [item_x + 2, item_y + i as i32 - item_offset],
                            fg,
                            if matches!(self.subsection, SubSection::Inventory)
                                && i as i32 == self.inv_selection
                            {
                                selected_bg
                            } else {
                                bg
                            },
                            &names.get(*item_id).0,
                        );
                    }
                }
            },
        );
    }

    pub fn draw(&self, world: &World, grid: &mut CharGrid, active: bool) {
        const SIDE_WIDTH: i32 = 15;
        const TOP_HEIGHT: i32 = 2;
        const ESC_TO_CLOSE: &str = " [Esc] to close ";

        let inv_len = world.run(
            |player_id: UniqueView<PlayerId>, inventories: View<Inventory>| {
                inventories.get(player_id.0).items.len() as i32
            },
        );
        let inv_height = std::cmp::min(
            grid.size_cells()[1] - (2 + TOP_HEIGHT + 3 + 1 + 1 + 2),
            std::cmp::max(13, inv_len),
        );
        let full_width = 2 + SIDE_WIDTH + 3 + self.main_width + 2;
        let full_height = 2 + TOP_HEIGHT + 3 + 1 + 1 + inv_height + 2;
        let base_x = (grid.size_cells()[0] - full_width) / 2;
        let base_y = (grid.size_cells()[1] - full_height) / 2;
        let fg = ui::recolor(ui::color::WHITE, active);
        let bg = ui::recolor(ui::color::BLACK, active);
        let equip_x = base_x + SIDE_WIDTH;
        let inv_x = equip_x;
        let inv_y = base_y + 2 + TOP_HEIGHT + 1;

        // Full box border.
        grid.draw_box([base_x, base_y], [full_width, full_height], fg, bg);

        let fg = Some(fg);
        let bg = Some(bg);

        // Side bar vertical divider.
        grid.put_color([equip_x, base_y], fg, bg, '┬');
        for y in (base_y + 1)..(base_y + full_height - 1) {
            grid.put_color([equip_x, y], fg, bg, '│');
        }
        grid.put_color([equip_x, base_y + full_height - 1], fg, bg, '┴');

        // Equipment/inventory horizontal divider.
        grid.put_color([inv_x, inv_y], fg, bg, '├');
        for x in (inv_x + 1)..(base_x + full_width - 1) {
            grid.put_color([x, inv_y], fg, bg, '─');
        }
        grid.put_color([base_x + full_width - 1, inv_y], fg, bg, '┤');

        grid.print_color(
            [
                base_x + full_width - ESC_TO_CLOSE.len() as i32 - 2,
                base_y + full_height - 1,
            ],
            fg,
            bg,
            ESC_TO_CLOSE,
        );

        self.draw_inventory(
            world,
            grid,
            active,
            [inv_x, inv_y],
            [full_width - SIDE_WIDTH, full_height - (2 + TOP_HEIGHT + 1)],
        );
    }
}
