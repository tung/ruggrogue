use shipyard::{EntityId, Get, UniqueView, UniqueViewMut, View, World};

use crate::{
    components::{Item, Name, Position, Renderable},
    gamekey::{self, GameKey},
    map::Map,
    message::Messages,
    player::PlayerId,
    ui,
};
use ruggle::{CharGrid, InputBuffer, InputEvent, KeyMods};

use super::{ModeControl, ModeResult, ModeUpdate};

const CANCEL: &str = "[ Cancel ]";
const PROMPT: &str = "Pick up which item?";

pub enum PickUpMenuModeResult {
    PickedItem(EntityId),
    Cancelled,
}

enum SubSection {
    Items,
    Cancel,
}

pub struct PickUpMenuMode {
    items: Vec<EntityId>,
    width: i32,
    subsection: SubSection,
    selection: i32,
}

/// Show a list of items that player is on top of and let them choose one to pick up.
impl PickUpMenuMode {
    pub fn new(world: &World) -> Self {
        let (items, width) = world.run(
            |map: UniqueView<Map>,
             player_id: UniqueView<PlayerId>,
             items: View<Item>,
             names: View<Name>,
             positions: View<Position>| {
                let player_pos = positions.get(player_id.0);
                let items = map
                    .iter_entities_at(player_pos.x, player_pos.y)
                    .filter(|id| items.contains(*id))
                    .collect::<Vec<_>>();
                let width = std::cmp::max(
                    PROMPT.len(),
                    std::cmp::max(
                        CANCEL.len(),
                        items
                            .iter()
                            .map(|it| names.get(*it).0.len() + 2)
                            .max()
                            .unwrap_or(2),
                    ),
                );

                (items, width)
            },
        );

        Self {
            items,
            width: width as i32,
            subsection: SubSection::Items,
            selection: 0,
        }
    }

    pub fn update(
        &mut self,
        world: &World,
        inputs: &mut InputBuffer,
        _pop_result: &Option<ModeResult>,
    ) -> (ModeControl, ModeUpdate) {
        if self.items.is_empty() {
            world.run(|mut msgs: UniqueViewMut<Messages>| {
                msgs.add("There are no items to pick up here.".into());
            });

            (
                ModeControl::Pop(PickUpMenuModeResult::Cancelled.into()),
                ModeUpdate::Immediate,
            )
        } else {
            inputs.prepare_input();

            if let Some(InputEvent::Press(keycode)) = inputs.get_input() {
                match gamekey::from_keycode(keycode, inputs.get_mods(KeyMods::SHIFT)) {
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
                    GameKey::Cancel => {
                        return (
                            ModeControl::Pop(PickUpMenuModeResult::Cancelled.into()),
                            ModeUpdate::Immediate,
                        )
                    }
                    GameKey::Confirm | GameKey::PickUp => {
                        let result = match self.subsection {
                            SubSection::Items => PickUpMenuModeResult::PickedItem(
                                self.items[self.selection as usize],
                            ),
                            SubSection::Cancel => PickUpMenuModeResult::Cancelled,
                        };

                        return (ModeControl::Pop(result.into()), ModeUpdate::Immediate);
                    }
                    _ => {}
                }
            }

            (ModeControl::Stay, ModeUpdate::WaitForEvent)
        }
    }

    pub fn draw(&self, world: &World, grid: &mut CharGrid, active: bool) {
        let width = self.width + 4;
        let height = std::cmp::max(
            9,
            std::cmp::min(grid.size_cells()[1], self.items.len() as i32 + 8),
        );
        let x = (grid.size_cells()[0] - width) / 2;
        let y = (grid.size_cells()[1] - height) / 2;
        let fg = ui::recolor(ui::color::WHITE, active);
        let bg = ui::recolor(ui::color::BLACK, active);

        grid.draw_box([x, y], [width, height], fg, bg);
        grid.print_color([x + 2, y + 2], Some(fg), None, PROMPT);

        let list_height = height - 8;
        let list_offset = std::cmp::max(
            0,
            std::cmp::min(
                self.items.len() as i32 - list_height,
                self.selection - (list_height - 1) / 2,
            ),
        );

        if self.items.len() as i32 > list_height {
            grid.draw_bar(
                true,
                [x + width - 1, y + 4],
                list_height,
                list_offset,
                list_height,
                self.items.len() as i32,
                Some(fg),
                Some(bg),
            );
        }

        world.run(|names: View<Name>, renderables: View<Renderable>| {
            for (i, item_id) in self
                .items
                .iter()
                .enumerate()
                .skip(list_offset as usize)
                .take(list_height as usize)
            {
                let render = renderables.get(*item_id);
                let bg = Some(ui::recolor(
                    if matches!(self.subsection, SubSection::Items) && i as i32 == self.selection {
                        ui::color::SELECTED_BG
                    } else {
                        ui::color::BLACK
                    },
                    active,
                ));

                grid.put_color(
                    [x + 2, y + 4 + i as i32 - list_offset],
                    Some(ui::recolor(render.fg, active)),
                    Some(ui::recolor(render.bg, active)),
                    render.ch,
                );
                grid.print_color(
                    [x + 4, y + 4 + i as i32 - list_offset],
                    Some(fg),
                    bg,
                    &names.get(*item_id).0,
                );
            }
        });

        let cancel_bg = Some(ui::recolor(
            if matches!(self.subsection, SubSection::Cancel) {
                ui::color::SELECTED_BG
            } else {
                ui::color::BLACK
            },
            active,
        ));

        grid.print_color([x + 4, y + height - 3], Some(fg), cancel_bg, CANCEL);
    }
}
