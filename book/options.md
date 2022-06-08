# Options

RuggRogue provides the player with an *options dialog* that can be brought up by either pressing the Esc key during play or choosing "Options" at the title screen.
This dialog allows the player to choose:

- whether the map uses a graphical tileset or an ASCII display with one of the fonts
- the font of the user interface (i.e. the sidebar and menus)
- 1x or 2x zoom for the map
- 1x or 2x zoom for the user interface

The game offers two fonts: the 8-by-8 pixel Terminal font and the 8-by-14 pixel [GohuFont](https://font.gohu.org/).
The graphical tileset available for the map is a monocolor version of the [Urizen OneBit Tilesets](https://vurmux.itch.io/urizen-onebit-tilesets) by vurmux.

If the options dialog is brought up while playing the game, it presents an option to save the game and exit back to the title screen.

## Options Data

The data for options exists in the form of the `Options` struct near the top of the `src/ui.rs` file:

```rust,ignore
pub struct Options {
    pub tileset: u32,
    pub font: u32,
    pub map_zoom: u32,
    pub text_zoom: u32,
}
```

The `tileset` and `font` fields decide which of the fonts or tilesets are used when drawing the map and the user interface respectively.
The game treats fonts as tilesets that can only display characters, so these fields are essentially indexes into a single list of tilesets.
The game makes an effort to limit the `font` field to only the two 'font-like' tilesets, as will be explained later.

The `map_zoom` field is the numeric zoom factor for the map display that can be toggled between 1x and 2x zoom.
The `text_zoom` field serves the same purpose but for the user interface instead.

The default values of these options are set all the way back in the `main` function in the `src/main.rs` file, like so:

```rust,ignore
world.add_unique(Options {
    tileset: 2,
    font: 0,
    map_zoom: 1,
    text_zoom: 1,
});
```

The above default values display the map using the Urizen graphical tileset, the user interface using GohuFont, and show them both at 1x zoom.

The numeric indexes of the `tileset` and `font` fields refer to the tilesets loaded further down in the `main` function:

```rust,ignore
let settings = RunSettings {
    // ...
    tileset_infos: vec![
        TilesetInfo::<GameSym> {
            image_path: PathBuf::from("assets/gohufont-8x14.png"),
            tile_size: (8, 14).into(),
            tile_start: (0, 0).into(),
            tile_gap: (0, 0).into(),
            font_map: TilesetInfo::<GameSym>::map_code_page_437(),
            symbol_map: HashMap::new(),
        },
        TilesetInfo::<GameSym> {
            image_path: PathBuf::from("assets/terminal-8x8.png"),
            tile_size: (8, 8).into(),
            tile_start: (0, 0).into(),
            tile_gap: (0, 0).into(),
            font_map: TilesetInfo::<GameSym>::map_code_page_437(),
            symbol_map: HashMap::new(),
        },
        gamesym::urizen_tileset_info(),
    ],
};
```

In order, these are GohuFont, the Terminal font and the Urizen graphical tileset, referred to by the `tileset` and `font` fields of the `Options` struct as 0, 1 and 2 respectively.
Note that the fonts are listed before the tileset; this fact is exploited by the options dialog to limit `font` customization to only the fonts.

## The Options Dialog

The options dialog is represented by the `OptionsMenuMode` that lives in the `src/modes/options_menu.rs` file.
This dialog allows the player to view and change the game options to suit their preferences.

There's a menu item labelled "Back" at the bottom of the dialog that dismisses it when chosen.
If the options dialog is brought up in the middle of a game, it will read "Save and Exit" instead, and dismissing it will save the game and return the player to the title screen.
The flag that controls this is the `prompt_to_save` boolean argument sent to the `OptionsMenuMode::new` function when the dialog is created.

Pressing the left and right keys alters the values of the various options; this takes place in the `OptionsMenuMode::update` function.
The "Font" option that controls the user interface font is limited to only fonts by being checked against the `NUM_FONTS` constant near the top of the `src/modes/options_menu.rs` file.
It's currently hard-coded to be `2`; adding more fonts would require updating this value accordingly.

## Real-Time Options Updates

If you mess with the options a bit, you'll notice that changes to the options are reflected immediately on the screen.
The rendering system consults the option values directly, so they'll update in real time.
To understand how this happens, we need to recap some concepts from the [Rendering](rendering.md) chapter:

Concept | Description
------- | -----------
Displayed tile grid | The physical appearance of a tile grid on the screen.
`TileGridView` | The size, position and zoom factor associated with a tile grid.
Tile grid texture | The GPU-side texture loaded with the pixels representing a tile grid.
Tile grid pixel buffer | The CPU-side buffer of the tile grid pixels, uploaded to the GPU-side texture.
`tileset_index` field of `TileGrid` | Index of the tileset that the tile grid should be rendered with.
`TileGrid` tile data | The logical cells of the tile grid.

Most of the data above depend on other data in the table.
The dependencies are as follows:

- Displayed tile grid
  - `TileGridView`
  - Tile grid texture
    - Tile grid pixel buffer
      - `tileset_index`
      - Tile data

Changes to the tileset and font in the options dialog affect the `tileset_index`.
This invalidates the *tile grid pixel buffer*, *tile grid texture* and *displayed tile grid*.
Changes to the `tileset_index` of tile grids are made through the `TileGrid::set_tileset` function in the `src/lib/tilegrid.rs` file that ensures that all this invalidation takes place.
The `TileGrid::display` function ensures that any stale data is updated, thus enabling real-time option updates.

Changes to the zoom options affect the `TileGridView` of different tile grids in the game.
This invalidates the *displayed tile grid*.
For "Font" zoom, the `text_zoom` field of the `Options` struct is directly read out by the various `prepare_grids` functions of most modes.
It is then used to calculate the size, position and zoom factor for the `TileGridView` of each `TileGrid`.
The common case of a tile grid centered in the screen is handled by the `TileGrid::view_centered` function, filling in the `TileGridView` with just a single call.

The `map_zoom` field of the `Options` struct is specifically read out by the `ChunkedMapGrid::prepare_grid` function in the `src/chunked.rs` file.
This function performs special sizing and positioning to ensure that the play area is covered and that there's a single center tile for the camera focus
The `map_zoom` field impacts the pixel size of the tile grid cells on screen and thus must be accounted for here.

Both preparing and displaying of tile grids must be done every single frame in response to potential window resize events, so option changes accounted for by these processes will occur in real time.
