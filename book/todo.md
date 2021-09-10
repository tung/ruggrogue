- **Intro**
  - What is this doc?
  - Who is it for?
  - What does it cover?
- **Dependencies**
  - bitflags
  - bitvec
  - rand, rand\_xoshiro
  - sdl2 (with "image" feature)
  - serde, serde\_json
  - shipyard
  - wyhash
- **Source code layout**
  - It's around 13K SLOC.
  - Most of this won't make sense; see further for context.
  - `assets/` - font and tile images
  - `src/` - game-specific files, quick overview of each file
  - `src/lib/` - game-agnostic files, quick overview of each file
  - `src/modes/` - mode stack implementation and every mode, describe each file starting with mod.rs
- **How stuff works**
  - No engine (unless SDL2 counts) and no roguelike helper libraries.
  - *Overall game flow*
  - *Game loop*
  - *The mode stack*
    - Avoiding the all-encompassing state machine from the Rust roguelike tutorial.
    - ModeResults, or how to handle responses from menus and dialog boxes.
    - Design approach to UI, or why is there so much UI code?
  - *Input queue*
  - *Rendering and display using TileGrids*
    - The phases of output: draw, render, upload, display.
    - The hybrid software-hardware rendering scheme
      - Core idea: Render to TileGrids, display TileGrids on screen.
      - TileGrids update at different rates; only render and upload when they change, e.g. TileGrids of top mode update a lot, but lower modes are mostly static.
      - Decent performance and high portability with SDL2 without having access to a programmable shader pipeline.
    - A primitive scene graph using TileGrid, TileGridLayer and Vec\<TileGridLayer\>.
    - TileGrid breakdown: RawTileGrids, SdlSurface, SdlTexture, TileGridView.
    - Improving render performance
      - Using RawTileGrids to render only changed tiles.
      - Minimizing changed tiles with wrapped offset rendering.
    - Improving map draw performance with chunked drawing.
  - *User interface*
    - Keeping controls simple with judicious use of menus.
    - Drawing and handling input for menus and dialogs.
    - Changing graphics and zoom options in real-time.
    - Inventory and equipment shortcut menus.
  - *Word wrapping*
  - *Managing data* (Entity life cycle)
    - The world, and which entities exist and when.
    - Despawning entities, including entities they refer to.
  - *Save and load*
    - The data to save: uniques and components.
    - The save file format.
    - Handling of EntityIds during saving and loading.
    - Run-length encoding, used to compress map tile data and field of view bits.
  - *Field of view*
  - *Pathfinding using A-star*
    - Fallback path target, similar to NetHack.
    - Tweaking the heuristic to make monsters line up cardinally with the player.
  - *Random number generator*
    - No global RNG.
    - Instead, hash game seed, input values and magic number to seed termporary RNGs.
    - Makes testing way easier, especially map generation (used to fix diff map gen between native vs. web).
  - *Map generation*
    - Prim's algorithm for connecting rooms.
    - Monster and item spawning.
    - Place downstairs or spawn victory item?
  - *Auto-run*
    - Idea: rotate world, check tiles, pick direction, unrotate world.
    - Checked tile patterns are hard-coded, but work well in practice.
    - Integrated to act like real player inputs with additional checks to interrupt it, e.g. monster appears or player presses a key.
  - *Turn order and combat*
    - The damage formula.
    - Avoiding zero damage using tanh.
  - *Items*
  - *Hunger and regeneration*
    - Stomach component and the basics of hunger: ticks down over time, fills when eating, regen when above threshold, starving when zero.
    - Sub-HP and turns to regen to max HP.
    - Implication: higher max HP means faster regen.
  - *Experience and difficulty progression*
    - Experience formula
    - The level factor, which determines everything else.
    - The difficulty tracker, which determines the spawn level of monsters and equipment.
  - *New game plus*
    - Increases monster and item spawns in rooms per win.
    - Resetting difficulty, but still spawning more powerful equipment.
    - Safety save before using victory item, and why.