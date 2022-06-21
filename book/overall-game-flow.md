# Overall Game Flow

The overall flow of the game can be divided into three main parts:

1. initialization
2. the main game loop
3. the mode stack that the main game loop continuously updates

We'll look at each of these in turn.

## Initialization

The `main` function in `src/main.rs` is where it all begins.
One of the most important things initialized is the *world*, courtesy of the Shipyard crate, whose sole purpose is to store and provide access to all game-related data.
There's a bunch of calls to `world.add_unique` to add *uniques*, which are the closest thing the game has to global variables.
"Uniques" are Shipyard's term for "resources", which is the term used by other Rust ECS crates such as [Specs](https://crates.io/crates/specs), [Legion](https://crates.io/crates/legion) and [bevy\_ecs](https://crates.io/crates/bevy_ecs).
In any case, the uniques that RuggRogue adds to its world are a mix of essential and dummy data.
Inserting dummy data now means it can be replaced unconditionally later, simplifying that code.

The most important thing initialized is the *mode stack*, which is initialized with a mode representing the title screen.
The mode stack deserves its own section, so that will be covered later; just keep it in mind for now.

The final bit of initialization in the `main` function is `RunSettings`, which controls the behavior of the `ruggrogue::run` function that launches the main loop of the game.
Alongside the basic window settings and frames per second is the `tileset_infos` field.
If you want to add new tilesets and fonts, this is where they're added.
The game's option menu assumes that fonts come before tilesets.
Note that if you add or remove fonts, you'll want to change the `NUM_FONTS` constant in `src/modes/options_menu.rs` to match your changes.
If you want to add a tileset, check out the `urizen_tileset_info` function in `src/gamesym.rs` for an example of how to map display symbols to tiles.

At the end of the `main` function is a call to `ruggrogue::run` that launches the main game loop with a callback that continuously updates the aforementioned mode stack.

## The Main Game Loop

The main game loop lives in the `run` function that can be found in `src/lib/run.rs`.
Note that we've gone from the binary crate to the library crate at this point.
We can start the game loop now, right?

I lied, there's more initialization.
This initialization is a lot more technical than the stuff that `main` was setting up before.
Most of the initialization here is to use SDL to prepare a window, a canvas representing the drawable area of that window and an event pump, which is how the game can react to player inputs and window resizing.

Beyond SDL, the `layers` variable is noteworthy.
It's a vector of `TileGridLayer`s, which in turn are vectors of `TileGrid`s.
Everything that appears on screen does so via this structure; it can be thought of as a hand-rolled scene graph.
`TileGrid`s and `TileGridLayer`s will be covered in more detail in a later chapter.

Alright, we can finally move onto the game loop proper, that is, everything inside `while !done { ... }`.
The game loop is modelled directly from the one described under "Play catch up" in the [Game Programming Patterns book](https://gameprogrammingpatterns.com/game-loop.html#play-catch-up).
The idea of this kind of game loop is that you think of wall clock time as producing a resource that is repeatedly consumed in fixed amounts by running the update logic.
However, it's heavily adapted to the needs of RuggRogue, which are kind of unusual compared to a conventional video game.

The biggest thing I wanted to avoid with RuggRogue was constantly spending CPU time running update logic when there was nothing to update.
I didn't want RuggRogue to turn the player's laptop into a hand warmer because they left the game window open while doing something else.
In this respect, RuggRogue's game loop is closer to that of a GUI program than a video game.
This sole requirement unfortunately rules out the use of almost every single Rust game engine, including [bracket-lib](https://crates.io/crates/bracket-lib) and [Bevy](https://bevyengine.org/) (at least in its current pre-editor state).
Virtually all of the Rust game engines out there have their own game loop that they want you to use, with no way to tell it to wait for an event before updating again.
To be honest, I wasn't intending to write my own game engine for RuggRogue, but this one requirement kind of forced me into it, and the rest is history.

There are two parts to making RuggRogue wait for input when needed: the `active_update` flag and the `RunControl` enum.
When `active_update` is true, updates are continuously requested at the desired FPS rate, and when it's `false` it'll wait for an event before updating instead.
The `active_update` flag is initially set, since the game needs to update at least once in order to draw the title screen.
The `RunControl` enum found at the top of `src/lib/run.rs` is how updates tell the game loop to set the `active_update` flag.
Every update function from the main gameplay to the smallest dialog returns one of the variants of `RunControl`.
Anything that finishes its work in a single frame returns `RunControl::WaitForEvent`, such as the player moving a single step or moving a menu cursor.
Things that require repeated updates instead return `RunControl::Update`, such as the player auto-running along a corridor or resting until healed.
`RunControl::Quit` is only returned when the mode stack empties out, which means there's nothing left to update or show on screen.

If you're reading the code, you may wonder why there's a big `if` with two whole branches that run the `update` callback.
This is to ensure correct time book-keeping when going back and forth between active and and inactive updating.
For example, say that there's an update, then the player waits for ten seconds before pressing a key that triggers active updates.
If the game is set to run at 30 FPS with active updates, this would trigger 300 catch-up updates without special handling!

## The Mode Stack

I've hinted at this idea of a "mode stack" before, so it's time to go into it in more detail.
The idea of a *mode* is a unit of state with associated update and drawing logic that has exclusive, or *modal* (hence the name), access to update and control logic at any given time.
The game has a single *mode stack* that houses all of the modes, drawing all of the modes in their stacked order while updating only the top mode.

At this point, you might be thinking,
"Hey, wait a minute, isn't this just a *game state stack*?
Why come up with a different name for something everybody else has already settled on?"
Indeed, you'll get a lot more useful results from search engines if you type "game state stack" instead of whatever the heck I'm calling it.
However, I didn't come up with a different name for no reason.
There is a dizzying amount of writing about game development on the Internet that goes over how to do different things, but in very similar and easy-to-confuse terms.
"State" is one of those words that means slightly different things to different people.
"State" refers to the pattern of bits in memory and allocated resources, but "state" is also a computer science concept for finite state machines.
Using different names for different ideas makes thinking about complex problems easier, and I refer to "modes" instead of "states" here for this reason.
My original inspiration for this approach comes from ["The *Interface* Stack Model"](https://web.archive.org/web/20160306054512/http://director-online.dasdeck.com/buildArticle.php?id=1134) (emphasis mine), but "interface" is unfortunately already a term used in object-oriented programming.
This approach is also somewhat similar to ["Pushdown Automata"](https://gameprogrammingpatterns.com/state.html#pushdown-automata) from the Game Programming Patterns book.
Anyway, enough about names, you can substitute "mode" with "state" and everything here should still make sense.

If RuggRogue is a living being, the main game loop would be its heart, pumping updates, while the mode stack would be its brain, deciding how to react to inputs and what gets drawn on screen.
The mode stack can be found near the bottom of `src/modes/mod.rs`, which is back in the binary crate.
This might seem strange; surely something as general as this mode stack belongs in the library crate instead, right?
However, my experience on this game says otherwise.
The original mode stack was much simpler than the one that exists in the game code now.
I treated it as a living, breathing thing and evolved it to suit the needs of the game, and you can't really put something that constantly evolves into a library that presents a stable interface.
The mode stack must be game code, not library code.

The mode stack is represented by the `ModeStack` struct (surprise), which is just a vector of `Mode` structs, along with a single `ModeStack::update` function.
The `ModeStack::update` function more or less does the following:

1. Call `Mode::prepare_grids` on all of the modes in the stack to create and position tile grids that the modes draw onto.
2. Call `Mode::update` on the top mode and catch its result.
3. React to the returned result if needed, e.g. to push a new mode or pop the top mode.
4. Call `Mode::draw` on all of the modes in the stack to fill in the contents of tile grids that will later be displayed on screen.

These mode-related functions exist just above the mode stack code inside the `impl Mode` block.
The purpose of these functions is to dispatch to the function of the same name in each mode according to its type.
For example, if there's a `YesNoDialogMode` at the top of the mode stack, `ModeStack::update` would call `Mode::update` which in turn would call `YesNoDialogMode::update`, which can be found in `src/modes/yes_no_dialog.rs`.
You'll notice that the dispatching code is mostly a copy-paste job; there are a couple of alternatives that would have been much more concise, but I decided against using.
Idiomatic Rust code would have made a `Mode` trait instead of an enum and relied on dynamic dispatch using `Box<dyn Mode>`, but having every mode living in different parts of heap memory was something I wanted to avoid for performance reasons.
There's also an [enum\_dispatch](https://crates.io/crates/enum_dispatch) crate that does what I did by hand automatically, but I didn't want to pull in dependencies for things I could easily write myself.

`Mode::update` returns a 2-tuple of `ModeControl` and `ModeUpdate`, whose definitions are just above `impl Mode` in `src/modes/mod.rs`.
`ModeControl` represents what the `update` function of any given mode wants to have done to the stack, like `ModeControl::Switch`, `ModeControl::Push` or `ModeControl::Pop`.
Note the `ModeResult` in `ModeControl::Pop`: every mode is accompanied by a corresponding *mode result*, e.g. `YesNoDialogMode` can pop itself off the stack and return either `YesNoDialogModeResult::Yes` or `YesNoDialogModeResult::No`.
The next mode whose `update` function is called will receive this result via its `pop_result` parameter.
Rust's expressive type system is invaluable for dispatch logic based on types like this.
A lot of articles written about game state stacks gloss over result handling, but it's the difference between a dialog returning something meaningful and just vanishing into thin air, so if you want proper dialogs and not just inert windows, it's crucial.

`ModeUpdate` determines what should happen after `ModeStack::update` is done.
`ModeUpdate::Update` and `ModeUpdate::WaitForEvent` correspond to active and inactive-style of main game loop updating described earlier.
You may have noticed the `while` loop surrounding most of the code in `ModeStack::update`; `ModeUpdate::Immediate` exists solely as a fallthrough case to repeat that loop.
`ModeUpdate::Immediate` is used to make the next mode's `update` function run in the *current* frame instead of the next.
This is useful to handle the result of a dialog straight away when it closes instead of lagging for a frame.

One last detail I left out is `Mode::draw_behind`.
Remember how modes represent screens as well as menus and dialogs?
There's no need to draw behind a mode that covers the entire screen, so `Mode::prepare_grids` and `Mode::draw` are skipped for other modes behind fullscreen modes like this, and `Mode::draw_behind` is what decides if a mode is fullscreen.

If you want an example of a simple mode, the simplest one can be found in `src/modes/yes_no_dialog.rs`.
The most important mode is probably `DungeonMode`, which represents the main gameplay screen and handles player and monster turn distribution; this can be found in `src/modes/dungeon.rs`.
