//! The mode stack is the central coordinator of the parts of the game that the player sees and
//! interacts with while playing the game.  The game parts, or "modes", are the high-level building
//! blocks of the game, e.g. title screen, main gameplay screen, inventory menu, dialog boxes, etc.
//! By placing and changing these modes on the "mode stack", the top-most mode has its update logic
//! performed, while all the modes on the stack perform their drawing from the bottom-up.
//!
//! To add a new mode, create a mode struct and add it to the [Mode] enum.  Create a corresponding
//! result enum and add it to the [ModeResult] enum, the purpose of which is to return any data to
//! other modes when the mode itself is popped from the stack.  Implementing [From] for these
//! enables the use of the handy [Into::into] method for convenience elsewhere.
//!
//! The new mode struct should implement `update` and `draw` methods; add the matching calls to the
//! Mode impl to dispatch to them.
//!
//! `update` for the new mode should perform update logic and return a two-tuple of [ModeControl]
//! and [ModeUpdate].  The [ModeControl] controls stack manipulation which should most often be
//! [ModeControl::Stay] to keep the stack as-is.  When using [ModeControl::Pop], return it with an
//! instance of your result enum.  Meanwhile, the [ModeUpdate] determines how the next `update`
//! should be handled; see its documentation for the possible values and their effects.
//!
//! `draw` for the new mode should draw whatever the mode wants to show.  Modes underneath this
//! mode on the stack will be drawn before this one, while modes on top will be drawn afterwards,
//! so it's possible to have e.g. an inventory menu mode draw itself smaller than the screen, so
//! the main gameplay mode underneath can be seen behind it.

pub mod dungeon;
pub mod yes_no_dialog;

use shipyard::World;

use ruggle::{CharGrid, InputBuffer, RunControl};

use dungeon::{DungeonMode, DungeonModeResult};
use yes_no_dialog::{YesNoDialogMode, YesNoDialogModeResult};

// /////////////////////////////////////////////////////////////////////////////

/// All possible modes that can be added to the mode stack.  Add new modes here.
pub enum Mode {
    DungeonMode(DungeonMode),
    YesNoDialogMode(YesNoDialogMode),
}

impl From<DungeonMode> for Mode {
    fn from(mode: DungeonMode) -> Self {
        Self::DungeonMode(mode)
    }
}

impl From<YesNoDialogMode> for Mode {
    fn from(mode: YesNoDialogMode) -> Self {
        Self::YesNoDialogMode(mode)
    }
}

// /////////////////////////////////////////////////////////////////////////////

/// All possible mode results that each mode can return when removed from the mode stack.  A result
/// should be added for every mode added.
pub enum ModeResult {
    DungeonModeResult(DungeonModeResult),
    YesNoDialogModeResult(YesNoDialogModeResult),
}

impl From<DungeonModeResult> for ModeResult {
    fn from(result: DungeonModeResult) -> Self {
        Self::DungeonModeResult(result)
    }
}

impl From<YesNoDialogModeResult> for ModeResult {
    fn from(result: YesNoDialogModeResult) -> Self {
        Self::YesNoDialogModeResult(result)
    }
}

// /////////////////////////////////////////////////////////////////////////////

/// Mode stack manipulation values to be returned from an `update` call.
#[allow(dead_code)]
pub enum ModeControl {
    /// Keep the stack as-is.
    Stay,
    /// Replace the current mode on the stack with a new mode.
    Switch(Mode),
    /// Push a new mode on top of the current mode on the stack.
    Push(Mode),
    /// Pop the current mode from the stack, with a corresponding result.
    Pop(ModeResult),
    /// Clear the whole stack, while returning a corresponding result.
    Terminate(ModeResult),
}

/// Desired behavior for the next update, to be returned from an `update` call.
#[allow(dead_code)]
pub enum ModeUpdate {
    /// Run the next update immediately, without waiting for the next frame.
    Immediate,
    /// Wait a frame before the next update; this will likely draw the mode for a frame.
    Update,
    /// Wait for an input event before the next update; this will likely draw the mode before
    /// waiting.
    WaitForEvent,
}

/// Mode method dispatcher.  Add `update` and `draw` calls for new modes here.
impl Mode {
    pub fn update(
        &mut self,
        world: &World,
        inputs: &mut InputBuffer,
        pop_result: &Option<ModeResult>,
    ) -> (ModeControl, ModeUpdate) {
        match self {
            Mode::DungeonMode(x) => x.update(world, inputs, pop_result),
            Mode::YesNoDialogMode(x) => x.update(world, inputs, pop_result),
        }
    }

    pub fn draw(&self, world: &World, grid: &mut CharGrid) {
        match self {
            Mode::DungeonMode(x) => x.draw(world, grid),
            Mode::YesNoDialogMode(x) => x.draw(world, grid),
        }
    }
}

// /////////////////////////////////////////////////////////////////////////////

/// The mode stack proper.  Create one of these with an initial mode, then call [ModeStack::update]
/// and [ModeStack::draw] at the appropriate points in the surrounding code; the mode stack and the
/// modes it holds will handle everything else.
pub struct ModeStack {
    stack: Vec<Mode>,
    pop_result: Option<ModeResult>,
}

impl ModeStack {
    /// Create a new mode stack.
    pub fn new(stack: Vec<Mode>) -> Self {
        Self {
            stack,
            pop_result: None,
        }
    }

    /// Perform update logic for the top-most mode of the stack.  This also converts [ModeUpdate]
    /// values into [ruggle::RunControl] values to control the behavior of the next update.
    pub fn update(&mut self, world: &World, inputs: &mut InputBuffer) -> RunControl {
        while let Some(top_mode) = self.stack.last_mut() {
            let result = top_mode.update(world, inputs, &self.pop_result);

            self.pop_result = None;

            match result.0 {
                ModeControl::Stay => {}
                ModeControl::Switch(mode) => {
                    self.stack.pop();
                    self.stack.push(mode);
                }
                ModeControl::Push(mode) => {
                    self.stack.push(mode);
                }
                ModeControl::Pop(mode_result) => {
                    self.pop_result = Some(mode_result);
                    self.stack.pop();
                }
                ModeControl::Terminate(mode_result) => {
                    self.pop_result = Some(mode_result);
                    self.stack.clear();
                }
            }

            match result.1 {
                ModeUpdate::Immediate => (),
                ModeUpdate::Update => return RunControl::Update,
                ModeUpdate::WaitForEvent => return RunControl::WaitForEvent,
            }
        }

        RunControl::Quit
    }

    /// Draw the modes in the stack from the bottom-up.
    pub fn draw(&self, world: &World, grid: &mut CharGrid) {
        let stack_size = self.stack.len();

        if stack_size == 0 {
            return;
        }

        for i in 0..(stack_size - 1) {
            self.stack[i].draw(world, grid);
        }

        if let Some(top_mode) = self.stack.last() {
            top_mode.draw(world, grid);
        }
    }
}
