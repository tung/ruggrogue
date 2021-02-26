use piston::input::Button;
use shipyard::{EntityId, Get, View, World};

use crate::{
    components::{AreaOfEffect, Name, Ranged, Renderable},
    gamekey::GameKey,
    ui,
};
use ruggle::{CharGrid, InputBuffer, InputEvent};

use super::{
    target::{TargetMode, TargetModeResult},
    ModeControl, ModeResult, ModeUpdate,
};

const CANCEL: &str = "[ Cancel ]";

pub enum InventoryActionModeResult {
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

    pub fn update(
        &mut self,
        world: &World,
        inputs: &mut InputBuffer,
        pop_result: &Option<ModeResult>,
    ) -> (ModeControl, ModeUpdate) {
        if let Some(result) = pop_result {
            return match result {
                ModeResult::TargetModeResult(result) => match result {
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

        if let Some(InputEvent::Press(Button::Keyboard(key))) = inputs.get_input() {
            match key.into() {
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

    pub fn draw(&self, world: &World, grid: &mut CharGrid, active: bool) {
        let width = self.inner_width + 4;
        let height = self.actions.len() as i32 + 8;
        let x = (grid.size_cells()[0] - width) / 2;
        let y = (grid.size_cells()[1] - height) / 2;
        let fg = ui::recolor(ui::color::WHITE, active);
        let bg = ui::recolor(ui::color::BLACK, active);

        grid.draw_box([x, y], [width, height], fg, bg);

        world.run(|names: View<Name>, renderables: View<Renderable>| {
            let render = renderables.get(self.item_id);

            grid.put_color(
                [x + 2, y + 2],
                Some(ui::recolor(render.fg, active)),
                Some(ui::recolor(render.bg, active)),
                render.ch,
            );
            grid.print_color([x + 4, y + 2], Some(fg), None, &names.get(self.item_id).0);
        });

        for (i, action) in self.actions.iter().enumerate() {
            let action_bg = Some(ui::recolor(
                if matches!(self.subsection, SubSection::Actions) && i as i32 == self.selection {
                    ui::color::SELECTED_BG
                } else {
                    ui::color::BLACK
                },
                active,
            ));

            grid.print_color(
                [x + 4, y + 4 + i as i32],
                Some(fg),
                action_bg,
                action.name(),
            );
        }

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
