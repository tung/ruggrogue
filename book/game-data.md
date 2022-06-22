# Game Data

The [Entity Component System chapter](entity-component-system.md) covered *how* RuggRogue uses Shipyard to store its game data.
This chapter is all about *what* that data actually is.
*Game data* here refers to all the data that defines RuggRogue as a game that isn't just technical book-keeping, such as the player, the map, items and monsters.

As mentioned in the entity component system chapter, game data is divided into two kinds:

1. *Uniques* that are independent of entities.
2. *Components* that are associated with an entity.

All game data is stored in a world that's passed pretty much everywhere throughout the code of RuggRogue.
The world stores exactly one of each type of unique, and every entity has either zero or one instance of each type of component.
This chapter will provide a run-down of what each of those types are, and will wrap up by covering how and when entities are spawned and despawned.

## Uniques

The following types are stored as uniques in RuggRogue's game world.

### `BaseEquipmentLevel`

Found in: `src/main.rs`

32-bit integer of the minimum power level of weapons and armors that spawn throughout the game.
It's set when starting a new loop of New Game Plus to ensure that all spawned equipment in this loop will be more powerful than in the previous loop.

### `Camera`

Found in: `src/chunked.rs`

Position that should be centered upon when drawing the map on the main game screen.
Usually this is the position of the player, but it can be shifted around in view mode.

### `Difficulty`

Found in: `src/experience.rs`

Tracks the total amount of experience to be gained by defeating every monster on the current level, as well as the ID of an entity that gains experience like the player.
Together, this tracking data calculates how much experience the player could gain upon defeating all of the monsters on each level.
The outcome of this tracking is used to determine the power level of items and monsters spawned on future levels.

### `GameSeed`

Found in: `src/main.rs`

64-bit unsigned integer that is used to provide random number sequences that are unique to each playthrough.
It's set to a random value or via a command line argument when starting a new game.
Loading a game populates this value from the save file.

### `Map`

Found in: `src/map.rs`

The map of the current dungeon level.
Mainly consists of a grid of tiles representing the level itself, but also includes the current dungeon depth, tracking of tiles the player has previously seen and a spatial cache of positions to entities.

### `MenuMemory`

Found in: `src/menu_memory.rs`

Tracks the last cursor position in various menus throughout the game so that they can be restored when the menu is opened again.
This makes it easier for the player to deal with longer menus repeatedly.

### `Messages`

Found in: `src/message.rs`

Holds the message log that appears in the sidebar in the main gameplay screen.
It has a maximum capacity of messages, and old messages will be cleared out as new ones are added when this capacity is exceeded.

### `MonsterTurns`

Found in: `src/monster.rs`

A heap that holds the entity IDs of monsters that should be given a turn to act between player turns.
The heap gives turns to monsters nearest to the player first.

### `Options`

Found in: `src/ui.rs`

Stores the tilesets and zoom settings of the tile grids that show the map in the main gameplay mode and the user interface as a whole.

### `PickUpHint`

Found in: `src/item.rs`

A flag that determines whether the game should append a hint of the key to press to pick up items when the player steps over one.
It's set at the start of each new game, and is unset once the player picks up an item.

### `PlayerAlive`

Found in: `src/player.rs`

A flag that's `true` when the player is alive and `false` when they've died.
This determines whether the player should keep getting turns, as well as if they should get a win screen or game over screen when the game ends.

### `PlayerId`

Found in: `src/player.rs`

ID of the entity representing the player.
This is consulted pretty much universally throughout the game to read from or modify data associated with the player.

### `TurnCount`

Found in: `src/main.rs`

64-bit unsigned integer representing the number of elapsed turns since the start of each game.
It's shown in the user interface and in the game ending screens.

### `Wins`

Found in: `src/main.rs`

64-bit unsigned integer that counts the number of times the player has won the game.
This impacts the number of items and monsters that spawn in successive New Game Plus runs.

## Components

The following types represent components that are associated with entities.
As mentioned before, an entity can have either zero or one instance of each of these components.
Components can all be found in the `src/components.rs` file.

### `AreaOfEffect`

Attached to item entities to determine the radius of their circular area of effect when they're used.

### `Asleep`

Attached to player or monster entities when they are afflicted with the sleep status.
This contains a bit of hit point tracking to check if the affected entity took damage between turns, which reduces their sleepiness.

### `BlocksTile`

Tag component that is attached monster entities to block other monsters from stepping into their tile.
This causes monsters to find paths around each other when pursuing the player.

### `CombatBonus`

Attached to weapon and armor entities to determine how much extra attack and defense they confer when wielded or worn.

### `CombatStats`

Attached to player and monster entities to track hit points, as well as hold base attack and defense values.
This is the main component that is dealt with during combat.
When hit points reach zero here, the entity dies.

### `Consumable`

Tag component that is attached to items that indicates that the item can be used and that it will be consumed on use.

### `Coord`

Attached to player, monster and item entities to hold their coordinates on the current level's map when they are on the map.
In particular, items will lose this component when picked up, and gain it again when dropped on the ground.

### `EquipSlot`

Attached to item entities to determine whether they can be equipped as a weapon or armor.

### `Equipment`

Tracks the entity IDs of the weapon and armor equipped by an entity.
In practice, only the player has one of these components.

### `Experience`

Attached to the player to track their experience level and total experience points.
The `Difficulty` unique also has an entity with this component attached to track the total experience that can be gained per dungeon level.

### `FieldOfView`

Attached to players and monsters to determine their immediate fields of view.
It consists of a grid of flags that track which tiles are visible relative to a position on the map.

### `GivesExperience`

Attached to monsters to determine how many experience points they should grant when defeated.

### `HurtBy`

Attached to entities that take damage to track the source of that damage.
This is used to determine who to grant experience to when something dies, as well as provide a reason on the game over screen when the player dies.
This component is cleared from all entities at the end of each turn.

### `InflictsDamage`

Attached to consumable items to determine how much damage they should inflict when used.

### `InflictsSleep`

Attached to items to inflict sleep on targeted entities when used.

### `Inventory`

Attached to an entity to hold items that the entity picks up.
In practice, only the player is given one of these.

### `Item`

Tag component attached to an entity to indicate that it is an item.
An entity must have this component in order to appear in the player's pick up menu.

### `Monster`

Tag component attached to an entity to indicate that it is a monster.
This grants turns and artificial intelligence to the entity that they belong to between player turns.

### `Name`

Attached to entities to refer to them in menus and messages throughout the game.

### `Nutrition`

Attached to items to provide nutrition when used.

### `Player`

Attached to the player to store player-specific data, which in practice is tracking of their auto-run state.
There's a few places in the code that try to support multiple players, but the vast majority of the game logic leans on the singular `PlayerId` unique instead.

### `ProvidesHealing`

Attached to items to indicate the amount of hit points they should restore on their targets.

### `Ranged`

Attached to consumable items to indicate that they can be used on a target at range.
If the player uses an item with this component, they can target a distant space with the item.
If the item also has an `AreaOfEffect` component, that distant space will be the center of the area of effect.

### `RenderOnFloor`

One of two tag components that tells the game to draw the entity on the map.
Entities with this component are drawn below entities with a `RenderOnMap` component.

### `RenderOnMap`

One of two tag components that tells the game to draw the entity on the map.
Entities with this component are drawn above entities with a `RenderOnFloor` component.

### `Renderable`

Attached to entities that are drawn on the map to determine their visual appearance, such as their game symbol, foreground and background colors.

### `Stomach`

Attached to the player to give them hunger and regeneration mechanics.
Fullness tracked in this component slowly drains over time, and is replenished when an item with a `Nutrition` component is used.
An entity with normal levels of fullness will slowly regenerate hit points over time.
An entity with an empty stomach will instead take damage over time.

### `Tally`

Attached to the player to track interesting statistics throughout the course of their game, such as damage taken, damage inflicted and number of defeated monsters.
The statistics are shown to the player when their game ends, win or lose.

### `Victory`

Tag component attached to an item that results in the player winning the game when the item is used.
One item with this specific component is spawned once the player has descended deep enough into the dungeon.

## Spawning Entities

All entity spawning logic is centralized in the `src/spawn.rs` file.
Most entities are spawned at map generation time when the `fill_rooms_with_spawns` function is called, which in turn calls the following functions:

- `spawn_guaranteed_equipment` to spawn level-appropriate equipment at an steady but unpredictable pace.
- `spawn_guaranteed_ration` to spawn a single ration per level.
- `fill_room_with_spawns` to randomly populate rooms with items and monsters.

Items are spawned via the `spawn_random_item_at` function that calls one of the following functions at random:

- `spawn_weapon`
- `spawn_armor`
- `spawn_health_potion`
- `spawn_magic_missile_scroll`
- `spawn_fireball_scroll`
- `spawn_sleep_scroll`

The `spawn_weapon` and `spawn_armor` functions consider the current difficulty to determine the power level of the equipment they create.

Monsters are spawned via the `spawn_random_monster` function that then calls the `spawn_monster` function to create the monster entity itself.
Like the `spawn_weapon` and `spawn_armor` functions, the `spawn_monster` function considers the current difficulty level when determining the monster to create and its power level.

The positions of entities that exist on the map are stored in the `Coord` component of the entity.
This means that an entity with a `Coord` component is considered to be "on the map".
In addition to this, the map maintains a *spatial cache* that tracks a list of entity IDs for any given position.
This may seem redundant, but it drastically improves performance when dealing with entities by position by avoiding the need to iterate over all entities and check their positions manually.
The consequence of all this is that any entity that is added to the map needs to be given a `Coord` component *and* be placed correctly in the map's spatial cache.
Items and monsters are placed in the spatial cache at the end of their respective spawn-related functions.
This placement is done by calling the `Map::place_entity` function, whose definition can be found in the `src/map.rs` file.

There are exactly two entities that are *not* automatically added to the map's spatial cache: the difficulty tracker and the player.

The difficulty tracking entity has only an `Experience` component.
Its role is to track the total amount of experience that could be gained by defeating all monsters on the current map in order to gauge an appropriate power level to spawn items and monsters on future levels.
The difficulty tracking entity is created by the `spawn_difficulty` function in the `src/spawn.rs` file, and its entity ID is stored in the `id` field of the unique `Difficulty` struct defined in the `src/experience.rs` file.
The difficulty tracking entity is spawned when a new game is started, namely in the `new_game_setup` function defined in the `src/modes/title.rs` file.

The player entity is created by the `spawn_player` function defined in the `src/spawn.rs` file, which is called when starting a new game in the `new_game_setup` function.
The ID of the player entity is needed almost everywhere in the game, so it's stored in the `PlayerId` unique for easy access.
As mentioned before, the player entity is not automatically added to the map's spatial cache, but unlike the difficulty tracking entity, it eventually needs to be added so that the player can move around and do things on the map.
When this needs to happen, the `add_coords_to_players` function (defined in `src/player.rs`) is called, followed by the `place_player_in_first_room` function (defined in `src/map.rs`) to position it and add it to the spatial cache.

You may notice that the difficulty tracking and player entities are also created in the `main` function in the `src/main.rs` file.
These are dummy entities whose only purpose is to guarantee that these entities exist so that code that replaces these entities can despawn them unconditionally, which makes their logic simpler.

## Despawning Entities

Entities are despawned for a number of reasons:

1. Despawning items and monsters when moving between dungeon levels.
2. Despawning monsters when they are defeated.
3. Despawning items and monsters after the game over sequence.
4. Despawning old entities when starting or loading a game.

We'll cover each of these in turn.

### Moving Between Dungeon Levels

The simplest and most common reason for entities to be despawned is when the player descends to the next dungeon level.
Moving between levels means replacing the current map with a fresh map, which in turn means despawning entities on the current map so they can be replaced with new spawns.
This is the task of the `despawn_coord_entities` function (defined in `src/spawn.rs`), which is called by the `player_do_descend` function (defined in `src/player.rs`) when the player takes the downstairs of the current level.
It simply despawns entities that have a `Coord` component, which are the ones that belong to the map.

But the player has a `Coord` component; how does it avoid being despawned when moving between levels?
The answer is simple: all entities with the `Player` tag component are stripped of their `Coord` component by the `remove_coords_from_players` function in the `src/player.rs` file.
Once this despawning is done, player entities regain their `Coord` component through the `add_coords_to_players` function in the same file.

There's also the matter of the items belonging to the player: how do they avoid being despawned between levels?
The `Coord` component is removed from items while being picked up by the `remove_item_from_map` function in the `src/item.rs` file.
Conversely, a `Coord` component is attached to dropped items by the `add_item_to_map` function in the same file.
These functions take care to manage the map's spatial cache, which must be correct while the map exists.

### Defeating Monsters

The next most common reason to despawn entities is when the player defeats a monster.
This is the job of the `despawn_entity` function in the `src/spawn.rs` file, which despawns an entity by its ID.
This is called when a monster dies in the `handle_dead_entities` function in the `src/damage.rs` file.
The monster is removed from the map's spatial cache just before being despawned, for the same reason as when an item is picked up off the map.

### After Game Over

The game over sequence is where things get interesting with respect to entity despawning.
When a monster is defeated, it's despawned as usual.
However, when the player is defeated, their entity is *not* despawned.
Instead, the unique `PlayerAlive` flag is set to `false`, and the player is whisked away to the game over screen.

The game over screen shows a bunch of information about the player at the time that they were defeated, but it also shows the reason that they were defeated to begin with.
If the reason is due to a monster, then it needs the name of the monster in order to display it.
This means that the monster entity still exists in the game over screen.
In fact, the entire map and all the entities in it still exist in the game over screen!
It is only when the player leaves the game over screen that it clears up most of these entities by calling the `post_game_cleanup` function in the `src/modes/title.rs` file, which calls the `despawn_coord_entities` function mentioned earlier to do its heavy lifting.

There is exactly *one* entity that is not despawned by the `post_game_cleanup` function: the player entity.
The reason for this is to support the new game plus feature, which carries the player into the next iteration of the game, items, stats and all.

### Starting or Loading a Game

If the `post_game_cleanup` function never despawns the player, then what does?
In the case of starting a new game, this is done by the `new_game_setup` function in the `src/modes/title.rs` file.
This is why the dummy player entity is created when the game is first launched: it simplifies this logic.

Meanwhile, loading a game from a save file pretty much loads replacement entities for everything.
Assuming the load was successful, all of the old entities are manually despawned by the loading code using the `despawn_entity` function that you should probably be familiar with now.
This despawning is done by the `load_game` function in the `src/saveload.rs` file.

If you take a moment to think, you'll realize there's something missing in this explanation: what happens to the inventory items and equipment carried by the player when the player is despawned?
The answer is that all of those are despawned as well in the `despawn_entity` function.
The code in that function gathers up the entity IDs of any equipped weapon and armor, as well as the entity IDs of all inventory items, and deletes them along with the original entity itself.

All of this ceremony around despawning entities referred to by other entities is needed to avoid leaving entities *unreachable*.
An entity is considered reachable if:

1. Its ID is stored in a unique, e.g. the `id` field of the `Difficulty` unique, or the `PlayerId`.
2. It has a `Coord` component, meaning that it exists on the current map.
3. Its ID is stored in a component owned by another entity that is reachable, like the `Equipment` or `Inventory` components.

If an entity doesn't fit in any of the above cases, it is considered unreachable, which is the equivalent of a memory leak.
By despawning entities through the `despawn_entity` function instead of deleting them raw, we avoid making entities unreachable and thus leaking memory.
