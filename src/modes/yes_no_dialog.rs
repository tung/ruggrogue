use shipyard::World;

use crate::{
    gamekey::{self, GameKey},
    ui,
};
use ruggle::{util::Size, CharGrid, Font, InputBuffer, InputEvent, KeyMods};

use super::{ModeControl, ModeResult, ModeUpdate};

const YES_STR: &str = "[ Yes ]";
const NO_STR: &str = "[ No ]";

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

    pub fn prepare_grids(
        &self,
        _world: &World,
        grids: &mut Vec<CharGrid>,
        font: &Font,
        window_size: Size,
    ) {
        let new_grid_size = Size {
            w: 4 + self.prompt.len().max(YES_STR.len() + NO_STR.len() + 2) as u32,
            h: 7,
        };

        if !grids.is_empty() {
            grids[0].resize(new_grid_size);
        } else {
            grids.push(CharGrid::new(new_grid_size));
            grids[0].view.clear_color = None;
        }

        grids[0].view_centered(font, (0, 0).into(), window_size);
    }

    pub fn update(
        &mut self,
        _world: &World,
        inputs: &mut InputBuffer,
        _pop_result: &Option<ModeResult>,
    ) -> (ModeControl, ModeUpdate) {
        inputs.prepare_input();

        if let Some(InputEvent::Press(keycode)) = inputs.get_input() {
            match gamekey::from_keycode(keycode, inputs.get_mods(KeyMods::SHIFT)) {
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

    pub fn draw(&self, _world: &World, grids: &mut [CharGrid], active: bool) {
        let grid = &mut grids[0];
        let fg = ui::recolor(ui::color::WHITE, active);
        let selected_bg = ui::recolor(ui::color::SELECTED_BG, active);
        let yes_dx = grid.width() as i32 - (YES_STR.len() + NO_STR.len() + 4) as i32;
        let no_dx = grid.width() as i32 - NO_STR.len() as i32 - 2;

        grid.draw_box(
            (0, 0),
            (grid.width(), grid.height()),
            ui::recolor(ui::color::WHITE, active),
            ui::recolor(ui::color::BLACK, active),
        );
        grid.print((2, 2), &self.prompt);

        if self.yes_selected {
            grid.print_color((yes_dx, 4), fg, selected_bg, YES_STR);
            grid.print_color((no_dx, 4), fg, None, NO_STR);
        } else {
            grid.print_color((yes_dx, 4), fg, None, YES_STR);
            grid.print_color((no_dx, 4), fg, selected_bg, NO_STR);
        }
    }
}
