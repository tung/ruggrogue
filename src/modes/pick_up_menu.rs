use shipyard::{EntityId, Get, UniqueView, UniqueViewMut, View, World};

use crate::{
    components::{Coord, Item, Name, Renderable},
    gamekey::{self, GameKey},
    gamesym::GameSym,
    map::Map,
    message::Messages,
    player::PlayerId,
    ui::{self, Options},
};
use ruggle::{
    util::{Color, Size},
    InputBuffer, InputEvent, KeyMods, TileGrid, Tileset,
};

use super::{ModeControl, ModeResult, ModeUpdate};

const CANCEL: &str = "[ Cancel ]";
const PROMPT: &str = "Pick up which item?";

pub enum PickUpMenuModeResult {
    AppQuit,
    Cancelled,
    PickedItem(EntityId),
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
             coords: View<Coord>,
             items: View<Item>,
             names: View<Name>| {
                let player_coord = coords.get(player_id.0);
                let items = map
                    .iter_entities_at(player_coord.0.x, player_coord.0.y)
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
            w: self.width as u32 + 4,
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

            if let Some(InputEvent::AppQuit) = inputs.get_input() {
                return (
                    ModeControl::Pop(PickUpMenuModeResult::AppQuit.into()),
                    ModeUpdate::Immediate,
                );
            } else if let Some(InputEvent::Press(keycode)) = inputs.get_input() {
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

    pub fn draw(&self, world: &World, grids: &mut [TileGrid<GameSym>], active: bool) {
        let grid = &mut grids[0];
        let width = grid.width();
        let height = grid.height();
        let fg = Color::WHITE;
        let bg = Color::BLACK;
        let selected_bg = ui::SELECTED_BG;

        grid.view.color_mod = if active { Color::WHITE } else { Color::GRAY };

        grid.draw_box((0, 0), (width, height), fg, bg);
        grid.print((2, 2), PROMPT);

        let list_height = height as i32 - 8;
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
                (width as i32 - 1, 4),
                list_height,
                list_offset,
                list_height,
                self.items.len() as i32,
                fg,
                bg,
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

                grid.put_sym_color(
                    (2, 4 + i as i32 - list_offset),
                    render.sym,
                    render.fg,
                    render.bg,
                );

                grid.print_color(
                    (4, 4 + i as i32 - list_offset),
                    &names.get(*item_id).0,
                    fg,
                    if matches!(self.subsection, SubSection::Items) && i as i32 == self.selection {
                        selected_bg
                    } else {
                        bg
                    },
                );
            }
        });

        grid.print_color(
            (4, height as i32 - 3),
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
