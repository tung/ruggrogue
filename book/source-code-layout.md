# Source Code Layout

RuggRogue comes to about 13,000 lines of source code.
In Rust terms, the entire RuggRogue source code comprises a *package* made up of two *crates*: a *library crate* and a *binary crate*.
The library crate contains things that RuggRogue needs to function that aren't specific to it as a game.
The binary crate has the game-specific stuff and makes use of the library crate to form the complete game when built.

## The Library Crate

Some people consider SDL to be a game engine, but what it provides is so low-level that it's really more of a layer to build a game engine on top of.
RuggRogue's library crate is effectively a small, custom game engine for RuggRogue the game.
It also has some utilities that are useful to RuggRogue as a roguelike, but are otherwise independent of the game itself.

The library crate lives in `src/lib/`, and is made up of the following files:

 - `src/lib/lib.rs` - The "crate root" of the library crate, in Rust terms, that pulls together all of the other files that make up the library crate.
 - `src/lib/field_of_view.rs` - Field of view calculation.
 - `src/lib/input_buffer.rs` - A first-in-first-out queue of simplified input events translated from SDL input events, consumed by the game proper.
 - `src/lib/path_find.rs` - A\* path finding algorithm that monsters use to pursue the player.
 - `src/lib/run.rs` - Window initialization and the main game loop.
 - `src/lib/tilegrid.rs` - A pixel-perfect tile grid implementation, used to render everything that shows up on screen; this is the biggest source code file in the game!
 - `src/lib/util.rs` - Contains small utility structs, namely `Color`, `Position` and `Size`.
 - `src/lib/word_wrap.rs` - Word wrapping algorithm that splits a long string into lines of at most a given number of characters.

In theory, the existence of this library crate means that other Rust projects could make use of it.
In practice, it might be tricky, since I only included enough features to make RuggRogue functional.
Faced with the task of writing my own game engine, my guiding principle was to never write code I didn't have an immediate use for.
Unused code is untested code, and untested code is buggy code.

## The Binary Crate

The binary crate contains all of the logic that is specific to RuggRogue as a game.
The top-level `src/` directory is a melting pot of different things:

 - `src/main.rs` - The crate root of the binary crate that pulls together the rest of the files listed below, with the entry point of the game that sets everything up and launches the game loop.
 - `src/bitgrid.rs` - Holds `BitGrid`, a struct used to track map tiles revealed by the player, as well as which tiles are contained in the fields of view of entities.
 - `src/chunked.rs` - Holds `ChunkedMapGrid`, a struct that handles a [dirty rectangles](https://wiki.c2.com/?DirtyRectangles) drawing scheme to avoid having to repeatedly redraw large portions of the map on screen.
 - `src/components.rs` - Definitions of component structs, which are data associated with entities.
 - `src/damage.rs` - Damage calculations and handling of dead entities.
 - `src/experience.rs` - Experience and difficulty tracking, as well as the definition of how combat stats relate to experience level values.
 - `src/gamekey.rs` - Translation of SDL key values into game-specific action keys.
 - `src/gamesym.rs` - Symbolic representation of tile appearances and their ASCII equivalents, as well as a hard-coded mapping for the tileset used by the game.
 - `src/hunger.rs` - Hunger and regeneration tracking.
 - `src/item.rs` - All item-related functionality and book-keeping, along with handling of item-inflicted status effects.
 - `src/magicnum.rs` - Arbitrary values used to help seed the different random number generators created in other places in the source code.
 - `src/map.rs` - Holds the `Tile` and `Map` structs, handles map generation and maintenance of a tile-based spatial cache for performance.
 - `src/menu_memory.rs` - Holds a `MenuMemory` struct that remembers the last position of the cursor in various menus.
 - `src/message.rs` - The message buffer.
 - `src/monster.rs` - Monster turn handling and AI.
 - `src/player.rs` - Player input and turn handling, as well as auto-run logic.
 - `src/render.rs` - Drawing of entities on the map.
 - `src/saveload.rs` - Everything to do with saving the game to and loading a game from a save file.
 - `src/spawn.rs` - Spawning and despawning of all entities, including filling map rooms with spawns, along with monster, weapon and armor appearances.
 - `src/ui.rs` - Arrangement and drawing of the main game interface, i.e. the map, sidebar and messages.
 - `src/vision.rs` - Updates fields of view for entities that have one and need it updated.

You'll also notice the `src/modes/` directory.
This is the home of what I call the *mode stack* as well as the *modes* that go on it.
There'll be more on this later on, but modes represent screens, menus and dialogs, while the mode stack determines what modes appear on screen and which one updates at any given time.
The files in `src/modes/` consist of:

 - `src/modes/mod.rs` - The Rust sub-module that pulls together the individual mode files, as well as holding the mode stack logic.
 - `src/modes/app_quit_dialog.rs` - Confirmation dialog when the player tries to close the window in the native build of the game.
 - `src/modes/dungeon.rs` - The main gameplay screen that drives the core gameplay loop and pulls all of the game logic together.
 - `src/modes/equipment_action.rs` - Menu of actions that can be performed when selecting an equipped item.
 - `src/modes/equipment_shortcut.rs` - Quick hotkey-reachable menu to remove or drop an equipped item without having to go through the inventory.
 - `src/modes/game_over.rs` - The game over and victory screens.
 - `src/modes/inventory.rs` - The inventory menu.
 - `src/modes/inventory_action.rs` - Menu of actions that can be performed when selecting an inventory item.
 - `src/modes/inventory_shortcut.rs` - Quick hotkey-reachable menu to perform an action on an item without having to go through the inventory.
 - `src/modes/message_box.rs` - A simple message box.
 - `src/modes/options_menu.rs` - The options menu where settings can be changed.
 - `src/modes/pick_up_menu.rs` - Menu of items that the player can pick up at their current map position.
 - `src/modes/target.rs` - A screen that allows the player to choose a target position when they use an item that needs a target.
 - `src/modes/title.rs` - The title screen.
 - `src/modes/view_map.rs` - A screen that lets the player move the camera around and describe map positions.
 - `src/modes/yes_no_dialog.rs` - A simple yes-or-no dialog.

## Assets

The `assets/` directory has files loaded by the game at runtime:

 - `assets/gohufont-8x14.png` - A PNG of [IBM Code Page 437](https://en.wikipedia.org/wiki/Code_page_437) rendered with [GohuFont](https://font.gohu.org/), the default font of the game.
 - `assets/terminal-8x8.png` - A PNG of IBM Code Page 437 rendered with a smaller 8-by-8 pixel font that came from the `resources.zip` of the [Rust Roguelike Tutorial](https://bfnightly.bracketproductions.com/chapter_1.html#hello-rust---rltk-style).
 - `assets/urizen/urizen-onebit-tileset-mono.png` - A custom black-and-white version of one of the tileset images from the [Urizen 1Bit Tilesets by vurmux](https://vurmux.itch.io/urizen-onebit-tilesets).
 - `assets/urizen/readme.txt` - Description of my changes to the Urizen tileset image.
 - `assets/urizen/LICENSE` - License text for Urizen 1Bit Tilesets.

## Everything Else

There's a bunch of non-Rust or not-quite-Rust files that can also be found in the source code:

 - `BUILD.md` - Instructions for building RuggRogue from source.
 - `Cargo.toml` - RuggRogue's package metadata that describes the source code structure and dependencies and is used by Cargo to build the game.
 - `.cargo/config.toml` - Extra Cargo configuration for building the web version of the game.
 - `clippy.toml` - Settings for Clippy, Rust's source code linter.
 - `index.html` - HTML page that hosts the web version of the game.
 - `LICENSE.md` - License text for RuggRogue.
 - `README.md` - Basic information about RuggRogue.
 - `ruggrogue.js` - Support JavaScript needed for the web version of the game.

Finally, there's `book.toml` and the `book/` directory, which is the very book you are reading right now!
If you have the RuggRogue source code, you can install [mdbook](https://crates.io/crates/mdbook) and run `mdbook build --open` for your very own local copy.
