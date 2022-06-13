# Rendering

This chapter is about how RuggRogue uses SDL to display things on the screen.
This will cover the game's overall rendering strategy, the use of tile grids to organize what's displayed, tilesets, the overarching phases involved in displaying things and finally a couple of strategies to improve drawing performance.

## Rendering Strategy

For a 2D tile-based roguelike game such as RuggRogue, there are two common strategies to getting pixels onto the screen when considering modern video hardware.
There's *software rendering*, where the CPU decides what each pixel on screen should look like, then uploads the whole screen as a texture for the GPU to ultimately display.
This is simple, but slow, especially considering that the CPU also has to deal with game logic.
On the other hand, there's the *hardware rendering* approach using *texture atlasses*, where tilesets are uploaded for the GPU in advance and the CPU feeds data buffers that describe where said tiles should be displayed on screen for the GPU.
This is much faster than the software approach, but a fair bit more complex, on top of of requiring an API that can create graphics pipelines with custom shaders.

Instead of either of those things, RuggRogue adopts a *hybrid rendering* strategy that combines elements of both software and hardware rendering.
The game breaks down what it wants to display on screen into tile grids whose contents are rendered by the CPU in a software rendering fashion.
It then uses SDL's drawing API to arrange those tile grids on screen, which is aware of video hardware and thus makes up the hardware rendering portion of the strategy.

So why hybrid rendering?
Early in development, RuggRogue rendered its grids by drawing the background of each cell as a colored rectangle, followed by the contents of the cell in the foreground color.
This approach was unplayably slow, to the point that transitioning to pure software rendering was a noticeable performance improvement.
The source of this slowness is the approach to drawing used by many 2D graphics libraries and game engines that advertise "hardware-accelerated graphics".
These libraries and engines provide ad-hoc drawing APIs where, at any time, the CPU can tell the GPU to draw something.
This approach is known as *immediate mode drawing*, which is impossible for accelerated graphics hardware to handle quickly.
Any 2D drawing API that doesn't use or allow the creation of a graphics pipeline to feed data into shaders will be slow in this way.
SDL's bundled drawing API, like many others, falls in this performance pit.
SDL provides an OpenGL escape hatch, but OpenGL is its own can of worms that I didn't feel like dealing with when I just wanted to get things on screen fast.

SDL's immediate mode drawing approach isn't useless though.
SDL's drawing API in general avoids the complexity of graphics APIs that require the creation of full-blown graphics pipelines, shader compilation and linking.
It also only really suffers performance issues when it's used to draw a lot of small things; it's actually quite fast when drawing a few large things instead.
In RuggRogue, these large things are tile grids, and this is how the hybrid rendering strategy came to be.
It's probably not as fast as a proper graphics pipeline, but it's much simpler to put together.
It also separates the drawing of the contents of a tile grid from where and how it's shown on screen, which makes drawing and arranging tile grids on screen easier.

Just before diving into the rest of this, there a couple of SDL-specific things to keep in mind.
SDL's drawing API stores image data in two ways: surfaces and textures.
An SDL surface is an image buffer in RAM that the CPU can render into.
An SDL texture is image data that exists in video hardware that the GPU can sample from and display on screen.
RuggRogue uses both of these things, typically by rendering into an SDL surface and uploading its contents into an SDL texture for display.

## Tile Grids

As you may have guessed, a tile grid is a grid of tiles (surprise).
Everything that RuggRogue displays is a tile grid, from the main gameplay screen and sidebar to the smallest dialog.
There are three parts to the tile grid system: tile grid layers that hold tile grids, tilesets and the tile grids themselves.

### Tile Grid Layers

`TileGridLayer` can be found at the bottom of `src/lib/tilegrid.rs`.
Its definition is rather boring: a vector of tile grids along with a drawing-related flag.
What matters more is its purpose and usage.
A `TileGridLayer` represents all of the tile grids that belong to a specific mode in the mode stack.
The main game loop (in the `run` function in `src/lib/run.rs`) composes tile grid layers into a vector stored in the `layers` variable.
The mode stack (at the bottom of `src/modes/mod.rs`) receives the `layers` vector and is responsible for ensuring that each of its modes gets a corresponding `TileGridLayer`.
The mode stack also sets the `draw_behind` flag that the main game loop later uses to determine the bottom-most visible `TileGridLayer` whose tile grids should be displayed on screen.
In this way, the `layers` variable in the main game loop is effectively the game's scene graph.

### Tilesets

Tilesets are represented by the appropriately-named `Tileset` struct that can be found towards the top of `src/lib/tilegrid.rs`.
Tilesets hold image data representing individual tiles that are drawn into tile grid buffers.
Tilesets perform three main tasks:

1. Loading and storing tile image data for quick rendering later.
2. Converting tile image data into grayscale to enable recoloring.
3. Rendering tile image data onto surfaces provided by tile grids themselves.

Loading of a tileset is controlled by the `TilesetInfo` struct near the top of `src/lib/tilegrid.rs`.
They can be seen in action in the `main` function in `src/main.rs`, where they define basic font and tileset information for loading.
The definition of `TilesetInfo` is well-commented, but the `font_map` and `symbol_map` are worth mentioning.
They hold the information needed for tilesets to translate their logical grid cell contents into tile images.

Tile image data is prepared specifically to facilitate fast rendering.
The tileset image is reduced down to only the tiles that the `font_map` and `symbol_map` refer to; all other tile image data is discarded after the loading process is done.
In addition, the surviving tile images are arranged as a one-tile-wide surface.
Image data is stored in row-major order, so this arrangement ensures that the different pixel rows of each tile image are as closely packed as possible to minimize cache misses while rendering.

To support recoloring, tile image data is stored in grayscale.
The grayscaling process occurs in the `Tileset::transfer_tiles` function.
A "grayness" value is calculated from the input image data.
A "grayness" of zero is set to transparent black, while anything else is set to white with "grayness" serving as alpha.

The job of rendering tile image data onto a surface is done by the `Tileset::draw_tile_to` function.
If you read the source code of this function, you'll notice references to `CellSym` and `text_fallback`.
`CellSym` will be covered in the next section.
`text_fallback` refers to the `Symbol::text_fallback` function all the way at the top of `src/lib/tilegrid.rs`.
`Symbol` itself is a Rust trait, and the purpose of `text_fallback` is to provide a font alternative to a game symbol if it doesn't define a graphical tile image.
RuggRogue's fallbacks can be found in the `GameSym::text_fallback` function in `src/gamesym.rs`, which is the `text_fallback` function inside the `impl Symbol for GameSym` block if you're not used to reading Rust syntax.
Apart from symbol fallback handling, `Tileset::draw_tile_to` recolors tiles using the SDL-provided `set_color_mod` function that multiplies the foreground color with the grayscaled tile image data from before.
The rendering proper is handled by calling the SDL-provided `blit` function, which performs surface-to-surface software rendering.

### Tile Grids

The `TileGrid` struct defined around the middle of `src/lib/tilegrid.rs` covers a lot of different responsibilities.
It probably has too many responsibilities and I'd consider splitting it up if I were to do this again, but endless refactoring also means endless game development cycles.
So keep in mind that what I'm about to describe is the state I left it in when I decided that the game eventually needed to be finished and is not necessarily an ideal design.
I'll briefly touch on all of the key players involved before covering how stuff works.

A `TileGrid` consists of a few major parts:

* Two `RawTileGrid` structs: one holds the logical contents of the grid, while the other is the same data from the previous frame.
* An SDL surface `buffer` that holds the grid contents rendered with the tile grid's associated tileset.
* An SDL texture that serves as the GPU-side destination of the contents of that buffer.
* A `TileGridView` that describes where and how the tile grid should appear on the screen.
* A bunch of dirty flags to avoid redoing work that isn't needed.

The `RawTileGrid` is defined above the `TileGrid` struct in `src/lib/tilegrid.rs`.
It holds a grid of `Cell`s (itself defined above `RawTileGrid` and not to be confused with the Rust standard library struct) that consist of a `CellSym`, a foreground color and a background color.
The role of `CellSym` (defined above `Cell`) is to hold either a character or a symbol.
This symbol is a non-library type that implements the `Symbol` trait whose definition is a the top of `src/lib/tilegrid.rs`.
The `Symbol` trait is implemented on the game side by the `GameSym` enum that can be found in `src/gamesym.rs`.
The purpose of `GameSym` is to provide distinct symbolic names to tile appearances, such as "Player", "Ration", "DownStairs" or "WallNe" (north-east wall corner).
This allows drawing code to use these symbolic names to represent tile appearances in a flexible manner.

`TileGridView` is defined just above `TileGrid` in `src/lib/tilegrid.rs`.
It holds the position, size and offset within a bounding box in which its `TileGrid` owner will be clipped.
The `color_mod` field alters the color of the whole tile grid at display time, which is mainly used to dim tile grids associated with inactive background modes.
It also holds an integer zoom factor that the options menu can alter to zoom the map and the user interface.

## The Rendering Process

Up until now, I've been using the terms "draw", "display" and "render" rather loosely.
To make this process easier to understand, I'll switch to these specific terms that describe how a tile goes from being drawn to displayed on screen, in this order:

1. **Draw**: Plotting of a character or symbol into a tile grid cell with foreground and background colors.
2. **Render**: Combining tile grid cells and colors with tileset data to produce a pixel appearance in a tile grid's buffer.
3. **Upload**: Uploading a tile grid's rendered buffer into the GPU-accessible texture associated with the tile grid.
4. **Display**: Putting the tile grid on the screen using the information in the `TileGridView`.

**Drawing** is the first thing that happens when the game wants something to appear on screen.
Drawing happens through the public functions of `TileGrid`, such as `TileGrid::print` and `TileGrid::put_sym_color`.
These functions are called from the `draw` functions of modes that can typically be found at the bottom of any of the files in the `src/modes/` directory.
Map drawing specifically occurs near the bottom of `src/chunked.rs`; a file that is covered in its own section a bit later.
Entity drawing happens in the deceptively-named `src/render.rs` file that, despite its name, only handles entity 'drawing' and not 'rendering' in these terms.
The `TileGrid` drawing functions dispatch to similar functions in `RawTileGrid` that perform the actual drawing by setting the cell (a character or symbol) along with its foreground and background colors.

**Rendering** is how cells are turned into pixel data.
This happens in the `TileGrid::render` function that is called near the top of `TileGrid::display` just before that function goes about its business.
Recall the two `RawTileGrid`s in `TileGrid`.
The `front` tile grid holds what the game logic has drawn to the tile grid, while the `back` tile grid is the same data but from the previous frame.
The `TileGrid::render` function renders a logical tile into its corresponding buffer location only if the same cell in the `front` and `back` grids are visibly different.
If a tile doesn't change, it doesn't get rendered.
The end of `TileGrid::render` updates the back grid with the contents of the front grid in preparation for the next frame.

**Uploading** occurs after rendering to update the contents of the tile grid's GPU-side texture with its CPU-side rendered buffer.
This is the `texture.update(...)` part of the `TileGrid::display` function, provided by SDL.

**Displaying** the uploaded tile grid texture is, unsurprisingly, the job of the `TileGrid::display` function.
The main loop all the way over in `src/lib/run.rs` goes through all of the tile grid layers in its `layers` vector, and then calls this function on each tile grid in each layer.
The majority of the `TileGrid::display` function is dedicated to calculating where and how the tile grid should appear and calling `canvas.copy(...)` to put the tile grid texture on screen.
This is what happens in the straightforward case, but if you read the code in this function you'll notice there's a lot more going on.
Why are there four separate calls to `canvas.copy`?
In order to understand this, I'm going to need to go into the technique I've used here that I call "wrapped offset rendering".

## Improving Rendering Performance with Wrapped Offset Rendering

When playing RuggRogue, the camera is generally always centered on the player, so when the player moves, the entire view of the map shifts accordingly.
Consider the following scenario.
If the player moves one tile to the right, the player is drawn stationary in the center tile of the tile grid while the entire dungeon is drawn one tile over to the left.
According to our tile grid rendering strategy, the player's tile is in the same place, so they won't need to be rendered again.
However, every single dungeon tile has shifted, so any tile that wasn't next to an identical tile will need to be rendered again, even though it's only the player that really moved.
Rendering multiple shifted dungeon tiles versus a single player tile seems pretty inefficient.
There must be some way to render only the player and their immediate surroundings while avoiding the need to render most of the visible dungeon tiles again.

What if, when the player moves one tile to the right, we *offset* all drawing one tile to the right internally as well?
This would normally cause drawing in the far right cell columns to overflow, so we need to *wrap* them over to the now-unused left cell columns instead.
The rendering phase will pick up that the player has moved one tile to the right, while the dungeon map remains stationary.
But the whole point of drawing the player at the center of the screen is, well, to have them centered.

This is where we get clever.
At display time, we *undo* the offset that was set when drawing, so the player that was drawn a tile over to the right is shifted a tile *back* to the left, thus recentering everything.
The wrapped column of cells that was drawn in the left column can then be split off and displayed over on the right side, where they were originally intended to be.
Presto!
We only had to render the player and immediate surroundings again, while the rest of the dungeon tiles can be skipped during rendering.
It is this central idea that underpins what I call *wrapped offset rendering*.

As you can probably guess from the example, wrapped offset rendering is used to reduce the number of dungeon tiles that need to be rendered when the player moves around.
The tile grid representing the dungeon map is given the player's position via its `TileGrid::set_draw_offset` function, which immediately passes it over to `RawTileGrid::set_draw_offset`, since the `RawTileGrid` handles drawing.
The `RawTileGrid::index` function underpins how all drawing functions 'see' the grid cell storage, and this is where the offset and wrapping are applied to affect drawing operations.
Meanwhile, the rendering process is blind to all of this offset business and renders whatever it sees.

This sets the stage for understanding why `TileGrid::display` calls `canvas.copy` (up to) four separate times.
All of the calculations in the lower half of `TileGrid::display` are to undo the wrapped offset rendering to display everything in the right place.
Only one `canvas.copy` call is needed if the offset is `(0, 0)`.
Two `canvas.copy` calls are needed if exactly one of the x-axis or the y-axis have a non-zero offset.
Finally, four `canvas.copy` calls are needed if both the x-axis and y-axis have non-zero offsets.
These additional calls take wrapped rows and columns of the grid and put them on the opposite side of the tile grid at display time, all in the name of reducing tile rendering.

If you're considering using this wrapped offset rendering technique in your own projects, there's a couple of points to keep in mind.
First, this is probably only really effective for software rendering and not hardware rendering, since graphics hardware renders all pixels every frame anyway.
Second, this approach won't work as well if tiles are constantly changing, like if they're being animated.
It is the unique combination of partial software rendering, a player-centric camera and a mostly static dungeon that makes wrapped offset rendering an effective performance-improving technique for RuggRogue.

In order for wrapped offset rendering to work, it needs an appropriate offset.
This happens early on in the `ChunkedMapGrid::draw` function in the `src/chunked.rs` file, which feeds the top-left corner of the top-left map chunk on screen into `TileGrid::set_draw_offset`, and all is well.

...

Wait, what's a "chunk"?

## Improving Map Drawing Performance with Chunked Drawing

It turns out that the performance rabbit hole goes even deeper than mere wrapped offset rendering.
Some time after I had finished work on getting wrapped offset rendering into a functioning state, I found myself profiling the web version of the game.
Performance still wasn't great at this time, and I wanted to know why.
What I saw in the profile data stuck out to me: *drawing* of the map was dominating execution time.
Not rendering, where all the pixels of each tile have to be handled, but just deciding what tiles were going to look like to begin with?
Well, I suppose that's what happens when you optimize a bottleneck: it moves elsewhere, and here it moved from rendering to map drawing.
It's worth noting that RuggRogue has a resizable window, but it doesn't stretch or zoom its contents to fit the window size.
Instead, it adds or removes space to accommodate more or less tiles.
In other words, if RuggRogue's window is very large, the game has to draw *a lot* of tiles, and the web version of the game was struggling with this.
In fact, it wasn't even strictly map drawing that was the bottleneck, the sheer number of tile grid cells that could potentially *show* a map tile was the performance bottleneck.
I had to do something to reduce the number of tile grid cells that had to be considered when drawing the map.

So I made a deal with the Programming Devil: I traded code simplicity for performance by pursuing a [dirty rectangles](https://wiki.c2.com/?DirtyRectangles) approach to map drawing.
The idea here is that instead of deciding the appearance of every single cell in the map tile grid every single frame, I'd divide the map tile grid into *chunks* of 8-by-8 tiles that would be drawn once and would only be revisited on request.
Therefore, the tile grid associated with the map is the only tile grid in the game whose contents are not fully redrawn every single frame.
I refer to this approach as *chunked map drawing*.

The entirety of the chunked map drawing implementation can be found in the aptly-named `src/chunked.rs` file.
The most important part of that file is the `ChunkedMapGrid` struct that contains all of the book-keeping and logic required to make chunked map drawing a reality.
The `screen_chunks` field of `ChunkedMapGrid` is a vector of small `ScreenChunk` structs.
The data in the `ScreenChunk` is the chunk of the map to be drawn, while the position of each `ScreenChunk` in the vector implies its position on the screen, i.e. the map tile grid.

In order to maximize performance, we want to avoid the need to constantly redraw partial chunks near the edges of the screen whenever the camera moves.
Therefore, we must maintain a map tile grid whose dimensions are a whole multiple of the chunk size (i.e. 8 tiles) and is big enough to cover the available screen space given to it.
These calculations are performed in the `ChunkedMapGrid::prepare_grid` function, and the results are stored in the `new_chunks_across` and `new_chunks_down` variables.
Special care is taken to ensure the width and height of this grid is strictly greater than its screen dimension if the screen width or height is exactly a multiple of the chunk pixel dimensions to enable shifting.

Now that we have a map tile grid whose bounds exceed the screen space, we need to ensure that the display of the grid itself is shifted so that the camera is centered on screen.
There are two values we need to know in order to calculate how much to shift the map tile grid by:

1. the pixel width of the screen, halved
2. the x-value of the central pixel of the tile relative to the left edge of the map tile grid

The latter value is the sum of the pixels between the central chunk and the left edge, and the pixels between the center of the camera tile and the left edge of the central chunk in which it resides.
The difference between those two values is computed early in the `ChunkedMapGrid::draw` function and stored in `grid.view.dx` in order to shift the grid the correct amount.
A similar process is used to fill `grid.view.dy` as well, substituting "x" with "y" and "width" with "height".

With size and position sorted, the next thing to work out is which screen chunk shows which map chunk.
This is the job of the `screen_chunks` field of the `ChunkedMapGrid` struct.
This is a vector of `ScreenChunk` structs that holds metadata for each chunk of the map tile grid that needs to be filled in.
Screen chunks are stored in this vector in row-major order, so `0` is the top-left 8-by-8 chunk of grid cells, `1` would be next to it on the right, and so on.
The `map_chunk` stores the 8-by-8 chunk of map tiles that the screen chunk should be showing.
Map chunks are stored as pairs of integers, but the idea is the same as for screen chunks, except representing map chunks instead, so `(0, 0)` is the top-left 8-by-8 chunk of map tiles, `(0, 1)` is to the left, `(1, 0)` is below, and so on.

The screen chunks are filled with map chunk data by figuring out which map chunk the top-left screen chunk should be showing, and populating the other screen chunks from there.
This is the task of the `ChunkedMapGrid::screen_top_left_map_chunk` function.
It takes the tile position of the camera on the map, and subtracts half a screen's width- and height-worth of tiles from it; whatever map chunk it lands in is assigned to be shown in the top-left screen chunk.

Each screen chunk is accompanied by a `dirty` flag.
When map chunks are assigned to a screen chunk, the new map chunk value is compared against the existing value remembered by the screen chunk.
If a change is detected, the dirty flag is set.
The presence of the dirty flag triggers the final checking and drawing of map tiles onto the tile grid on screen.
Map rendering is minimized by feeding the top-left tile of the top-left screen chunk into the `TileGrid::set_draw_offset` function, tying into the wrapped offset rendering described in the previous section.
All of this work is done by the `ChunkedMapGrid::draw` function.

If we stopped here, we'd only be redrawing chunks of the map as they enter from the edges of the screen.
We still need to handle the player, whose actions cause the contents of the map grid in their field of view to change pretty much every single turn.
When the player, or really the camera, moves around the map, the `ChunkedMapGrid::mark_dirty` function sets the dirty flags of the corresponding screen chunks.
When the player descends into a new map, the `ChunkedMapGrid::mark_all_dirty` function sets the dirty flags of every screen chunk.
These calls are made in the `DungeonMode::update` function after it performs most of its logic.

## Wrap Up

Whew, I think that's everything.
As you can see, rendering is a simple process involving hybrid rendering, tile grids, tile grid layers, tilesets, raw tile grids, surfaces, textures, tile grid views, cells, cellsyms, symbols, drawing, rendering, uploading, displaying, wrapped offset rendering and chunked map drawing.

...

Okay, maybe it's not so simple.
So after all of that, how's the performance of the game?
The performance of the web version is... passable.
It's far better than the initial version, but I can't improve it any further short of rethinking the web port from the ground-up.
In contrast, the native build of the game is butter smooth, both the debug build but especially the release build.
I'm really happy about how this all turned out.

If you've read this far, congratulations.
`src/lib/tilegrid.rs` is the longest source code file in the game, and `src/chunked.rs` is probably the most complicated.
Future chapters shouldn't be anywhere near as complicated as this one.
Hopefully this gives an idea about how 2D tile grid rendering works overall, and some hints about what's involved in pulling it all together.
