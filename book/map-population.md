# Map Population

Once a map has been generated, the next task at hand is to place interesting things in it, such as monsters and items.

## When Map Population Takes Place

Maps are populated right after they're generated with the `map::generate_rooms_and_corridors` function, in two places:

1. When starting a new game in the `new_game_setup` function in the `src/modes/title.rs` file.
2. When the player descends a dungeon level in the `player_do_descend` function in the `src/player.rs` file.

## Placing the Victory Item

If the `map::generate_rooms_and_corridors` function doesn't place stairs, it will instead return a position where the victory item should be placed; this is the center of the last room in the room list.
The code looks like this in both places it occurs:

```rust,ignore
if let Some(victory_pos) = world.run(map::generate_rooms_and_corridors) {
    spawn::spawn_present(world, victory_pos);
}
```

The `spawn::spawn_present` function creates the victory item and places it in the given position on the map.
The function can be found in the `src/spawn.rs` file.

## Placing the Player

The player entity exists beyond the life of any one map, so whenever a map isn't ready, the player entity lacks a `Coord` component.
To place the player, this component is added again by calling the `player::add_coords_to_players` function defined in the `src/player.rs` file.
The player can then be moved to their starting position on the map using the `map::place_player_in_first_room` function defined in the `src/map.rs` file.

The player is guaranteed to start the map in a different room than the downstairs (or victory item), so long as the map has more than one room (which is almost always true).

## Placing Monsters and Items

With the player and possibly the victory item out of the way, all that's left is to fill the map with monsters and items.
The function responsible for this is `spawn::fill_rooms_with_spawns`, defined in the `src/spawn.rs` file.
This in turn kicks off three main tasks:

1. Spawn any weapons or armor that should be guaranteed based on the map's dungeon depth.
2. Spawn monsters and items in random rooms.
3. Spawn a guaranteed ration somewhere on the level.

A random number generator is created to decide how most, but not all, of this should play out; consult the [Randomness chapter](randomness.md) for details.

The exact order of these tasks doesn't matter, and they're simplest to talk about in reverse order, so we'll start with the ration.

## Guaranteed Ration

Every map is guaranteed to contain a single ration; this is the job of the `spawn_guaranteed_ration` function in the `src/spawn.rs` file.
It uses the `pick_random_pos_in_room` function to pick a random spot in a random room, and spawns a ration there with the `spawn_ration` function; both of these helper functions are in the same file.

## Filling the Rooms

The `spawn::fill_rooms_with_spawns` function goes through every room and randomly decides to place monsters and items in it, except for the first room where the player starts.
It calls the `fill_room_with_spawns` helper function (note the singular "room") to do this.

There is a 1-in-4 chance for an item to be spawned in each room.
The `spawn_random_item_at` helper function chooses, creates and places items.
The exact item selection is covered in a different chapter, but generally consists of consumable scrolls and potions, with the occasional weapon or armor.

Each room also has a 1-in-2 chance of spawning between one to three monsters.
Early levels limit the number of monsters that can spawn per room:

- Depths 1 and 2: one monster per room only.
- Depths 3 and 4: up to two monsters per room.
- Depth 5 and deeper: up to three monsters per room.

Like item spawning, there's a `spawn_random_monster_at` helper function that chooses, creates and places monsters.
Monster selection is a topic of a different chapter.

Finally, the limit for the number of items and monsters that can be spawned per room increases by one every time the player beats the game and picks New Game Plus.

## Guaranteed Weapons and Armor

The `spawn_guaranteed_equipment` function in the `src/spawn.rs` file is responsible for creating and placing weapons and armor beyond those from random room filling.
Unlike their random counterparts, guaranteed weapons and armor are always created at a power level appropriate for the current difficulty of the level.

The first kind of guaranteed equipment is the starting weapon and armor.
The game picks two random spots in the starting room and spawns and places the weapon and armor in them.

The second kind of guaranteed equipment needs some explanation.
As the player descends the dungeon, the monsters get stronger.
The player's base attack and defense power rises as they gain levels, but it rises slower than the power of the monsters.
To keep up, the player must pick up weapons and armor.
If weapons and armor were only created at random, it would be possible for the player to be left far less powerful than the monsters they face for a very long time.

To counter this issue, the game creates weapons and armors periodically according to the dungeon depth.
The simplest approach would be to spawn such equipment once every, say, four levels.
RuggRogue does essentially that, with a couple of twists to make the pattern less predictable:

1. The *period offset* is adjusted per game, e.g. one run groups levels 1-4, 5-8, 9-12, etc. while another groups them 1-2, 3-6, 7-10, etc.
2. The *chosen level* differs per group, e.g. pick level 2 out of levels 1-4, then level 7 out of levels 5-8, etc.

There's no data carried between different levels to affect their generation.
Instead, new random number generators are created that are seeded such that each level in a group gets the same seed; this is accomplished by integer dividing the map depth by the `EQUIPMENT_SPAWN_PERIOD` constant whose value is `4`.

The `periodic_weapon_rng` uses the low bits of the game seed to adjust the period offset.
A single random number from `0` to `2` is extracted from this random number generator (`3` is never chosen to avoid guaranteed equipment spawning in adjacent levels).
This number is checked against the depth of the level within its group; a successful match spawns a weapon in a random room and position.

This process is repeated for armor using `periodic_armor_rng`, except offsetting with the high game seed bits.
