# Saving and Loading

RuggRogue features a pretty basic save system.
When the player chooses to save and exit from the options menu, all game data is written into a save file.
The title screen will detect the presence of this file and show an option to load the game.
The game also auto-saves at a couple of other points, such as when the player takes the stairs, and when they're about to win the game.
If the player dies, any detected save file is deleted.
If the player chooses to start a new game and a save file exists, a prompt will appear to delete it first.

All of this save-and-load action happens in the fittingly-named `src/saveload.rs` file, which will be the focus of most of this chapter.

## The Save File Format

When the game is saved, save data will be written to a file named `savegame.txt` in the same directory as the game itself.
This file is in plain text format where each line represents either a unique or a component, made up of three tab-separated fields of data.
Each unique line consists of an asterisk character, the type name of the unique and the unique data.
Each component line consists of the ID of the entity it belongs to, the type name of the component and the component data.

Here is an example of the contents of a small, complete save file:

```plaintext
*	GameSeed	9542716676452101438
*	TurnCount	10
*	Wins	0
*	BaseEquipmentLevel	0
*	Difficulty	{"id":[8,0],"exp_for_next_depth":40}
*	Messages	{"capacity":100,"msg_queue":["This is a test save!"],"num_highlighted":0}
*	PlayerAlive	true
*	PlayerId	[5,0]
*	Map	{"depth":1,"width":80,"height":50,"tiles":[["W",1952],["F",8],["W",72],["F",8],["W",72],["F",8],["W",72],["F",8],["W",72],["F",8],["W",72],["F",8],["W",72],["F",8],["W",72],["F",8],["W",1760]],"rooms":[{"x1":32,"y1":24,"x2":39,"y2":31}],"seen":{"width":80,"height":50,"bv":[[1,4000]]}}
[2,0]	BlocksTile	null
[3,0]	CombatBonus	{"attack":0.0,"defense":1.4}
[4,0]	CombatBonus	{"attack":3.2,"defense":0.0}
[5,0]	CombatStats	{"max_hp":40,"hp":40,"attack":4.8,"defense":2.4}
[2,0]	CombatStats	{"max_hp":14,"hp":14,"attack":8.0,"defense":4.0}
[6,0]	Consumable	null
[7,0]	Consumable	null
[5,0]	Coord	{"x":33,"y":25}
[7,0]	Coord	{"x":33,"y":30}
[2,0]	Coord	{"x":38,"y":30}
[6,0]	Coord	{"x":38,"y":25}
[3,0]	EquipSlot	"Armor"
[4,0]	EquipSlot	"Weapon"
[5,0]	Equipment	{"weapon":[4,0],"armor":[3,0]}
[5,0]	Experience	{"level":1,"exp":0,"next":50,"base":0}
[8,0]	Experience	{"level":1,"exp":0,"next":50,"base":0}
[2,0]	Experience	{"level":1,"exp":0,"next":0,"base":0}
[5,0]	FieldOfView	{"tiles":{"width":17,"height":17,"bv":[[0,108],[1,10],[0,7],[1,10],[0,7],[1,10],[0,7],[1,10],[0,7],[1,10],[0,7],[1,10],[0,7],[1,10],[0,7],[1,9],[0,8],[1,9],[0,8],[1,7],[0,21]]},"range":8,"center":[33,25],"dirty":false}
[2,0]	FieldOfView	{"tiles":{"width":17,"height":17,"bv":[[0,21],[1,7],[0,8],[1,9],[0,8],[1,9],[0,7],[1,10],[0,7],[1,10],[0,7],[1,10],[0,7],[1,10],[0,7],[1,10],[0,7],[1,10],[0,7],[1,10],[0,108]]},"range":8,"center":[38,30],"dirty":false}
[2,0]	GivesExperience	10
[5,0]	Inventory	{"items":[]}
[3,0]	Item	null
[4,0]	Item	null
[6,0]	Item	null
[7,0]	Item	null
[2,0]	Monster	null
[6,0]	Name	"Health Potion"
[7,0]	Name	"Ration"
[5,0]	Name	"Player"
[3,0]	Name	"+1 Jerkin"
[4,0]	Name	"+1 Knife"
[2,0]	Name	"Blob"
[7,0]	Nutrition	750
[5,0]	Player	{}
[6,0]	ProvidesHealing	{"heal_amount":15}
[7,0]	RenderOnFloor	null
[6,0]	RenderOnFloor	null
[5,0]	RenderOnMap	null
[2,0]	RenderOnMap	null
[6,0]	Renderable	{"sym":"HealthPotion","fg":{"r":255,"g":0,"b":255},"bg":{"r":0,"g":0,"b":0}}
[7,0]	Renderable	{"sym":"Ration","fg":{"r":191,"g":92,"b":0},"bg":{"r":0,"g":0,"b":0}}
[5,0]	Renderable	{"sym":"Player","fg":{"r":255,"g":255,"b":0},"bg":{"r":0,"g":0,"b":0}}
[3,0]	Renderable	{"sym":"Jerkin","fg":{"r":170,"g":97,"b":32},"bg":{"r":0,"g":0,"b":0}}
[4,0]	Renderable	{"sym":"Knife","fg":{"r":165,"g":165,"b":165},"bg":{"r":0,"g":0,"b":0}}
[2,0]	Renderable	{"sym":"Blob","fg":{"r":89,"g":162,"b":191},"bg":{"r":0,"g":0,"b":0}}
[5,0]	Stomach	{"fullness":1491,"max_fullness":1500,"sub_hp":0}
[5,0]	Tally	{"damage_dealt":0,"damage_taken":0,"kills":0}
```

If you have a native build of RuggRogue, you can copy and paste this into a file named `savegame.txt` and the game will load it.
The above save data contains the player, a monster and some items confined in a small enclosed room in the center of the map.
We can break down this example save data, starting with the top lines:

```plaintext
*	GameSeed	9542716676452101438
*	TurnCount	10
*	Wins	0
*	BaseEquipmentLevel	0
*	Difficulty	{"id":[8,0],"exp_for_next_depth":40}
*	Messages	{"capacity":100,"msg_queue":["This is a test save!"],"num_highlighted":0}
*	PlayerAlive	true
*	PlayerId	[5,0]
*	Map	{"depth":1,"width":80,"height":50,"tiles":[["W",1952],["F",8],["W",72],["F",8],["W",72],["F",8],["W",72],["F",8],["W",72],["F",8],["W",72],["F",8],["W",72],["F",8],["W",72],["F",8],["W",1760]],"rooms":[{"x1":32,"y1":24,"x2":39,"y2":31}],"seen":{"width":80,"height":50,"bv":[[1,4000]]}}
```

All of these lines represent uniques, since they all start with an asterisk character.
There's some basic data for uniques such as `GameSeed`, `TurnCount`, `Wins` and `PlayerAlive` whose data should hopefully be self-explanatory.
Looking at some of the other lines reveals that all data is serialized in JSON format.

The line for the `Map` unique is interesting here.
The `"tiles"` field stores the contents of each tile in the map: `"W"` is a wall, `"F"` is a floor and `"D"` would be a downstairs tile that isn't featured in this tile data.
Tiles have a lot of redundancy, so they're stored in a special compressed form that will be covered later in this chapter.
Even with compression, this `Map` line will often be a lot longer than this in a typical save file.

You may have noticed the pairs of numbers present in the lines for the `Difficulty` and the `PlayerId` uniques.
These are entity IDs, so each pair uniquely identifies a entity.
Here, the difficulty tracker is represented by the `[8,0]` entity, while the player is represented by the `[5,0]` entity.

We can peek at the component data for the difficulty tracker by singling out lines of component data starting with the `[8,0]` entity ID, of which there is just one:

```plaintext
[8,0]	Experience	{"level":1,"exp":0,"next":50,"base":0}
```

As we can see, the difficulty tracking entity has a single `Experience` component that is at level 1 and will advance to the next level once it gains 50 experience points.

Filtering for the player by looking at lines starting with `[5,0]` shows that they hold a lot more data:

```plaintext
[5,0]	CombatStats	{"max_hp":40,"hp":40,"attack":4.8,"defense":2.4}
[5,0]	Coord	{"x":33,"y":25}
[5,0]	Equipment	{"weapon":[4,0],"armor":[3,0]}
[5,0]	Experience	{"level":1,"exp":0,"next":50,"base":0}
[5,0]	FieldOfView	{"tiles":{"width":17,"height":17,"bv":[[0,108],[1,10],[0,7],[1,10],[0,7],[1,10],[0,7],[1,10],[0,7],[1,10],[0,7],[1,10],[0,7],[1,10],[0,7],[1,9],[0,8],[1,9],[0,8],[1,7],[0,21]]},"range":8,"center":[33,25],"dirty":false}
[5,0]	Inventory	{"items":[]}
[5,0]	Name	"Player"
[5,0]	Player	{}
[5,0]	RenderOnMap	null
[5,0]	Renderable	{"sym":"Player","fg":{"r":255,"g":255,"b":0},"bg":{"r":0,"g":0,"b":0}}
[5,0]	Stomach	{"fullness":1491,"max_fullness":1500,"sub_hp":0}
[5,0]	Tally	{"damage_dealt":0,"damage_taken":0,"kills":0}
```

The player's `CombatStats` show that they're at 40 out of 40 hit points, with a base attack value of 4.8 and a base defense value of 2.4.
According to the `Coord` component they are located at coordinates 33,25.
Like the difficulty tracker, the player has an `Experience` component.
Components such as `Name`, `Renderable`, `Stomach` and `Tally` contain basic data.

The `RenderOnMap` component is a tag component with no data, so its serialized to the JSON `null` value.
The `Player` component would be saved the same way, except it stores some runtime data that doesn't need to be saved, so it comes out as an empty JSON object instead.

The `FieldOfView` component represents which tiles in the immediate vicinity of the player should be visible.
It's a very long line out of necessity since each line must hold the full data of either a unique or a component.
The `"bv"` field within the top-level `"tiles"` field is compressed in the same way as the tiles of the map unique.

The `Equipment` and `Inventory` components typically contain entity IDs.
Here, the player's inventory is empty, but they are armed with a weapon and armor.
Here are the lines for the weapon entity, which is a "+1 Knife":

```plaintext
[4,0]	CombatBonus	{"attack":3.2,"defense":0.0}
[4,0]	EquipSlot	"Weapon"
[4,0]	Item	null
[4,0]	Name	"+1 Knife"
[4,0]	Renderable	{"sym":"Knife","fg":{"r":165,"g":165,"b":165},"bg":{"r":0,"g":0,"b":0}}
```

The player's armor is a "+1 Jerkin":

```plaintext
[3,0]	CombatBonus	{"attack":0.0,"defense":1.4}
[3,0]	EquipSlot	"Armor"
[3,0]	Item	null
[3,0]	Name	"+1 Jerkin"
[3,0]	Renderable	{"sym":"Jerkin","fg":{"r":170,"g":97,"b":32},"bg":{"r":0,"g":0,"b":0}}
```

There's a "Health Potion" that's on the floor at coordinates 38,25:

```plaintext
[6,0]	Consumable	null
[6,0]	Coord	{"x":38,"y":25}
[6,0]	Item	null
[6,0]	Name	"Health Potion"
[6,0]	ProvidesHealing	{"heal_amount":15}
[6,0]	RenderOnFloor	null
[6,0]	Renderable	{"sym":"HealthPotion","fg":{"r":255,"g":0,"b":255},"bg":{"r":0,"g":0,"b":0}}
```

Finally, there's an enemy "Blob" in the opposite corner of the room to the player:

```plaintext
[2,0]	BlocksTile	null
[2,0]	CombatStats	{"max_hp":14,"hp":14,"attack":8.0,"defense":4.0}
[2,0]	Coord	{"x":38,"y":30}
[2,0]	Experience	{"level":1,"exp":0,"next":0,"base":0}
[2,0]	FieldOfView	{"tiles":{"width":17,"height":17,"bv":[[0,21],[1,7],[0,8],[1,9],[0,8],[1,9],[0,7],[1,10],[0,7],[1,10],[0,7],[1,10],[0,7],[1,10],[0,7],[1,10],[0,7],[1,10],[0,7],[1,10],[0,108]]},"range":8,"center":[38,30],"dirty":false}
[2,0]	GivesExperience	10
[2,0]	Monster	null
[2,0]	Name	"Blob"
[2,0]	RenderOnMap	null
[2,0]	Renderable	{"sym":"Blob","fg":{"r":89,"g":162,"b":191},"bg":{"r":0,"g":0,"b":0}}
```

Monsters share a lot of common component types with the player, but they have a `Monster` tag component instead of a `Player` component.
The `BlocksTile` tag component marks that other monsters should take a path around this monster, rather than trying to go through it.

And that's it: a full run-down of a complete save file.
The format is pretty flexible: lines can technically occur in any order so long as every unique type is present.
Save data is considered valid as long as all entity IDs exist, and the existence of entities is implied by the ID values at the start of component lines.

## Saving

The `save_game` function is responsible for saving the game.
It's called from several other places in the code:

1. In the `use_item` function in the `src/item.rs` file when the player uses the victory item.
2. In the `DungeonMode::update` function in `src/modes/dungeon.rs` in response to:
    - confirming when closing the game (the `AppQuitDialogModeResult::Confirmed` case)
    - taking the stairs (the `YesNoDialogModeResult::Yes` case)
    - choosing to save and exit from the options menu (the `OptionsMenuModeResult::ReallyQuit` case)

The logic of the `save_game` function is simple: open a buffered writer for the `savegame.txt` file, write lines for all uniques and component storages, then flush the buffered writer.
Since this is Rust, the writer will automatically be closed when it falls out of scope at the end of the function.

The writing of a unique line is handled by the `save_named_unique` function, which outputs the asterisk, the unique type name and the unique data in tab-separated form.
Used as-is, it would normally appear like this in the `save_game` function:

```rust,ignore
save_named_unique<_, GameSeed>(world, &mut writer, "GameSeed")?;
```

To avoid having to specify the type name of the unique twice, the `save_game` function instead uses a helper macro named `save_unique!`, shortening the above to:

```rust,ignore
save_unique!(GameSeed, world, &mut writer)?;
```

While the `save_named_unique` function writes a single line for a unique, the `save_named_storage` function instead writes multiple lines for a given component type, one for each individual component.
Used as-is, it would look like this:

```rust,ignore
save_named_storage<_, AreaOfEffect>(world, &mut writer, "AreaOfEffect")?;
```

There's also a helper macro for this named `save_storage!` that shortens it to this instead:

```rust,ignore
save_storage!(AreaOfEffect, world, &mut writer)?;
```

That's all there is to saving the game.
You should appreciate the simplicity of this, because the loading logic is a lot more involved.

It's worth noting what *isn't* being considered here: entity IDs.
It turns out that entity IDs are not only serializable, but they're 100% safe to save as-is with no further intervention.
If game data were stored and managed with something like references or pointers instead, it would need an [unswizzling](https://en.wikipedia.org/wiki/Pointer_swizzling) strategy that would have to invade the data serialization logic.

## Loading

Saving is a relatively straightforward affair, free of branches and loops, with very simple error conditions.
Loading, on the other hand, is a lot more complex.
Part of this is due to how permissive the save file format is; in particular, lines for uniques and components can technically appear in any order and still be valid.
But a lot of this complexity comes from the fact that the very nature of loading involves setting up and altering a lot of data, which is something that the saving process never has to worry about.

The loading process can be broadly broken down into these major parts:

1. Load data for each line in the save file:
    - If it starts with an asterisk, load the line as a unique.
    - Otherwise, try to load component data and attach it to a specific entity, creating it if it doesn't exist yet.
2. Check that all uniques needed were loaded from the save file.
3. Fix entity IDs across all uniques and components that store them.
4. Place entities with `Coord` components on the map.
5. Commit the freshly-loaded uniques and entities to the world.
6. Despawn any entities that need to be despawned.

We'll look at each part one at a time.

### Handling Entity Despawning

This is not a mistake: we're looking at the last phase first.
If you take a look at the `load_game` function, you'll notice that despawning entities is all that it really does; most of loading logic is instead handled by the `load_save_file` function that it calls.
Why is it set up like this?

To understand the answer, we need to step back and think about what loading actually means in terms of data.
While we read in each line of the save file, we'll be loading components.
In order to load components, we need to create new entities to attach them to.
At this point, there will be entities in the world that existed before loading began alongside newly-created entities spawned in the process of loading.
If loading fails, these new entities need to be despawned so that we don't have half-loaded entities floating about in the world.
Likewise, if loading succeeds, old entities need to be despawned since they've been fully replaced by the loaded entities and are thus no longer needed.

The sole purpose of the `load_game` function is to give a blank list for the `load_save_file` function to fill with the IDs of entities that need to be despawned, and guarantee that they are despawned afterwards.
The `load_game` function is called from the `TitleMode::update` function in the `src/modes/title.rs` file when the player chooses to load a game from the title screen.
If the `load_save_file` function fails to load the game, this list will contain the newly-loaded entity IDs so that they can be cleaned up.
If it succeeds, this list will instead contain the IDs of old entities that weren't part of the save file.

### Loading Data a Line at a Time

The loading of the save data proper is handled by the `load_save_file` function.
The top half of this function is dedicated to setting up for and loading the data out of the save file, one line at a time.

We can think of loaded data as transitioning through two phases: temporary and committed.
As the save file is processed a line at a time, data is loaded in some temporary form.
Once we're happy that everything was loaded successfully, we can take the temporary data and convert it into its committed form; the form that the rest of the game will see and deal with.

#### Loading a Unique Line

The big loop near the beginning of the `load_save_file` function is what reads in each line of the save file.
Just before it are a bunch of lines that look like this:

```rust,ignore
let mut game_seed: Option<GameSeed> = None;
let mut turn_count: Option<TurnCount> = None;
let mut wins: Option<Wins> = None;
let mut base_equipment_level: Option<BaseEquipmentLevel> = None;
let mut difficulty: Option<Difficulty> = None;
let mut messages: Option<Messages> = None;
let mut player_alive: Option<PlayerAlive> = None;
let mut player_id: Option<PlayerId> = None;
let mut map: Option<Map> = None;
```

These are all temporary variables for unique data.
When the per-line loop reads in a unique, it fills one of these with a `Some` variant containing the loaded data for that unique.

The first conditional block of the per-line loop checks for an asterisk character and a whitespace.
If those characters are detected, the line is trimmed to the point after them and the loading process will attempt to interpret the line according to all of the unique types it knows of, one at a time.

Unique lines are loaded via the `deserialize_named_unique` function, which has a helper `deserialize_unique!` macro to reduce typing redundancy.
The `deserialize_named_unique` function accepts one of the temporary variables for holding unique data.
It does one of the following things, depending on the contents of the data line it sees:

1. Returns `Ok(false)` if the unique type name doesn't match what was expected.
2. Returns `Ok(false)` if deserializing failed for whatever reason.
3. Returns `Err(LoadError::DuplicateUnique(...))` if the temporary variable was already filled in and the load would otherwise have succeeded.
4. Returns `Ok(true)` and fills in the temporary variable with the loaded data if none of the above occurred.

In other words, a return value of `Ok(true)` means the unique data was successfully loaded, while `Ok(false)` indicates that other unique types should be attempted.

#### Loading a Component Line

When components are loaded in, they need to be attached to newly-created entities.
These entities share the same world space as any entities that existed before the loading process started.
When entities are created during the loading process they're added to the `despawn_ids` vector passed into the `load_save_file` function.
As mentioned before, entities in this vector will eventually be despawned if they're still there when the `load_save_file` function is done.
Therefore, we can think of components loaded and attached to these entities as temporary storage.

With that in mind, we can now consider lines that should contain component data.
The loading process will try to interpret any line that doesn't start with an asterisk character as a component line instead.

The first part of a component line is the entity that the component should be attached to.
This entity ID is meaningful within the save data, but is meaningless in the current world.
To reconcile this, we need to map each distinct entity ID that we encounter while loading components to *fresh* entities that have their own new entity IDs.
Near the top of the `load_save_file` function is the data structure whose job is to manage exactly this:

```rust,ignore
let mut old_to_new_ids: HashMap<EntityId, EntityId> = HashMap::new();
```

The keys of this hash map are the entity IDs as listed in the save file, while the values are the entity IDs of the corresponding fresh new entities that represent them in their real, loaded form.
If the entity ID at the beginning of the component line exists, it is simply retrieved.
If it doesn't, it is created and added to this hash map.

Since we have the ID of the new entity, we can proceed to attempt to load components, trying one type at a time.
This is done with the `deserialize_named_component` function and the `deserialize_component!` helper macro that work much like how `deserialize_named_unique` and `deserialize_unique!` did for uniques.
In fact, the `deserialize_named_component` function works the same way as the `deserialize_named_unique` function, except that it attaches component data to a new temporary entity passed in by ID and emits `Err(LoadError::DuplicateComponent(...))` instead.

At this point, any line that cannot be read as data for a unique or for a component causes the `load_save_file` function to return `LoadError::UnrecoginzedLine` as an error.

### Checking Uniques

Once every line in the save file has been processed, we need to check that every unique is accounted for.
The code is short enough to show here in its entirety:

```rust,ignore
// Check that all uniques are present.
let game_seed = game_seed.ok_or(LoadError::MissingUnique("GameSeed"))?;
let turn_count = turn_count.ok_or(LoadError::MissingUnique("TurnCount"))?;
let wins = wins.ok_or(LoadError::MissingUnique("Wins"))?;
let base_equipment_level =
    base_equipment_level.ok_or(LoadError::MissingUnique("BaseEquipmentLevel"))?;
let mut difficulty = difficulty.ok_or(LoadError::MissingUnique("Difficulty"))?;
let messages = messages.ok_or(LoadError::MissingUnique("Messages"))?;
let player_alive = player_alive.ok_or(LoadError::MissingUnique("PlayerAlive"))?;
let mut player_id = player_id.ok_or(LoadError::MissingUnique("PlayerId"))?;
let mut map = map.ok_or(LoadError::MissingUnique("Map"))?;
```

The above code checks that each unique type was loaded, and bails with an error if any of them are missing.
It also uses *shadowing* to redefine the temporary unique variables into non-`Option` form so they're easier to work with later in the loading code.

### Fixing Entity IDs

When component lines were being processed, new entities were being created according to the entity ID found at the beginning of those lines.
However, there are also entity IDs present in the data payloads at the end of unique and component lines as well that are loaded verbatim, which means they refer to the IDs at the beginning of lines.
We need to fix these IDs to point to the IDs of the entities created during the loading process by converting them according to the `old_to_new_ids` hash map that was built up earlier.

There are two uniques and two components that hold entity IDs and thus need fixing.
The unique types are `Difficulty` and `PlayerId`, while the component types are `Equipment` (weapon and armor) and `Inventory` (items).
The loading code takes care to only iterate over entities that were created during the loading process by filtering by the values of the `old_to_new_ids` hash map.

Converting old save IDs to new loaded entity IDs also doubles as an integrity check to ensure that each ID refers to an existing entity in the save file.
If any of the IDs to fix are absent from the `old_to_new_ids` hash map, a `LoadError::UnknownId` error is raised.

### Placing Entities on The Map

If you look at the serialized version of the map in a save file and compare it to the definition of the `Map` struct in the `src/map.rs` file, you'll notice that the `tile_entities` field isn't being serialized.
This is the spatial cache that's used to speed up access to entities according to their position in the map.
It doesn't need to be saved or loaded because the same information is stored in the `Coord` components of each entity.
However, this spatial cache still needs to be restored when loading a save file; this is done by simply iterating over all entities with a `Coord` component and using the `Map::place_entity` function to fill in the cache.

### Committing Loaded Uniques and Entities

So far all of our data has been loaded in a temporary form: uniques are loaded in local variables, while components are attached to temporary entities.
We want to *commit* our temporary data; that is, prepare it so it can be used by the rest of the game.

Committing temporary entities involves clearing out the `despawn_ids` vector that was passed in at the beginning.
Its contents are replaced with old entities that were around before loading that need to be despawned.
There are only two entities that fall under this description: the difficulty tracking entity and the player entity (plus any equipment and items they may have).
We know that we only have to handle these two entities because loading only happens at the title screen, so no other unassociated entities exist at that point in the game.

After committing temporary entities comes committing uniques, which is a simple matter of assigning over or replacing each unique type individually.
The API of Shipyard 0.4 has no way to replace or remove a unique once one has been added to a world, so this involves some clumsy `replace` function definitions for some unique types, but otherwise it works.

### After Loading

At this point all of the saved data has been loaded and prepared, so all that's left is to bounce the player right back into the gameplay.
The original invocation of the `load_game` function in the `TitleMode::update` function triggers a mode switch to `DungeonMode` which does pretty much that.

So like I said earlier: loading is a lot more complicated than saving.
Despite all of these checks and safe-guards, there's a lot of ways a save file can be loaded and accepted by the game, but still be broken.
For example:

- A map has a set width and height, but could be loaded with insufficient tiles.
- Numbers that are typically positive could be negative.
- What if entities are missing important components?
- A lot of nonsense can happen if entity IDs are changed to refer to unintended entities.

The loading logic of RuggRogue doesn't check for any of these; it's complex enough as-is and there's almost no limit to the number of things that could be wrong with the data that it loads.
Instead, it's mostly content to successfully load a save file that was produced by the saving logic.
If the save file is messed up, the impact on the game can vary from minor unintended behavior all the way to *panicking*, which is Rust's safe way of bailing out at the first sign of trouble it detects.

Opening and reading a save file is a good way of gaining insight as to what data exists at any given point in a game.
You can also have some fun by modifying a save file: try cranking up `Wins` to 300 and witness the flood of monsters and items, or add a zero or two to the player's maximum hit points.

## Run-Length Encoding

If you've been reading up to this point, you might have noticed that something is missing in this explanation: how is serialization and deserialization actually performed?
If each line of the save file ends with JSON-formatted data, how are data structures converted to and from JSON when saving and loading?

The reason that all of this has been glossed over is because RuggRogue outsources the task to two crates: [serde](https://crates.io/crates/serde) and [serde\_json](https://crates.io/crates/serde_json), that make it almost trivial.
RuggRogue first uses Serde to annotate any data structure that needs this treatment using the *derive macros* that provides, like so:

```rust,ignore
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct GameSeed(u64);
```

In the above example, `GameSeed` is a simple tuple struct, but this annotation works for larger structs with fields and enums as well.
Every data structure that RuggRogue needs to save into the save file is annotated in this way.

Once annotated like this, these data structures can then be serialized into JSON format using serde\_json, e.g.:

```rust,ignore
// NOTE: Not real saving code!

use serde_json::Serializer;

let mut writer = SomethingToWriteInto::new();
let game_seed = world.borrow::<UniqueView<GameSeed>>();
game_seed.serialize(&mut Serializer::new(&mut writer))?;
```

Deserialization from JSON is also handled by serde\_json:

```rust,ignore
// NOTE: Not real loading code!

use serde_json::Deserializer;

let line = GetALineFromSomewhere::new();
let mut ds = Deserializer::from_str(line);
let mut game_seed = GameSeed::deserialize(&mut ds)?;
world.borrow::<UniqueViewMut<GameSeed>>().0 = game_seed.0;
```

The above code samples aren't real code, but they're rough approximations of what happens inside the `serialize_named_unique`, `serialize_named_storage`, `deserialize_named_unique` and `deserialize_named_storage` functions.

For simple data, this is all that is needed for serialization and deserialization.
However, let's take a look at the definition of the `Map` struct in the `src/map.rs` file, paying attention to the annotations:

```rust,ignore
#[derive(Deserialize, Serialize)]
pub struct Map {
    pub depth: i32,
    pub width: i32,
    pub height: i32,
    #[serde(with = "crate::saveload::run_length_encoded")]
    tiles: Vec<Tile>,
    pub rooms: Vec<Rect>,
    pub seen: BitGrid,

    // (x, y) -> (blocking_entity_count, entities_here)
    #[serde(skip)]
    tile_entities: HashMap<(i32, i32), (i32, Vec<EntityId>)>,

    // zero-length non-zero-capacity vectors for reuse in tile_entities
    #[serde(skip)]
    empty_entity_vecs: Vec<Vec<EntityId>>,
}
```

Note the `"crate::saveload::run_length_encoded"` annotation on the `tiles` field.
This is how RuggRogue tells Serde that it wants to save the `tiles` field differently than how it normally would.
In this case, we know that there would be a lot of redundancy when saving the tile data of the map, so to cut it down we want to handle this field specially.

The way we want to cut down the size of the tile data that we save is by [run-length encoding](https://en.wikipedia.org/wiki/Run-length_encoding) it.
Instead of writing each tile individually, we want to write the tile followed by the number of repetitions until a different tile appears.
This has a pretty dramatic impact on save file size: basic tests show that it reduces save file size by about two-thirds!

The `"crate::saveload::run_length_encoded"` bit refers to the `run_length_encoded` module near the bottom of the `src/saveload.rs` file.
The `serialize` function contained within just reads in tiles and tracks runs of repeated tiles into a temporary vector, and just serializes that vector.
Conversely, the `deserialize` function reads in the vector in that format and uses some Rust iterator functions to convert it back into a normal vector of tiles that the `Map` struct actually wants.

For the tiles of a map, that's all that we need to perform run-length encoding.
However, we also want to apply run-length encoding to things like our fields of view, specifically to the `bv` field of our homegrown `BitGrid` struct in the `src/bitgrid.rs` file:

```rust,ignore
/// A width-by-height-sized BitVec for convenient handling of a grid of boolean values.
#[derive(Deserialize, Serialize)]
pub struct BitGrid {
    width: i32,
    height: i32,
    #[serde(with = "crate::saveload::bit_vec")]
    bv: BitVec,
}
```

We have a `BitVec` here which is notably not Rust's standard `Vec` type that the `run_length_encoded` module wants.
The `"crate::saveload::bit_vec"` bit refers to the `bit_vec` module at the very bottom of the `src/saveload.rs` file.
The only job of this module is to convert the `BitVec` to and from a standard Rust `Vec<u8>` of ones and zeroes and then hand it off to the `run_length_encoded` module to apply its run-length encoding.
The end result is run-length encoding for `BitVec` fields that decreases the amount of space needed to save them.

A final note: apparently the `BitVec` type can be processed by Serde with an internal representation that's even more compact than the roundabout run-length encoding that RuggRogue uses.
However, the documentation of that crate warns that its output isn't guaranteed to stay stable.
What's more, even trying to use it as-is resulted in serialized output that the deserializer would choke on.
In a sense, the run-length encoding that RuggRogue uses is actually a workaround for the `BitVec` type not serializing and deserializing correctly to begin with.

## Save Support for the Web Build

The native build of RuggRogue writes game data to the `savegame.txt` file in the same directory as the game itself, but what about the web build?
Obviously the web build can't just write files to the visitor's local filesystem directly.
The web version of RuggRogue is created using [Emscripten](https://emscripten.org/), which provides an in-memory file system by default called [`MEMFS`](https://emscripten.org/docs/api_reference/Filesystem-API.html#filesystem-api-memfs) that enables standard file operations to just work.
The downside of this default system is that anything saved to this file system is lost the moment the player closes the tab.

In order to allow the player to save the game and load it when visiting the game page at a later point in time, we need an Emscripten file system that will preserve the save file written into it.
RuggRogue uses [`IDBFS`](https://emscripten.org/docs/api_reference/Filesystem-API.html#filesystem-api-idbfs) to accomplish this, which provides a file system backed by an `IndexedDB` instance provided by the web browser.

The first step to using `IDBFS` is to link it in so it can be used at all.
RuggRogue does this by passing `-lidbfs.js` as a linker option to the Emscripten toolchain in the `.cargo/config.toml` file.

If you take a look near the top of the `src/saveload.rs` file, you may have noticed this bit regarding the location of the save file:

```rust,ignore
#[cfg(target_os = "emscripten")]
const SAVE_FILENAME: &str = "/ruggrogue/savegame.txt";

#[cfg(not(target_os = "emscripten"))]
const SAVE_FILENAME: &str = "savegame.txt";
```

This sets the save file location to `savegame.txt` when building the native version of the game, while putting the save in the fixed `/ruggrogue/savegame.txt` location in the web version instead.
The `/ruggrogue` directory is a location that we want to create in Emscripten's virtual file system, which will be mounted as an `IDBFS`.
This is done with some JavaScript inside the `index.html` file:

```javascript
var Module = {
    // ...
    'preRun': [function () {
        FS.mkdir("/ruggrogue");
        FS.mount(IDBFS, {}, "/ruggrogue");
        FS.syncfs(true, function (err) {});
    }],
};
```

The above snippet creates `/ruggrogue` as a mount point in Emscripten's virtual file system, mounts an `IDBFS` instance there, and loads in any data saved in the web browser's IndexedDB into it.
In theory, that's all that's needed to get saving and loading to work in the web version of RuggRogue.

Unfortunately, this process is imperfect.
The final `FS.syncfs` call above is asynchronous, and the callback provided is supposed to be called when it's done, but I couldn't work out how to make Emscripten wait for it to be called before jumping into the title screen of the game.
If you take a peek at the menu logic in the `src/modes/title.rs` file, you can see that RuggRogue works around this by always including the "Load Game" option in the web build.

The other caveat of this IndexedDB approach is that persistent IndexedDB instances aren't available in private browsing tabs.
In that case, the game will silently fall back to the in-memory file system, so save files will be forgotten when the game page is closed.

To be honest, I don't know if doing all of this the way I'm supposed to.
Emscripten has a decent amount of reference documentation, but it's very thin on guidance, so a lot of what I did above was cobbled together from bits and pieces of the docs I could find.
There feels like there should be a better and more reliable way to do what I've done here, but I haven't found one, so I just had to make do with what I could find.
