# RuggRogue

RuggRogue is a simple but complete roguelike written in Rust with SDL2, playable native or in a web browser in WebAssembly form with the help of Emscripten.
It was made primarily as a learning exercise after following the [Rust roguelike tutorial](http://bfnightly.bracketproductions.com/), but wanting to work around a couple of issues that could only be solved by starting from scratch and writing an engine by hand.

## Features

* A dungeon around 20 randomly-generated levels deep.
* 20 monsters, 10 weapons and 10 armors that spawn with increasing power at deeper levels.
* 5 different types of consumable items.
* Hunger mechanic with health regeneration based on fullness.
* Tiles and ASCII graphics options, with optional 2x zoom.
* Save and load that works even in the browser version.
* Menu-based interface with hotkey support for common actions.
* Auto-run that follows corridors and stops for items and stairs.
* New game plus mode that spawns more monsters and items and more powerful equipment for successive wins.
* The simple satisfaction of ever-increasing numbers.

## Controls

The controls have been designed to be similar to other classic roguelikes for the most part.
Like other roguelikes, simply move into monsters to attack them.

* **numpad, arrows, hjklyubn** - move in eight directions
* **period, space, numpad 5** - wait a turn
* **shift + direction** - auto-run in a direction
* **shift + space** - rest until healed
* **enter, shift + period (>)** - use downstairs
* **escape** - options menu
* **i** - inventory
* **g, comma** - get (pick up) item
* **d** - drop item
* **a** - apply (use) item
* **w, e** - wield (equip) weapon or wear armor
* **r** - remove weapon or armor
* **v** - enter view mode; move the cursor to describe distant tiles and move the camera around; hold shift for faster movement

Menus can be navigated with the arrow keys, confirmed with enter and cancelled with escape.
Item menus also support navigation keys like home, end, page up and page down.

## How to build the game

First, get the source code at <https://github.com/tung/ruggrogue>

To compile any version of the game, you'll need to [install Rust](https://www.rust-lang.org/tools/install).

On Linux, install the development libraries for [SDL2 (`libsdl2-dev`)](https://libsdl.org/) and [SDL2\_image (`libsdl2-image-dev`)](https://www.libsdl.org/projects/SDL_image/), then run `cargo build --release` to build the game, and `cargo run --release` to start playing.

There's no support for building a native version of the game for Windows out of the box.
The easiest way would probably be to alter `Cargo.toml` to fetch and build SDL2 and SDL2\_image into a static binary using cargo-vcpkg according to the [rust-sdl2 instructions](https://github.com/Rust-SDL2/rust-sdl2#windows-linux-and-macos-with-vcpkg).
If anybody can try this and test that it works, patches are welcome.

To build the web browser version:

1. Start by running `rustup target add wasm32-unknown-emscripten` to install the WebAssembly + Emscripten target for Rust.
2. Next, install version **1.39.20** of [Emscripten](https://emscripten.org/docs/getting_started/downloads.html) using emsdk; *newer versions will not work*.
3. Activate `emsdk_env.sh`, then run `cargo build --target=wasm32-unknown-emscripten --release`.
4. Follow the instructions in `index.html` to serve it from a local web server to try it out in a browser; aside from `index.html`, the relevant files are `ruggrogue.wasm`, `ruggrogue.js`, and `deps/ruggrogue.data`, which can all be found under `target/wasm32-unknown/release/`.

## License

RuggRogue is released under the MIT License; see [LICENSE.txt](/LICENSE.txt) for the full text.

The tile graphics are from the [Urizen 1Bit Tilesets by vurmux](https://vurmux.itch.io/urizen-onebit-tilesets), licensed under Creative Commons Zero; see [assets/urizen/LICENSE](/assets/urizen/LICENSE) and [assets/urizen/readme.txt](/assets/urizen/readme.txt).

[GohuFont](https://font.gohu.org/) is licensed under the [WTFPL](http://www.wtfpl.net/about/).

`assets/terminal-8x8.png` was borrowed from the assets accompanying the [Rust roguelike tutorial](http://bfnightly.bracketproductions.com/), under whatever license that the tutorial originally got the font image from.
