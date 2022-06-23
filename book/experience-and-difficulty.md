# Experience and Difficulty

Combat between the player and the monsters is at the core of RuggRogue.
Combat is defined by the damage formula described in the [Turn Order and Combat](turn-order-and-combat.md) chapter, but that by itself isn't enough; the formula operates on numbers that still need to be decided.
These numbers include the hit points, attack and defense of the player and the monsters, as well as the attack and defense bonuses of weapons and armor.
The values of these numbers need to be able to answer questions such as how many hits are needed for a player to defeat a monster of equal level.

There's another problem.
Like many roguelikes and RPGs, the player in RuggRogue earns experience points when defeating monsters.
When enough experience points are earned, the player gains a level, rewarding them with more power.
How does the game avoid becoming easier over time?

## Game Balance

Questions about picking numbers are really questions about how easy or hard the game should be; in other words, they're questions about game balance.
For game balance to be reasoned about, we need to consider the relationship between the numbers of the player and those that make up the challenges and rewards encountered in the dungeon.

If the player has an experience level, the game must spawn monsters with some concept of a level as well.
Rewards that the player finds in the dungeon, such as weapons and armor, must also consider the concept of a level, if only to avoid being too strong or too weak when found.

Everything that becomes more powerful over time is created with an experience level, but exactly how powerful should each level be?
We definitely don't want level 2 to be twice as powerful as level 1, for example.
RuggRogue defines a *level factor* to control how powerful each level should be.
It's definition can be found in the `src/experience.rs` file, and looks like this:

```rust,ignore
fn level_factor(level: i32) -> f32 {
    (1.0 + (level - 1) as f32 * 0.1).max(0.1)
}
```

The level factor is 1.0 at level 1, 1.1 at level 2, 1.2 for level 3, and so on.
In other words, every level is 10% more powerful than level 1.

The level factor is used to derive every other number that grows more powerful over time.
The formulas are described as functions in the `src/experience.rs` file:

Function Name | Formula (f = level factor) | Description
------------- | ------- | -----------
`calc_player_max_hp` | (f * 30.0).round() as i32 | Player maximum hit points
`calc_player_attack` | f * 4.8 | Player base attack
`calc_player_defense` | f * 2.4 | Player base defense
`calc_monster_max_hp` | (f * 14.0).round() as i32 | Monster maximum hit points
`calc_monster_attack` | f * 8.0 | Monster attack
`calc_monster_defense` | f * 4.0 | Monster defense
`calc_monster_exp` | (f * 10.0).ceil() as u64 | Experience points for defeating monster
`calc_weapon_attack` | f * 3.2 | Weapon attack bonus
`calc_armor_defense` | f * 1.6 | Armor defense bonus

For example, consider a player at experience level 6.
The level factor at level 6 is `(1.0 + (6 - 1) as f32 * 0.1).max(0.1) = 1.5`.
Such a player thus has `(1.5 * 30.0).round() as i32 = 45` maximum hit points, `1.5 * 4.8 = 7.2` base attack and `1.5 * 2.4 = 3.6` base defense.
Note that attack and defense values are internally stored and handled as floating point values; the interface rounds them to whole numbers for display purposes.

Defining numbers in terms of the level factor like this allows us to make statements about things when their experience levels are equal:

- The sum of player attack (4.8) and weapon attack (3.2) is 8.0, which is equal to monster attack (8.0).
- Likewise, the sum of player defense (2.4) and armor defense (1.6) is 4.0, which is equal to monster defense (4.0).

According to the damage formula described in the [Turn Order and Combat](turn-order-and-combat.md) chapter, melee hits cause 50% of the attack value worth of damage when attack is twice defense.
A melee hit of 8.0 attack versus 4.0 defense causes 4.0 damage.
This allows us to talk about how tough the player and monsters are:

- A player with 30 maximum hit points is defeated by 4.0-damage monster attacks in 7.5 turns.
- A monster with 14 maximum hit points is defeated by 4.0-damage player attacks in 3.5 turns.

Statements like this establish numeric relationships that form the foundation of game balance in RuggRogue.

## Player Experience

The experience data for the player is stored in an `Experience` component defined in the `src/experience.rs` file:

```rust,ignore
pub struct Experience {
    pub level: i32,
    pub exp: u64,
    pub next: u64,
    pub base: u64,
}
```

The `level` field is the player's experience level.
The `exp` field is the number of experience points the player has accumulated.
When it reaches the threshold value stored in the `next` field, the player gains a level, `next` is deducted from `exp` and `next` is increased to a larger value.
The `base` field stores the total amount of experience points that have been cashed in as levels.
The sum of `base` and `exp` is the total number of experience points earned by the player, which is shown in the sidebar while playing the game.
The player is initially spawned with a `level` of 1 and `next` set to 50 experience points to gain for their next level, as defined in the `spawn::spawn_player` function in the `src/spawn.rs` file.

Meanwhile, the number of experience points awarded for defeating a monster is stored in a `GivesExperience` component attached to monster entities, defined in the `src/components.rs` file like so:

```rust,ignore
pub struct GivesExperience(pub u64);
```

When the player attacks a monster, the monster is tagged with a `HurtBy` component that credits the player for the damage.
This is done within the `damage::melee_attack` function in the `src/damage.rs` file.
If the hit kills the monster, the `damage::handle_dead_entities` function in the same file will award the experience points in the monster's `GivesExperience` component to the player's `Experience` component.

## Gaining Levels

The experience points stored in `Experience` components are checked when the `DungeonMode::update` function in the `src/modes/dungeon.rs` file calls the `experience::gain_levels` function in the `src/experience.rs` file.
These calls occur whenever experience could have been awarded, i.e. when time passes.

The `experience::gain_levels` function checks if the `exp` value exceeds the `next` value; if so:

- The entity's level is increased by 1.
- `next` is deducted from `exp`.
- `next` is added to `base`.
- `next` is increased by 10% of its current value.

If the entity has a `CombatStats` component, then their hit points, attack and defense are increased to match their freshly-gained level.
For players, this involves the `calc_player_max_hp`, `calc_player_attack` and `calc_player_defense` functions.
If monsters could gain levels, they would use the `calc_monster_max_hp`, `calc_monster_attack` and `calc_monster_defense` functions instead.
Care is taken to preserve any maximum hit points gained from drinking health potions while fully healed.

Finally, a message is logged if the entity gaining the level happens to be the player.

## Difficulty Tracker

The player gains experience levels over time while playing the game and thus becomes more powerful over time.
If the game never increased the levels of spawned monsters, the game would get easier over time!
But how quickly should the level of spawned monsters increase?
Too slowly and game would still get easier over time.
Too quickly and the player would eventually be overwhelmed.

It would be tempting to solve this by simply spawning monsters with the same level as the player.
However, this would lead to *rubber banding*, which can make players feel like they're being punished for playing too well.

RuggRogue takes a different approach in the form of a *difficulty tracker*.
The difficulty tracker is associated with an entity that has an `Experience` component and is thus able to gain experience and levels, but is invisible and has no `CombatStats` component.
The `spawn::spawn_difficulty` function in the `src/spawn.rs` file creates this entity when called from the `title::new_game_setup` function in the `src/modes/title.rs` file.

The difficulty tracker itself is a unique `Difficulty` struct defined in the `src/experience.rs` file as follows:

```rust,ignore
pub struct Difficulty {
    pub id: EntityId,
    exp_for_next_depth: u64,
}
```

The difficulty tracker gains experience in a different way to the player:
After map population, the `experience::calc_exp_for_next_depth` function defined in the `src/experience.rs` file is called.
This sums the experience carried by all monsters on the map and saves it to the `exp_for_next_depth` field.
When the player descends, the points saved in the `exp_for_next_depth` field are transferred to the difficulty tracker entity.
From there, the `experience::gain_levels` function is called so the difficulty tracker entity can gain levels if it needs to.

The level of the difficulty tracker entity determines the maximum level of monsters to spawn.
It also decides the base power level of spawned weapons and armor.

Oftentimes, the difficulty tracker will be part-way towards the next level in experience points.
For example, if the difficulty tracker is at level 4 and has 10% of the experience points needed for level 5, it will effectively be considered level 4.1 for the purpose of spawning monsters, weapons and armor.
If an integer level is needed, the `Difficulty::get_round_random` function in the `src/experience.rs` file will return something like "5" 10% of the time and "4" the remaining 90% of the time.
Spawning code will also often want the direct "4.1" value, retrieved via the `Difficulty::as_f32` function in the same file.
Such code will often use the `experience::f32_round_random` helper function to turn it into an integer after processing if needed.

## Monster Selection

The level of a spawned monster decides its name and appearance.
This data is stored in the `MONSTERS` array at the top of the `src/spawn.rs` file, which is consulted by the `spawn_random_monster_at` function in the same file.

The `spawn_random_monster_at` function doesn't always spawn a monster matching the level of the difficulty tracker; this would lead to a very monotonous dungeon population.
Instead, it considers the level provided by the difficulty tracker as the *highest* level to spawn a monster, then picks one of the following outcomes:

- 20% for a level-matching monster
- 40% for a monster 1 to 3 levels lower
- 40% for an even lower-level monster

The final level chosen decides the name and appearance for the monster.
The correct numbers for such a monster at the chosen level is filled in by the `spawn_monster` function with the help of the monster-related functions in the `src/experience.rs` file: `calc_monster_max_hp`, `calc_monster_attack`, `calc_monster_defense` and `calc_monster_exp`.

## Weapon and Armor Selection

The name and appearance of weapons is determined by the `WEAPONS` array near the top of the `src/spawn.rs` file.
The `spawn_weapon` function in the same file accepts the difficulty tracker's level externally at either the `spawn_random_item_at` or `spawn_guaranteed_equipment` functions.
The `WEAPONS` array is shorter than the `MONSTERS` array, so the `rescale_level` function in the same file is used to cycle through the `WEAPONS` array at a slower pace.

The name and appearance of armor works exactly like those of the weapons, except using an `ARMORS` array consulted by the `spawn_armor` function.

Weapons and armor spawned by the `spawn_random_item_at` function are granted a +1 to +3 bonus to their power, but this doesn't affect the name and appearance chosen for them.

## Picking the Final Dungeon Level

The `MONSTERS` array at the top of the `src/spawn.rs` file has 25 entries.
Once the difficulty tracker has allowed them all to spawn, there are no new monsters to be seen; the player has effectively seen all of the content the game has to offer.
Since monsters are chosen directly from the level of the difficulty tracker, this point is reached once the difficulty tracker reaches level 25.
If this point has been reached when a new map is spawned by the `map::generate_rooms_and_corridors` function in the `src/map.rs` file, the downstairs is replaced by the location for the victory item that ends the game.
Depending on how many monsters spawn over the course of the game, the final dungeon depth usually ends up being between 25 to 30 levels deep.
