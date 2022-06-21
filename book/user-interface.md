# User Interface

This chapter covers how RuggRogue handles menus and dialogs, the layout and drawing of the main game screen, and how application closing is handled.

## Menus

Most of RuggRogue's interface exists in the form of menus and dialogs.
As mentioned in the overall game flow chapter, menus and dialogs are represented as modes in the game's mode stack.
Because of this, there's no real difference between a "menu" and a "dialog": they both present themselves as tile grids, react to player input and return some result.
This is a perfect excuse to demonstrate how menus work by using a dialog as an example instead.

The `YesNoDialogMode` struct in the `src/modes/yes_no_dialog.rs` file is the simplest dialog, and therefore the simplest menu, in the game.
The struct itself contains the `prompt` field that is shown to the player and a `yes_selected` boolean that the player can change by pressing keys.
Every menu and dialog holds data like this: one or more fields related to presentation, and a selection that represents a player-controlled cursor.
Sometimes this selection will be accompanied by a `subsection` field for more complex menus; the `YesNoDialogMode` doesn't need one, so it doesn't have one.

Above the definition of `YesNoDialogMode` is the `YesNoDialogModeResult` enum.
When the `YesNoDialogMode` is closed, it returns an instance of `YesNoDialogModeResult` to the mode immediately below it in the mode stack.
There are three variants: `Yes`, `No` and `AppQuit`.
The first two variants should be obvious; the `AppQuit` variant is explained in the [Event Handling chapter](event-handling.md#the-appquit-event).

If the game wants to show a yes-or-no prompt, it has to create a `YesNoDialogMode` using the `YesNoDialogMode::new` function.
There's an example of this when the player chooses to "save and exit" in the options menu.
This corresponds to the `OptionsMenuMode` in `src/modes/options_menu.rs`; look for the "Save and return to title screen" message near the end of the `OptionsMenuMode::update` function.
There are two important things that need to be done to show a yes-or-no prompt:

1. Create a `ModeControl::Push` with an instance of `YesNoDialogMode` created with the `YesNoDialogMode::new` function.
2. Clear the input queue using `inputs.clear_input()` followed by `ModeUpdate::Immediate` for same-frame result handling while avoiding double-handling of keys.

The mode stack will then take the `YesNoDialogMode` that was wrapped in the `ModeControl::Push` return value, add it to the mode stack and prepare a fresh `TileGridLayer` for it.

Once a mode is present in the mode stack, it calls these mode-related functions in order:

1. `prepare_grids`
2. `update`, if the mode is at the top of the stack
3. `draw`

Back in `src/modes/yes_no_dialog.rs`, the `YesNoDialogMode::prepare_grids` function is the very first function that is called when the `YesNoDialogMode` is on the stack.
This ensures that the `update` and `draw` functions have the same view of the screen and tile grids on any given frame.
The first thing this function does is calculate the dimensions of the tile grid it wants to draw in, whether or not such a tile grid even exists yet.
On the very first call, the vector of `TileGrid`s corresponding to the `TileGridLayer` assigned to the mode is empty, so the `YesNoDialogMode::prepare_grids` function will create a fresh `TileGrid` with the desired dimensions.
On subsequent calls that tile grid will already exist, so it will just be resized instead.
The `YesNoDialogMode::prepare_grids` function wraps up by setting its tileset, position (`TileGrid::view_centered` is a helper to adjust the `TileGridView`) and zoom factor.
The fact that this information is calculated and assigned every frame is what allows the options menu to instantly take effect on the entire interface.

The `YesNoDialogMode::update` function is how the dialog responds to player input.
First, an input is pulled in from the input queue by calling the `input.prepare_input` function.
Next, that input event is read out by calling the `input.get_input` function.
Assuming it's a key press event, it is then translated into a logical game key by calling the `gamekey::from_keycode` function.
The `YesNoDialogMode::update` function reacts to `GameKey::Left` and `GameKey::Right` by altering the selected option.

The `YesNoDialogMode::draw` function draws the dialog itself.
The first thing it does is dim itself if it's not the top-most mode on the stack by setting `color_mod` to `Color::GRAY` in response to the value of the `active` parameter.
The drawing itself takes place after that, drawing the box border and message.
When drawing the "Yes" and "No" options, it reads the `yes_selected` field of the mode to highlight whichever option the player currently has selected.

Eventually the player will pick either the "Yes" or "No" options.
This is picked up in the `YesNoDialogMode::update` function when it receives `GameKey::Confirm` or `GameKey::Cancel` as a input key.
At this point, the `YesNoDialogMode` will create an instance of either `YesNoDialogModeResult::Yes` or `YesNoDialogModeResult::No`, and wrap it in `ModeControl::Pop` to tell the mode stack to pop the `YesNoDialogMode` and send the `YesNoDialogModeResult` to whatever mode pushed it on to begin with.

This takes us back to the "save and exit" logic in the `OptionsMenuMode::update` function in `src/modes/options_menu.rs` file.
The `YesNoDialogModeResult` will be received in the `pop_result` parameter of the `OptionsMenuMode::update` function, and then responded to in the block starting with `if let Some(result) = pop_result`.
In this case, `OptionsMenuMode` responds to `YesNoDialogModeResult::Yes` by popping itself off the mode stack with its own `OptionsMenuModeResult::ReallyQuit` value.

This covers the entire life-cycle of a yes-or-no dialog:

1. A mode that wants a yes-or-no dialog creates a `YesNoDialogMode` instance that gets pushed onto the mode stack.
2. The `YesNoDialogMode::prepare_grids` function is called to create a tile grid or adjust an existing one.
3. `YesNoDialogMode::update` responds to player inputs.
4. `YesNoDialogMode::draw` draws the dialog itself.
5. `YesNoDialogMode::update` eventually pops itself off the mode stack with an instance of `YesNoDialogModeResult`.
6. The original mode beneath catches the `YesNoDialogModeResult` and reacts to it.

This life-cycle is the foundation of every single dialog and menu in the game, even the `InventoryMode`, found in the `src/modes/inventory.rs` and the biggest of all the menus.

## The Main Game Screen

The majority of the gameplay takes place in `DungeonMode`, which can be found in the `src/modes/dungeon.rs` file.
It is responsible for handling player control, distributing turns and drawing the main game interface.
This section describes how the interface is laid out and drawn; player control and turn order will be covered in a later chapter.

The main game screen consists of multiple tile grids that the dungeon mode creates in its mode-stack-designated tile grid layer:

1. The *map grid* that shows the dungeon map, the player, items and monsters.
2. The *status grid* that shows the player's status information, such as their level, health, hunger and turns.
3. The *item grid* that shows the player's equipment and number of carried inventory items.
4. The *message frame grid* that draws a border around the message log.
5. The *message grid* that shows the message log.

The distinction between the message frame grid and the message grid is a bit janky.
The split was part of a plan to use wrapped offset rendering to increase message rendering performance, but it never ended up happening.
If I were to revisit this part of the code I would just have a single message grid that draws its frame like everything else.

The `DungeonMode::new` function prepares the book-keeping for the dungeon mode, the most important part of which is for chunked map drawing, described in detail back in the [Rendering chapter](rendering.md#improving-map-drawing-performance-with-chunked-drawing).

Things get slightly more interesting with the `DungeonMode::prepare_grids` function, which immediately delegates all of its work to the `ui::prepare_grids` function.
This function can be found at the very bottom of the `src/ui.rs` file, and is responsible for calculating and setting the size and position of all the main game screen tile grids.
Despite living in a separate file, it serves the same function as any code found in the `prepare_grids` function of any other mode.
After setting the size of the map grid, it calls the `ChunkedMapGrid::prepare_grid` function so that it can prepare and adjust itself to the map tile grid and screen dimensions.

Back in `src/modes/dungeon.rs`, the `DungeonMode::draw` function is responsible for coordinating the drawing of all the main game screen tile grids.
Pretty much all of the drawing is delegated here as well.
The `ChunkedMapGrid::draw` function renders the map itself, while entities on the map are drawn via the `render::draw_renderables` function, defined in the `src/render.rs` file.
All of the sidebar tile grids are drawn via the `ui::draw_ui` function, found in the `src/ui.rs` file.
The `ui::draw_ui` function in turn calls the `draw_status`, `draw_item_info` and `draw_messages` functions to fill out each of the grids.
The `draw_messages` function in particular applies word wrapping to message lines; this is covered in its own chapter.

Apart from `DungeonMode`, there are two other modes that also draw the main game screen in this fashion: `TargetMode` and `ViewMapMode`.
`TargetMode` is defined in `src/modes/target.rs` and allows the player to pick a target tile when using an item that needs a target.
`ViewMapMode` is defined in `src/modes/view_map.rs` and allows the player to pan the camera while describing map tiles.
Both of these modes show dynamically-updating text in the message area by filling in the optional `prompt` parameter when calling the `ui::draw_ui` function.
