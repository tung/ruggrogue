use shipyard::World;

use ruggle::{CharGrid, InputBuffer, InputEvent};

use super::{ModeControl, ModeResult, ModeUpdate};
use crate::{gamekey::GameKey, ui};

pub enum YesNoDialogModeResult {
    Yes,
    No,
}

pub struct YesNoDialogMode {
    prompt: String,
    yes_selected: bool,
}

impl From<bool> for YesNoDialogModeResult {
    fn from(yes: bool) -> Self {
        if yes {
            Self::Yes
        } else {
            Self::No
        }
    }
}

/// A yes-or-no dialog box with a prompt that shows up in the center of the screen.
impl YesNoDialogMode {
    pub fn new(prompt: String, yes_default: bool) -> Self {
        Self {
            prompt,
            yes_selected: yes_default,
        }
    }

    pub fn update(
        &mut self,
        _world: &World,
        inputs: &mut InputBuffer,
        _pop_result: &Option<ModeResult>,
    ) -> (ModeControl, ModeUpdate) {
        inputs.prepare_input();

        if let Some(InputEvent::Press(keycode)) = inputs.get_input() {
            match keycode.into() {
                GameKey::Left => self.yes_selected = true,
                GameKey::Right => self.yes_selected = false,
                GameKey::Confirm => {
                    return (
                        ModeControl::Pop(YesNoDialogModeResult::from(self.yes_selected).into()),
                        ModeUpdate::Immediate,
                    )
                }
                GameKey::Cancel => {
                    return (
                        ModeControl::Pop(YesNoDialogModeResult::No.into()),
                        ModeUpdate::Immediate,
                    )
                }
                _ => {}
            }
        }

        (ModeControl::Stay, ModeUpdate::WaitForEvent)
    }

    pub fn draw(&self, _world: &World, grid: &mut CharGrid, active: bool) {
        let yes_str = "[ Yes ]";
        let no_str = "[ No ]";
        let fg = Some(ui::recolor(ui::color::WHITE, active));
        let selected_bg = Some(ui::recolor(ui::color::SELECTED_BG, active));
        let width = std::cmp::max(self.prompt.len(), yes_str.len() + no_str.len() + 2) as i32 + 4;
        let height = 7i32;
        let x = (grid.size_cells()[0] - width) / 2;
        let y = (grid.size_cells()[1] - height) / 2;
        let yes_dx = width - yes_str.len() as i32 - no_str.len() as i32 - 4;
        let no_dx = width - no_str.len() as i32 - 2;

        grid.draw_box(
            [x, y],
            [width, height],
            ui::recolor(ui::color::WHITE, active),
            ui::recolor(ui::color::BLACK, active),
        );
        grid.print([x + 2, y + 2], &self.prompt);

        if self.yes_selected {
            grid.print_color([x + yes_dx, y + 4], fg, selected_bg, yes_str);
            grid.print_color([x + no_dx, y + 4], fg, None, no_str);
        } else {
            grid.print_color([x + yes_dx, y + 4], fg, None, yes_str);
            grid.print_color([x + no_dx, y + 4], fg, selected_bg, no_str);
        }
    }
}
