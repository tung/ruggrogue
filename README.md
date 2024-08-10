# RuggRogue

itch.io: <https://tungtn.itch.io/ruggrogue>

Website: <https://tung.github.io/ruggrogue/>

Fight monsters and find loot as you battle your way to the bottom of the dungeon!
Play the game [in your web browser](https://tung.github.io/ruggrogue/play/), or download it for Windows or Linux [at GitHub in the Releases section](https://github.com/tung/ruggrogue/releases).

RuggRogue is a simple, complete roguelike inspired by the first part of the [Rust Roguelike Tutorial](https://bfnightly.bracketproductions.com/).
Despite this, it uses [SDL2](https://libsdl.org) directly (and [Emscripten](https://emscripten.org) for the web port), without any game engine or roguelike helper libraries.
The source code can be found [at GitHub](https://github.com/tung/ruggrogue).
Roguelike developers may also want to see the [RuggRogue Source Code Guide](https://tung.github.io/ruggrogue/source-code-guide/); a 20-odd chapter technical web book about the development, ideas and code architecture of the game.

## Features

- Discover new monsters and equipment the deeper you go.
- Hunger and regeneration: stay fed and stay healed!
- Choose between graphical tiles and ASCII display.
- Menu-based UI with hot keys.
- Auto-run to quickly follow corridors and cross open space.
- Save and load system.
- New Game Plus mode!

## Controls

The controls are mostly similar to many other classic roguelikes.
Move into monsters to attack them.

*Movement keys:*

- **Numpad**, **Arrows**, **vi-keys** - move in eight directions
- **Period**, **Space**, **Numpad 5** - wait a turn
- **Shift + direction** - auto-run
- **Shift + Space** - rest until healed
- **Enter**, **>** (Shift + Period) - use downstairs

*Other keys:*

- **Esc** - options menu
- **v** - view mode; move the cursor to view distant tiles

*Item keys:*

- **i** - inventory menu
- **g**, **,** (Comma) - pick up item

*Hot keys:*

- **a** - apply (use) item
- **d** - drop item
- **w**, **e** - wield weapon or wear armor
- **r** - remove weapon or armor

*Menu keys:*

- **Movement keys**, **Page Up/Page Down/Home/End** - move cursor
- **Enter** - confirm selection
- **Esc** - cancel

Hot keys can be used in certain item menus to quickly perform actions.

## Licenses

RuggRogue is released under the [MIT License](/LICENSE.txt).

The tile graphics are from the [Urizen 1Bit Tilesets by vurmux](https://vurmux.itch.io/urizen-onebit-tilesets), licensed under [Creative Commons Zero](/assets/urizen/LICENSE).

[GohuFont](https://font.gohu.org/) is licensed under the [WTFPL](http://www.wtfpl.net/about/).

`assets/terminal-8x8.png` was borrowed from the assets accompanying the [Rust Roguelike Tutorial](http://bfnightly.bracketproductions.com/).
