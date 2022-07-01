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

pub mod app_quit_dialog;
pub mod dungeon;
pub mod equipment_action;
pub mod equipment_shortcut;
pub mod game_over;
pub mod inventory;
pub mod inventory_action;
pub mod inventory_shortcut;
pub mod message_box;
pub mod options_menu;
pub mod pick_up_menu;
pub mod target;
pub mod title;
pub mod view_map;
pub mod yes_no_dialog;

use shipyard::World;

use crate::gamesym::GameSym;
use ruggrogue::{util::Size, InputBuffer, RunControl, TileGrid, TileGridLayer, Tileset};

use app_quit_dialog::{AppQuitDialogMode, AppQuitDialogModeResult};
use dungeon::{DungeonMode, DungeonModeResult};
use equipment_action::{EquipmentActionMode, EquipmentActionModeResult};
use equipment_shortcut::{EquipmentShortcutMode, EquipmentShortcutModeResult};
use game_over::{GameOverMode, GameOverModeResult};
use inventory::{InventoryMode, InventoryModeResult};
use inventory_action::{InventoryActionMode, InventoryActionModeResult};
use inventory_shortcut::{InventoryShortcutMode, InventoryShortcutModeResult};
use message_box::{MessageBoxMode, MessageBoxModeResult};
use options_menu::{OptionsMenuMode, OptionsMenuModeResult};
use pick_up_menu::{PickUpMenuMode, PickUpMenuModeResult};
use target::{TargetMode, TargetModeResult};
use title::{TitleMode, TitleModeResult};
use view_map::{ViewMapMode, ViewMapModeResult};
use yes_no_dialog::{YesNoDialogMode, YesNoDialogModeResult};

// /////////////////////////////////////////////////////////////////////////////

/// Helper macro to convert a type into an enum variant with the same name.
macro_rules! impl_from {
    ($to:ty, $from:ident) => {
        impl From<$from> for $to {
            fn from(f: $from) -> Self {
                Self::$from(f)
            }
        }
    };
}

/// All possible modes that can be added to the mode stack.  Add new modes here.
#[allow(clippy::enum_variant_names)]
pub enum Mode {
    AppQuitDialogMode(AppQuitDialogMode),
    DungeonMode(DungeonMode),
    EquipmentActionMode(EquipmentActionMode),
    EquipmentShortcutMode(EquipmentShortcutMode),
    GameOverMode(GameOverMode),
    InventoryMode(InventoryMode),
    InventoryActionMode(InventoryActionMode),
    InventoryShortcutMode(InventoryShortcutMode),
    MessageBoxMode(MessageBoxMode),
    OptionsMenuMode(OptionsMenuMode),
    PickUpMenuMode(PickUpMenuMode),
    TargetMode(TargetMode),
    TitleMode(TitleMode),
    ViewMapMode(ViewMapMode),
    YesNoDialogMode(YesNoDialogMode),
}

impl_from!(Mode, AppQuitDialogMode);
impl_from!(Mode, DungeonMode);
impl_from!(Mode, EquipmentActionMode);
impl_from!(Mode, EquipmentShortcutMode);
impl_from!(Mode, GameOverMode);
impl_from!(Mode, InventoryMode);
impl_from!(Mode, InventoryActionMode);
impl_from!(Mode, InventoryShortcutMode);
impl_from!(Mode, MessageBoxMode);
impl_from!(Mode, OptionsMenuMode);
impl_from!(Mode, PickUpMenuMode);
impl_from!(Mode, TargetMode);
impl_from!(Mode, TitleMode);
impl_from!(Mode, ViewMapMode);
impl_from!(Mode, YesNoDialogMode);

// /////////////////////////////////////////////////////////////////////////////

/// All possible mode results that each mode can return when removed from the mode stack.  A result
/// should be added for every mode added.
#[allow(clippy::enum_variant_names)]
pub enum ModeResult {
    AppQuitDialogModeResult(AppQuitDialogModeResult),
    DungeonModeResult(DungeonModeResult),
    EquipmentActionModeResult(EquipmentActionModeResult),
    EquipmentShortcutModeResult(EquipmentShortcutModeResult),
    GameOverModeResult(GameOverModeResult),
    InventoryModeResult(InventoryModeResult),
    InventoryActionModeResult(InventoryActionModeResult),
    InventoryShortcutModeResult(InventoryShortcutModeResult),
    MessageBoxModeResult(MessageBoxModeResult),
    OptionsMenuModeResult(OptionsMenuModeResult),
    PickUpMenuModeResult(PickUpMenuModeResult),
    TargetModeResult(TargetModeResult),
    TitleModeResult(TitleModeResult),
    ViewMapModeResult(ViewMapModeResult),
    YesNoDialogModeResult(YesNoDialogModeResult),
}

impl_from!(ModeResult, AppQuitDialogModeResult);
impl_from!(ModeResult, DungeonModeResult);
impl_from!(ModeResult, EquipmentActionModeResult);
impl_from!(ModeResult, EquipmentShortcutModeResult);
impl_from!(ModeResult, GameOverModeResult);
impl_from!(ModeResult, InventoryModeResult);
impl_from!(ModeResult, InventoryActionModeResult);
impl_from!(ModeResult, InventoryShortcutModeResult);
impl_from!(ModeResult, MessageBoxModeResult);
impl_from!(ModeResult, OptionsMenuModeResult);
impl_from!(ModeResult, PickUpMenuModeResult);
impl_from!(ModeResult, TargetModeResult);
impl_from!(ModeResult, TitleModeResult);
impl_from!(ModeResult, ViewMapModeResult);
impl_from!(ModeResult, YesNoDialogModeResult);

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

/// Mode method dispatcher.  Add `prepare_grids`, `update` and `draw` calls for new modes here.
impl Mode {
    fn prepare_grids(
        &mut self,
        world: &World,
        grids: &mut Vec<TileGrid<GameSym>>,
        tilesets: &[Tileset<GameSym>],
        window_size: Size,
    ) {
        match self {
            Mode::AppQuitDialogMode(x) => x.prepare_grids(world, grids, tilesets, window_size),
            Mode::DungeonMode(x) => x.prepare_grids(world, grids, tilesets, window_size),
            Mode::EquipmentActionMode(x) => x.prepare_grids(world, grids, tilesets, window_size),
            Mode::EquipmentShortcutMode(x) => x.prepare_grids(world, grids, tilesets, window_size),
            Mode::GameOverMode(x) => x.prepare_grids(world, grids, tilesets, window_size),
            Mode::InventoryMode(x) => x.prepare_grids(world, grids, tilesets, window_size),
            Mode::InventoryActionMode(x) => x.prepare_grids(world, grids, tilesets, window_size),
            Mode::InventoryShortcutMode(x) => x.prepare_grids(world, grids, tilesets, window_size),
            Mode::MessageBoxMode(x) => x.prepare_grids(world, grids, tilesets, window_size),
            Mode::OptionsMenuMode(x) => x.prepare_grids(world, grids, tilesets, window_size),
            Mode::PickUpMenuMode(x) => x.prepare_grids(world, grids, tilesets, window_size),
            Mode::TargetMode(x) => x.prepare_grids(world, grids, tilesets, window_size),
            Mode::TitleMode(x) => x.prepare_grids(world, grids, tilesets, window_size),
            Mode::ViewMapMode(x) => x.prepare_grids(world, grids, tilesets, window_size),
            Mode::YesNoDialogMode(x) => x.prepare_grids(world, grids, tilesets, window_size),
        }
    }

    fn update(
        &mut self,
        world: &World,
        inputs: &mut InputBuffer,
        grids: &[TileGrid<GameSym>],
        pop_result: &Option<ModeResult>,
    ) -> (ModeControl, ModeUpdate) {
        match self {
            Mode::AppQuitDialogMode(x) => x.update(world, inputs, grids, pop_result),
            Mode::DungeonMode(x) => x.update(world, inputs, grids, pop_result),
            Mode::EquipmentActionMode(x) => x.update(world, inputs, grids, pop_result),
            Mode::EquipmentShortcutMode(x) => x.update(world, inputs, grids, pop_result),
            Mode::GameOverMode(x) => x.update(world, inputs, grids, pop_result),
            Mode::InventoryMode(x) => x.update(world, inputs, grids, pop_result),
            Mode::InventoryActionMode(x) => x.update(world, inputs, grids, pop_result),
            Mode::InventoryShortcutMode(x) => x.update(world, inputs, grids, pop_result),
            Mode::MessageBoxMode(x) => x.update(world, inputs, grids, pop_result),
            Mode::OptionsMenuMode(x) => x.update(world, inputs, grids, pop_result),
            Mode::PickUpMenuMode(x) => x.update(world, inputs, grids, pop_result),
            Mode::TargetMode(x) => x.update(world, inputs, grids, pop_result),
            Mode::TitleMode(x) => x.update(world, inputs, grids, pop_result),
            Mode::ViewMapMode(x) => x.update(world, inputs, grids, pop_result),
            Mode::YesNoDialogMode(x) => x.update(world, inputs, grids, pop_result),
        }
    }

    fn draw(&mut self, world: &World, grids: &mut [TileGrid<GameSym>], active: bool) {
        match self {
            Mode::AppQuitDialogMode(x) => x.draw(world, grids, active),
            Mode::DungeonMode(x) => x.draw(world, grids, active),
            Mode::EquipmentActionMode(x) => x.draw(world, grids, active),
            Mode::EquipmentShortcutMode(x) => x.draw(world, grids, active),
            Mode::GameOverMode(x) => x.draw(world, grids, active),
            Mode::InventoryMode(x) => x.draw(world, grids, active),
            Mode::InventoryActionMode(x) => x.draw(world, grids, active),
            Mode::InventoryShortcutMode(x) => x.draw(world, grids, active),
            Mode::MessageBoxMode(x) => x.draw(world, grids, active),
            Mode::OptionsMenuMode(x) => x.draw(world, grids, active),
            Mode::PickUpMenuMode(x) => x.draw(world, grids, active),
            Mode::TargetMode(x) => x.draw(world, grids, active),
            Mode::TitleMode(x) => x.draw(world, grids, active),
            Mode::ViewMapMode(x) => x.draw(world, grids, active),
            Mode::YesNoDialogMode(x) => x.draw(world, grids, active),
        }
    }

    /// Should the current mode draw modes behind it in the stack?
    fn draw_behind(&self) -> bool {
        match self {
            Mode::AppQuitDialogMode(_) => true,
            Mode::DungeonMode(_) => false,
            Mode::EquipmentActionMode(_) => true,
            Mode::EquipmentShortcutMode(_) => true,
            Mode::GameOverMode(_) => false,
            Mode::InventoryMode(_) => true,
            Mode::InventoryActionMode(_) => true,
            Mode::InventoryShortcutMode(_) => true,
            Mode::MessageBoxMode(_) => true,
            Mode::OptionsMenuMode(_) => true,
            Mode::PickUpMenuMode(_) => true,
            Mode::TargetMode(_) => false,
            Mode::TitleMode(_) => false,
            Mode::ViewMapMode(_) => false,
            Mode::YesNoDialogMode(_) => true,
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

    /// Perform update logic for the top mode of the stack, and then drawing logic for all  modes.
    ///
    /// This also converts [ModeUpdate] values into [ruggrogue::RunControl] values to control the
    /// behavior of the next update.
    pub fn update(
        &mut self,
        world: &World,
        inputs: &mut InputBuffer,
        layers: &mut Vec<TileGridLayer<GameSym>>,
        tilesets: &[Tileset<GameSym>],
        window_size: Size,
    ) -> RunControl {
        if !self.stack.is_empty() && layers.is_empty() {
            // Initialize a layer for each mode in the stack.
            // There will always be a layer for each mode, even if it doesn't use it.
            for mode in &self.stack {
                layers.push(TileGridLayer {
                    draw_behind: mode.draw_behind(),
                    grids: Vec::new(),
                });
            }
        }

        while !self.stack.is_empty() {
            // Prepare grids for modes, starting from the lowest visible mode.
            let prepare_grids_from = self
                .stack
                .iter()
                .rposition(|mode| !mode.draw_behind())
                .unwrap_or(0);

            for (i, mode) in self.stack.iter_mut().enumerate().skip(prepare_grids_from) {
                mode.prepare_grids(world, &mut layers[i].grids, tilesets, window_size);
            }

            // Update the top mode.
            let (mode_control, mode_update) = {
                let top_mode = self.stack.last_mut().unwrap();
                let top_layer = layers.last().unwrap();
                top_mode.update(world, inputs, top_layer.grids.as_slice(), &self.pop_result)
            };

            self.pop_result = None;

            // Control the stack as requested by the top mode update logic.
            match mode_control {
                ModeControl::Stay => {}
                ModeControl::Switch(mode) => {
                    self.stack.pop();
                    layers.pop();
                    layers.push(TileGridLayer {
                        draw_behind: mode.draw_behind(),
                        grids: Vec::new(),
                    });
                    self.stack.push(mode);
                }
                ModeControl::Push(mode) => {
                    layers.push(TileGridLayer {
                        draw_behind: mode.draw_behind(),
                        grids: Vec::new(),
                    });
                    self.stack.push(mode);
                }
                ModeControl::Pop(mode_result) => {
                    self.pop_result = Some(mode_result);
                    self.stack.pop();
                    layers.pop();
                }
                ModeControl::Terminate(mode_result) => {
                    self.pop_result = Some(mode_result);
                    self.stack.clear();
                    layers.clear();
                }
            }

            // Draw modes in the stack from the bottom-up.
            if !self.stack.is_empty() && !matches!(mode_update, ModeUpdate::Immediate) {
                let draw_from = self
                    .stack
                    .iter()
                    .rposition(|mode| !mode.draw_behind())
                    .unwrap_or(0);
                let top = self.stack.len().saturating_sub(1);

                // Draw non-top modes with `active` set to `false`.
                for (i, mode) in self.stack.iter_mut().enumerate().skip(draw_from) {
                    mode.draw(world, &mut layers[i].grids[..], false);
                }

                // Draw top mode with `active` set to `true`.
                self.stack[top].draw(world, &mut layers[top].grids[..], true);
            }

            match mode_update {
                ModeUpdate::Immediate => (),
                ModeUpdate::Update => return RunControl::Update,
                ModeUpdate::WaitForEvent => return RunControl::WaitForEvent,
            }
        }

        RunControl::Quit
    }
}
