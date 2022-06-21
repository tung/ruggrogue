# Event Handling

As a game, it's crucial for RuggRogue to react to *events*.
The most important events are the key presses of the player playing the game, but there are other kinds that need to be handled too, like window resizing, mouse inputs and attempts to close the game window.
This chapter will talk about where events come from and how the game responds to them.

## Receiving Events

Our journey begins in the `ruggrogue::run` function in the `src/lib/run.rs` file.
The `sdl2` crate provides what's called an *event pump*, which is the source of all events that the game will see.
This event pump is stored in the aptly-named `event_pump` variable so that the main game loop can pull events out of it with its `wait_event` and `poll_iter` methods.

Each event from the event pump is handled in two phases:

1. Do things that need to be handled straight away.
2. Enqueue the event in an input buffer so that it can be handled later during a proper game tick.

Some events are directly handled only and aren't enqueued, such as window resizing and mouse inputs.
On the other hand, key press events have a little bit of direct handling before being enqueued for the game logic to handle them properly later.
Events that the game doesn't care about are simply ignored.

## Direct Event Handling

There are three kinds of events that are handled directly in the game's main loop: window resizing, mouse inputs and a couple of rendering-related events.

Window resize events update the `window_size` variable in the main loop with the new window size.
This is later sent into the `update` callback that was given to the `ruggrogue::run` function so that the updating and drawing logic are always aware of the size of the game window.
The updating logic in particular needs this info so that menus know how far to scroll when pressing the page up and page down keys.

The game hides the mouse cursor in response to key presses; mouse input events reveal it again.
These include mouse movement, mouse button presses and mouse wheel movement.

The two rendering-related events that need direct handling are both things that can happen on Windows with DirectX being used as the graphics backend for SDL:

- `sdl2::event::Event::RenderTargetsReset` happens when graphical texture data needs to be updated.
- `sdl2::event::Event::RenderDeviceReset` happens when textures have been lost and need to be recreated entirely.

In both cases the game will flag its graphics-related data to do the right thing the next time that they need to be drawn to the screen.

## The Input Buffer

Once any direct handling is done, the event may be added to the *input buffer*.
The game logic will almost always run less often than the main loop, so the purpose of the input buffer is to save events from the main loop so that the game logic can react to them later.

The `inputs` variable in the `ruggrogue::run` function holds the input buffer.
This is an `InputBuffer` struct that enqueues mainly keyboard events when its `InputBuffer::handle_event` function is called with an event.

The `InputBuffer` struct is defined in the `src/lib/input_buffer.rs` file.
When the game logic wishes to check for input events, it follows these steps:

1. The game logic calls the `InputBuffer::prepare_input` function to retrieve a single input event from the buffer.
2. The game logic calls the `InputBuffer::get_input` function to check the prepared input event.
3. The end of the game loop calls the `InputBuffer::clear_input` function to make way for the next call to the `InputBuffer::prepare_input` function.

The events stored in the `InputBuffer` struct are a stripped-down form of SDL's events in the form of small `InputEvent` enums that mainly hold SDL key codes that are unique for each keyboard key.
As `InputEvent`s are pulled from the `InputBuffer`, the `InputBuffer` tracks the press state of the *modifier keys* (i.e. `Shift`, `Ctrl` and `Alt`) that the game logic can read using the `InputBuffer::get_mods` function.

The game logic will typically combine the prepared input and modifier key state into a logical *game key*, represented by the `GameKey` enum defined in the `src/gamekey.rs` file.
The `gamekey::from_keycode` function in that file translates the SDL key code values into logical game key values.
Note that multiple key codes can translate into a single game key, e.g. the up cursor key, `8` on the number pad and `k` all translate into the `GameKey::Up` value.

## Player Input Logic

Every game mode pulls inputs from the input buffer, but the most important of these modes is `DungeonMode`.
The `DungeonMode::update` function defined in the `src/modes/dungeon.rs` file is the central place that drives player and monster turns.
Player inputs are handed off from this function to the `player::player_input` function defined near the bottom of the `src/player.rs` file.

Under normal circumstances, where the player is not asleep or auto-running, the `player::player_input` function will prepare, retrieve and translate an input event into a game key.
How each game key is handled falls into one of three categories:

1. *Movement* - Move the player or melee attack an adjacent enemy with the `try_move_player` helper function.
2. *Wait in place* - Cause the player to wait a single turn with the `wait_player` helper function.
3. *Anything else* - Return a value that the `DungeonMode::update` function should handle instead, usually by manipulating the mode stack to show a dialog or menu.

The return value of the `player::player_input` function is a `PlayerInputResult` enum variant whose definition is found near the top of the `src/player.rs` file.
The most important values are `PlayerInputResult::NoResult` and `PlayerInputResult::TurnDone`, which control whether or not the `DungeonMode::update` function should finish the player's turn and advance time.
Valid player actions will typically alter the world state during the `player::player_input` function call and then cause it to return the `PlayerInputResult::TurnDone` value.

## The `AppQuit` Event

In the native build of RuggRogue, when the player attempts to close the game window, the `sdl2` crate emits the `sdl2::event::Event::Quit` event.
This is translated into an `InputEvent::AppQuit` event that gets inserted into the `InputBuffer` in the `InputBuffer::handle_event` function.
This means that every place in the game logic that checks for input must also respond to this `InputEvent::AppQuit` event.
Responses fall into one of three categories:

1. Most modes pop themselves off the mode stack and return an `AppQuit` result, e.g. a `FooMode::update` function returns a `FooModeResult::AppQuit` result.
2. `DungeonMode` pushes an instance of `AppQuitDialogMode` (defined in the `src/modes/app_quit_dialog.rs` file) to show a save-and-exit confirm dialog; it also does this if any mode on top of it in the mode stack returns its own `AppQuit` result.
3. `AppQuitDialogMode` ignores `AppQuit` events while waiting for the player to pick a response.

The combined effect of these responses will either quit the game outright (by emptying out the mode stack) or show a save-and-exit confirm dialog if the player is in the middle of playing the game (the `DungeonMode` catches `AppQuit` events and mode results to show the dialog).
