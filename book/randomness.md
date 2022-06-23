# Randomness

RuggRogue is a roguelike, and the signature feature of roguelikes is the use of procedural content generation: levels, items and enemy placement differ from play to play.
This is typically achieved through the use of a *random number generator* (or *RNG*).
This chapter will cover generation, usage and considerations of random numbers by RuggRogue.

## Generating Random Numbers

Roguelikes typically get their random numbers from what's called a *pseudo-random number generator* (or *PRNG*).
"Pseudo" means "fake", so a PRNG produces a *deterministic* series of numbers that just happen to look random enough for the purposes of a computer game.
RuggRogue takes advantage of this determinism.
Most roguelikes just initialize a single PRNG, hang onto it and share it across the whole game logic.
RuggRogue instead creates *temporary* PRNGs whenever some code needs random numbers and discards those PRNGs afterwards.
By carefully controlling how these PRNGs are initialized we can achieve *seeded games*, where two games with the same seed produce the same dungeon and spawns.
Seeded games are useful for debugging and play-testing, and can be fun for players to mess around with.

The nuts and bolts of generating random numbers is handled via external crates: [rand](https://crates.io/crates/rand), [rand\_xoshiro](https://crates.io/crates/rand_xoshiro) and [wyhash](https://crates.io/crates/wyhash).
rand provides a convenient interface to make use of random numbers generated from a backend source crate.
rand\_xoshiro is a fast PRNG that provides the actual random numbers.
wyhash is a hasher that takes input values and produces seed values to initialize PRNGs.
These three crates serve as the foundation of all of the randomness in RuggRogue.

A PRNG is initialized with a seed value that determines the sequence of random numbers that will be produced, so the first step in generating random numbers is to create this seed value.
These seed values are created by combining the following input values using wyhash:

1. A unique magic number.
2. The game seed.
3. Any other relevant differentiating input values.

Each place in the code that needs random numbers starts by feeding a magic number into a wyhash hasher.
Each magic number is an arbitrary number that is used in only one place in the code, the values of which can be found in the `src/magicnum.rs` file:

```rust,ignore
///! Arbitrary constants to seed hashers whose output is in turn used to seed RNGs.

pub const GENERATE_ROOMS_AND_CORRIDORS: u64 = 0x3fdc77fb4d7f5d2f;
pub const SPAWN_GUARANTEED_WEAPON: u64 = 0x67caf3e7b16e9df2;
pub const SPAWN_GUARANTEED_ARMOR: u64 = 0x74e90549dbcadfd0;
pub const FILL_ROOM_WITH_SPAWNS: u64 = 0xd85af3d2cf6dcbc5;
pub const MELEE_ATTACK: u64 = 0x258890651a33d5d;
```

Since there are five magic number constants, there are five unique places in the code where PRNGs are created.
The fact that the magic numbers have different values helps to avoid the same seed being used by different PRNGs, which would otherwise produce the same random number sequence.

The game seed is a unique number associated with a game that is the sole reason that different games have different dungeon layouts and outcomes.
The initial game seed value can be provided as a command line argument or randomly generated as needed; this is one of the first things done in the `main` entry point function in the `src/main.rs` file.
Starting a new game causes that game to adopt that initial value as that game's seed; this value is preserved across saves and loads.
If the player returns to the title screen for whatever reason, the initial game seed value is changed into another random value to avoid accidentally playing the same dungeon again.

With the magic number and game seed fed into the hasher, the final thing the hasher needs is some relevant differentiating input values.
For example, the PRNG associated with `GENERATE_ROOMS_AND_CORRIDORS` provides the dungeon depth so that depth 2 has a different layout to depth 1.

The final hash value is then used as the seed for that particular PRNG.
Here's a code excerpt for `GENERATE_ROOMS_AND_CORRIDORS` from the `src/map.rs` file that demonstrates initializing a PRNG from hashed input values:

```rust,ignore
use rand::SeedableRng;
use rand_xoshiro::Xoshiro128PlusPlus as GameRng;
use std::hash::Hasher;
use wyhash::WyHash;

use crate::{magicnum, GameSeed};

pub fn generate_rooms_and_corridors(/* ... */) {
    // ...

    let mut rng = {
        let mut hasher = WyHash::with_seed(magicnum::GENERATE_ROOMS_AND_CORRIDORS);
        hasher.write_u64(game_seed.0);
        hasher.write_i32(map.depth);
        GameRng::seed_from_u64(hasher.finish())
    };

    // Use rng to get random numbers...
}
```

## Uses of Random Numbers

RuggRogue uses random numbers to control map generation, monster and item spawns and combat outcomes.

### Map Generation

The PRNG for map generation is initialized in the `generate_rooms_and_corridors` function in the `src/map.rs` file with the help of:

- `magicnum::GENERATE_ROOMS_AND_CORRIDORS`
- The game seed.
- The dungeon depth for that map.

This PRNG determines:

- the position and size of rooms,
- whether to connect rooms with a horizontal corridor followed by a vertical corridor or vice versa, and
- which pairs of rooms are connected with additional corridors beyond the minimum required to connect all of the rooms together.

### Monster and Item Spawns

The main PRNG for determining monster and item spawns is initialized in the `fill_rooms_with_spawns` function in the `src/spawn.rs` file with the help of:

- `magicnum::FILL_ROOM_WITH_SPAWNS`
- The game seed.
- The dungeon depth of the map being filled with spawns.

This PRNG determines:

- the placement of the starting weapon and armor in the room that the player starts in,
- the placement of the guaranteed ration on each level,
- whether a room should spawn items and where they should be placed,
- whether a spawned item should be equipment or consumable,
- whether spawned equipment should be a weapon or armor,
- the exact type of a spawned consumable item,
- the random extra bonus for spawned weapons and armors, and whether to round fractional power values up or down,
- whether a room should spawn monsters and where they should be placed, and
- the power levels of spawned monsters, and whether to round fractional power values up or down.

The game makes use of two additional PRNGs to periodically spawn guaranteed weapons and armors beyond the first level.
Both of these PRNGs exist in the `spawn_guaranteed_equipment` function in the `src/spawn.rs` file.
The PRNG for spawning guaranteed weapons is initialized with:

- `magicnum::SPAWN_GUARANTEED_WEAPON`
- The game seed.
- The sum of the dungeon depth and the numeric value of the low four bytes of the game seed, all divided by four.

The division by four in the last value causes each sequence of four adjacent levels to seed the guaranteed weapon PRNG with the same value and thus produce the same random number sequence.
This allows those levels to effectively share the same PRNG sequence so that only one of those levels will spawn a guaranteed weapon.
The four bytes of game seed adjusts the offset of the levels so the depth sequences aren't just 1-4, 5-8, etc. for every single game.

The initialization of the PRNG to determine guaranteed armor spawning is identical to that of the guaranteed weapon, except for the use of `magicnum::SPAWN_GUARANTEED_ARMOR` and the use of the high four bytes of the game seed instead of the low four bytes.

### Combat Outcomes

The combat PRNG exists in the `melee_attack` function in the `src/damage.rs` file, and is initialized with the help of:

- `magicnum::MELEE_ATTACK`
- The game seed.
- The current turn count.
- The x and y coordinates of the attacker.
- The x and y coordinates of the defender.

The combat PRNG determines:

- whether the melee attack hits or misses, assuming the defender is not asleep,
- whether to fluctuate damage, and if so, whether to modify it plus or minus 50%, and
- whether to round fractional damage values up or down to the nearest whole number.

## Ensuring Identical Randomness with Native and Web Builds

In the course of testing, I noticed that there were differences between the native and web versions of the game with the presence and placement of monsters and items, given the same game seed.
This meant that the same game seed was causing different random numbers to be produced by the same PRNG across the two builds!

After a lot of debugging, I discovered that this was due to the native and web builds pulling out different amounts of data from the PRNGs.
This divergence comes from pulling a `usize` from a PRNG; this is 64 bits on the `x86_64` architecture of the native build, but only 32 bits on the `wasm32` architecture of the web build.
Replacing the `usize` with a `u32` fixes the issue, but where this fix is needed can be pretty subtle.
For example, can you spot the problem here?

```rust,ignore
let num = rng.gen_range(1..2);

for pos in room.iter_xy().choose_multiple(rng, num) {
    spawn_random_item_at(world, rng, pos);
}
```

In the above code, the `num` variable is inferred by Rust to be of the `usize` type due to the lack of type annotations and the fact that it's the type of the second argument of the `choose_multiple` function.
This causes `rng.gen_range(1..2)` to produce different values for the native and web versions of the game.
Annotating the `1..2` input with an integer type of a fixed size resolves the issue:

```rust,ignore
let num = rng.gen_range(1i32..2i32);

for pos in room.iter_xy().choose_multiple(rng, num as usize) {
    spawn_random_item_at(world, rng, pos);
}
```

The use of `rng.gen_range(1i32..2i32)` extracts a 32-bit `i32` value on both the `x86_64` and `wasm32` architectures.
Note that the `num` variable above is now also of type `i32`, so it needs to be cast into `usize` when being passed into the `choose_multiple` function.

## Conclusion

This chapter serves as a high-level overview of RuggRogue's approach to generating and using random numbers.
The biggest thing to take away from all of this is the focus on determinism by seeding PRNGs with the hashed combination of carefully selected input values.
I've deliberately glossed over the nitty gritty details of exactly how each random number is used to produce random outcomes, which are better covered by other chapters.
