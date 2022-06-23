# New Game Plus

RuggRogue allows the player to continue playing the game after they win by way of the *New Game Plus* mode.
When the player dismisses the victory screen, they're fully healed and are allowed deeper into the dungeon.
The player keeps their level, experience, stats and all of their equipment and items, apart from the victory item.
The game in turn will allow more monsters and items to spawn the more times the player wins.
The power of weapons and armor spawned will continue to increase, but the power of monsters resets.
Thus, New Game Plus is intended to serve as a victory lap for the player to collect ever more powerful equipment, and not so much deliver a proper challenge.

## Starting New Game Plus

The player wins the game by using the Present item.
The Present item has a `Victory` tag component that is checked by the `item::use_item` function in the `src/item.rs` file.
The game auto-saves itself before consuming the Present item, then increments the *win counter* before guiding the `DungeonMode::update` function in the `src/modes/dungeon.rs` file to bring up the `GameOverMode`.

Recall that `GameOverMode` is used both for player victory and defeat, depending on the state of the unique `PlayerAlive` flag.
When the player presses a key to dismiss the screen, the following code in the `GameOverMode::update` function in the `src/modes/game_over.rs` file runs:

```rust,ignore
let player_alive = world.borrow::<UniqueView<PlayerAlive>>().0;

title::post_game_cleanup(world, !player_alive); // <-- (1)
if player_alive {
    title::new_game_setup(world, true); // <-- (2)
}

inputs.clear_input();
return (
    ModeControl::Switch(if player_alive {
        // Jump straight into new game plus.
        DungeonMode::new().into() // <-- (3)
    } else {
        TitleMode::new().into()
    }),
    ModeUpdate::Immediate,
);
```

In the case of victory in the above code, the `player_alive` boolean flag will be `true`, so the game does the following:

1. Run the `title::post_game_cleanup` function.
2. Run the `title::new_game_setup` function.
3. Switch back to the `DungeonMode`.

The `title::post_game_cleanup` function is defined in the `src/modes/title.rs` file.
It simply despawns map-local entities such as monsters still alive and any items left behind, and in the case of victory that's all it does.

The `title::new_game_setup` function in the same file is used to start both new games and the New Game Plus runs.
If New Game Plus is needed, the `new_game_plus` argument is set to `true`, causing the function to:

- Increment the turn count and depth, as if the player had taken a downstairs on the final level.
- Restore the player to full health.
- Store the level of the difficulty tracker as the *base equipment level* for the New Game Plus run.

Since the difficulty tracker is reset for both new games and New Game Plus runs, the last point ensures that new weapons and armor that spawn will do so with ever-increasing power.

## Win Counter

As mentioned earlier, the *win counter* tracks the number of times the player beats the game.
It's defined in the form of the `Wins` unique all the way back in the `src/main.rs` file, like so:

```rust,ignore
pub struct Wins(u32);
```

The `Wins` unique is increased by one when the player uses the Present item in the `item::use_item` function in the `src/item.rs` file.

The win counter increases the maximum number of randomly-spawned items and monsters per room by one in New Game Plus runs.
The `fill_rooms_with_spawns` function in the `src/spawn.rs` file checks the `Wins` unique to accomplish this.

## Base Equipment Level

Recall that the difficulty tracker is reset by the `title::new_game_setup` function.
This resets the power of spawned monsters, but would also do the same thing to weapons and armor!
To counteract this, there's a concept of a *base equipment level*, stored in the form of the `BaseEquipmentLevel` unique in the `src/main.rs` file like so:

```rust,ignore
pub struct BaseEquipmentLevel(i32);
```

The `title::new_game_setup` function stores the level of the difficulty tracker in the `BaseEquipmentLevel` unique before resetting the difficulty tracker.
The `BaseEquipmentLevel` is then consulted by the `spawn_weapon` and `spawn_armor` functions in the `src/spawn.rs` file to add to the power of spawned equipment in New Game Plus runs.
This prevents equipment in New Game Plus runs from being useless relative to the equipment that the player was allowed to carry over from the previous run.
