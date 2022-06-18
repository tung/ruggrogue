# How to Build the Game

RuggRogue can be compiled natively with [SDL2](https://libsdl.org) and [SDL2\_image](https://libsdl.org/projects/SDL_image/), and for web browsers using [Emscripten](https://emscripten.org).

## Compiling for Linux

These steps will create a native Linux binary using system-installed versions of SDL2 and SDL2\_image.

1. Get the source code: <https://github.com/tung/ruggrogue>
2. Install [Rust](https://www.rust-lang.org/tools/install).
3. Install the SDL2 and SDL2\_image development libraries for your distribution; for example, these are named `libsdl2-dev` and `libsdl2-image-dev` on Ubuntu.
4. Run `cargo build --release`

The game binary can then be found at `target/release/ruggrogue` and played by running `cargo run --release`.
The game reads the `assets` direction and writes its save file to the current directory.

## Compiling for Windows

These steps will create a native Windows executable version of the game.

1. Get the source code: <https://github.com/tung/ruggrogue>
2. Install [Rust](https://forge.rust-lang.org/infra/other-installation-methods.html), and remember your choice of MSVC vs. MinGW (GNU) for the next step.
3. Choose one of the ways to install SDL2 for Rust on Windows, matching MSVC vs. MinGW for your Rust installation, and don't forget SDL2\_image!
4. Run `cargo build --release`

If there were no errors, the game can be played by running `cargo run --release`.
You may need `SDL2.dll` from SDL2 and `SDL2_image.dll`, `libpng16-16.dll` and `zlib1.dll` from SDL2\_image to run the game properly.

## Cross Compiling for Windows on Linux

These steps can be run from Linux to create a Windows executable of the game.

1. Get the source code: <https://github.com/tung/ruggrogue>
2. Install [Rust](https://www.rust-lang.org/tools/install).
3. Run `rustup target add x86_64-pc-windows-gnu`
4. Install [Zig](https://ziglang.org) and [cargo-zigbuild](https://crates.io/crates/cargo-zigbuild).
5. Download the Windows development libraries for [SDL2](https://libsdl.org/download-2.0.php) and [SDL2\_image](https://libsdl.org/projects/SDL_image/); ensure you get the MinGW versions!
6. Extract the SDL2 and SDL2\_image files somewhere convenient; they can be safely deleted later.
7. Copy the files in `x86_64-w64-mingw32/lib/` from SDL2 into `~/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-pc-windows-gnu/lib/`
8. Repeat the last step, but with SDL2\_image.
9. Run `cargo zigbuild --release --target=x86_64-pc-windows-gnu`

The file `ruggrogue.exe` should appear in the `target/x86_64-pc-windows-gnu/release/` directory.
The following DLL files need to be in the same directory for it to run:

- `SDL2.dll` from the `x86_64-w64-mingw32/bin/` directory of the extracted SDL2 files
- `SDL2\_image.dll`, `libpng16-16.dll` and `zlib1.dll` from the `x86_64-w64-mingw32/bin/` directory of the extracted SDL2\_image files

## Compiling the Browser Version with Emscripten

These steps will create a WebAssembly file and supporting files that can appear in a web page for a browser-playable version of the game.

1. Get the source code: <https://github.com/tung/ruggrogue>
2. Install [Rust](https://www.rust-lang.org/tools/install).
3. Run `rustup target add wasm32-unknown-emscripten`
4. Install [Emscripten](https://emscripten.org/docs/getting_started/downloads.html) version **1.39.20**; the latest version won't work!
5. Activate Emscripten and run `cargo build --release --target=wasm32-unknown-emscripten`

This creates `ruggrogue.wasm`, `ruggrogue.js` and `deps/ruggrogue.data` in the `target/wasm32-unknown-emscripten/` directory.
Loading `ruggrogue.js` in a *hosted* web page will load the other files and run the game; to test this on Linux:

1. Copy `index.html` from the root of the source code into `target/wasm32-unknown-emscripten/release/`.
2. Enter the `target/wasm32-unknown-emscripten/` directory.
3. Run `python3 -m http.server 9000`
4. Open `http://127.0.0.1:9000` in a web browser to play the game.
