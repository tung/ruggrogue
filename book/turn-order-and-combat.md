# Turn Order and Combat

RuggRogue is a turn-based game, so the game waits until the player finishes their turn, then gives the monsters a turn.
There's no special speed system, so turns just alternate between the player and the monsters.

The most common interaction between the player and the monsters during their turns is performing a *melee attack*.
These melee attacks inflict damage to their target until they eventually die, and these dead targets need special handling and clean-up.

## Alternating Turns Between the Player and the Monsters

Time advances in the game with repeated calls to the `DungeonMode::update` function in the `src/modes/dungeon.rs` file.
For the purpose of understanding turns, it looks roughly like this:

```rust,ignore
impl DungeonMode {
    pub fn update(&mut self, world, inputs, _grids, pop_result) -> (ModeControl, ModeUpdate) {
        if world.run(player::player_is_alive) {
            // ...
            let time_passed: bool = if let Some(result) = pop_result {
                //
                // Dialog and menu result handling here.
                //
            } else {
                match player::player_input(world, inputs) {
                    //
                    // Player input result handling here.
                    //
                }
            };

            if time_passed {
                world.run(damage::handle_dead_entities);
                world.run(experience::gain_levels);
                // field of view stuff...
                world.run(monster::enqueue_monster_turns);

                if world.run(player::player_is_alive) {
                    monster::do_monster_turns(world);
                    world.run(damage::handle_dead_entities);
                    world.run(experience::gain_levels);
                    // field of view stuff...

                    if world.run(player::player_is_alive) {
                        // hunger handling...
                        world.run(damage::handle_dead_entities);
                        world.run(experience::gain_levels);
                        // field of view stuff...

                        if world.run(player::player_is_alive) {
                            world.run(damage::clear_hurt_bys);
                            world.borrow::<UniqueViewMut<TurnCount>>().0 += 1;
                        }
                    }
                }

                // ...
            }

            // ...
        } else if player::player_is_dead_input(inputs) {
            (
                ModeControl::Switch(GameOverMode::new().into()),
                ModeUpdate::Immediate,
            )
        } else {
            // ...
        }
    }
}
```

The most important thing to pick up in the above code skeleton is the `time_passed` variable.
The player's turn consists of everything that they're allowed to do while `time_passed` is set to `false`.
Once the player performs a time-consuming action, the `time_passed` variable is set to `true`.

The `time_passed` variable is either set directly after player input handling, or indirectly after handling the result of a dialog or menu.
A return value of `PlayerInputResult::TurnDone` from the `player::player_input` function sets `time_passed` to `true`, while `PlayerInputResult::NoResult` sets it to `false`.
The other variants of `PlayerInputResult` defined at the top of the `src/player.rs` file will cause the `DungeonMode::update` function to create a dialog or menu to show.
Confirming or performing actions in these dialogs and menus usually sets `time_passed` to `true`, while cancelling out sets it to `false`.

Once the `time_passed` variable is set to `true`, the `if time_passed ...` conditional block performs the rest of the turn:

1. Handle dead entities and level gains.
2. Give monsters their turn, handle dead entities and level gains again.
3. Tick hunger, handle dead entities and level gains yet again.
4. Clear damage book-keeping and increment the turn counter.

Note that dead entities and experience levels need to be dealt with at each point after damage can occur.

## Monster Turns

Once the `time_passed` variable in the `DungeonMode::update` function is set to `true`, the monsters get their turn.
All monsters are given a turn with a call to the `monster::enqueue_monster_turns` function in the `src/monster.rs` file.
Its job is to fill in the `MonsterTurns` queue with the entity ID of each monster.
The `monster::do_monster_turns` function then pops entity IDs out to give each monster their turn.

Why not just loop through all monsters, handle their turns directly and avoid the need for a queue?
The answer to this is that the `MonsterTurns` queue grants turns to monsters closest to the player first to minimize blocking when a group of monsters chase the player down a corridor.
At the top of the `src/monster.rs` file, the `MonsterTurns` queue is declared as a heap that stores monster entity IDs and their distance from the player.
Since `MonsterTurns` is a heap, monster IDs are popped out closest-first, giving the desired monster turn order.

Each monster's turn is individually handled by the `do_turn_for_one_monster` function in the `src/monster.rs` file.
Monster AI is trivial: if the player can be seen, move towards them as described in the [Pathfinding chapter](pathfinding.md) and perform a melee attack if adjacent.

## Melee Attacks and Damage

If the player moves into a monster or vice versa, a melee attack is performed.
Melee attacks are handled by the `damage::melee_attack` function in the `src/damage.rs` file.
Player melee attacks call this in the `try_move_player` function in the `src/player.rs` file.
Monster melee attacks call this in the `do_turn_for_one_monster` function in the `src/monster.rs` file.

The first consideration of the `damage::melee_attack` function is accuracy.
There is a flat 10% miss chance for any attack against a target as long as they aren't asleep.

The `damage::melee_attack` function deals with two entities that each have a `CombatStats` component, defined in `src/components.rs` like so:

```rust,ignore
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub attack: f32,
    pub defense: f32,
}
```

Notice that the `attack` and `defense` fields are typed `f32`, i.e. 32-bit single precision floating point numbers.
This allows for much finer-grained control of their values compared to if they were integers instead.

The player has an `Equipment` component that points to any equipped weapon or armor:

```rust,ignore
pub struct Equipment {
    pub weapon: Option<EntityId>,
    pub armor: Option<EntityId>,
}
```

Any such weapon or armor has a `CombatBonus` component:

```rust,ignore
pub struct CombatBonus {
    pub attack: f32,
    pub defense: f32,
}
```

The `damage::melee_attack` function thus calculates attack and defense values by starting with their base values in the `CombatStats` component, and adding bonuses from the `CombatBonus` components of any equipped weapon and armor.

The base damage calculation considers the attack power of the attacker versus the defense of the target.
The code looks like this:

```rust,ignore
let mut damage = if attack_value >= defense_value * 2.0 {
    attack_value - defense_value
} else {
    attack_value * (0.25 + (0.125 * attack_value / defense_value.max(1.0)).min(0.25))
};
```

There are two key take-aways of the above calculation.
First, if attack is at least twice defense, base damage is just the difference between the two values.
Second, to avoid zero damage when attack is less than defense, damage from lower attack values is varied from 25% to 50% of the attack value, depending on how much lower it is than the defense value.

This approach has some nice properties:

1. High attack values have a simple one-to-one relationship to damage.
2. Attacks still do a little bit of damage even if defense is higher than the attack value.
3. Low-damage attacks are still reduced by increases to defense.

After base damage has been calculated it has a 25% chance of being multiplied by 1.5 (a critical hit) and 25% chance of being multiplied by 0.5 (a weak hit).

At this point, the damage needs to be converted from a floating point number to an integer.
Fractional values are rounded up with the help of an RNG, e.g. 3.1 damage has a 10% chance of being rounded up to 4.

To inflict damage, the freshly-minted integer damage value is deducted from the `hp` field of the target's `CombatStats` component.
This may push the `hp` field to zero or negative, but entity death is handled elsewhere.
An appropriate hit message is also added to the message log, with suffix punctuation to match normal (exclamation mark), critical (double exclamation mark) and weak hits (period).

A damaged entity is given a `HurtBy` component that's defined like this in the `src/components.rs` file:

```rust,ignore
pub struct HurtBy {
    Someone(EntityId),
    Starvation,
}
```

In this case, the target entity is given a `HurtBy::Someone(attacker)`, where `attacker` is the entity ID of the attacker.
If the target is a monster, this will be used later on try to grant whoever killed it experience.
If the target is the player, this will point to the monster that killed them so it can be shown on the game over screen.
These `HurtBy` components are cleared off of all entities at the end of the turn back up in `DungeonMode::update` with a call to the `damage::clear_hurt_bys` function.

On the topic of the game over screen, the `damage::melee_attack` function also modifies any `Tally` component that it finds attached to the attacker or defender:

```rust,ignore
pub struct Tally {
    pub damage_dealt: u64,
    pub damage_taken: u64,
    pub kills: u64,
}
```

The `damage_dealt` of the attacker and the `damage_taken` of the target are incremented by the final damage value if either entity has a `Tally` component.
This information is also shown on the game over screen, so in practice only the player is given a `Tally` component.

## Handling Death

If an entity falls below zero hit points, it is now dead and needs to be handled appropriately.
This job falls upon the `damage::handle_dead_entities` function defined in `src/damage.rs`.

The `damage::handle_dead_entities` function goes through all entities with a `CombatStats` component and checks to see if their hit points are zero or less.
The entity IDs of any such entities are gathered in batches of ten each, then processed before taking up to another ten, etc.

A dead entity grants experience points to whoever hurt it last so long as the following conditions hold:

1. The dead entity is marked with a `HurtBy::Someone(...)` component.
2. The dead entity has a `GivesExperience` component, holding the number of experience points it should grant.
3. The entity referred to by the `HurtBy::Someone(...)` component has an `Experience` component to accept the granted experience points.

If the dead entity is a monster, it is removed from the map before the entity is deleted entirely.

If the dead entity is the player, "Press SPACE to continue..." is added to the message log, the `PlayerAlive` unique flag is set to `false`, any existing save file is deleted and any remaining dead entity handling is skipped.

The `PlayerAlive` unique flag is checked by the `player::player_is_alive` function that is checked all the way back in the `DungeonMode::update` function.
Once the player is dead, control flow in the `DungeonMode::update` function flows into the `player::player_is_dead_input` function defined in the `src/player.rs` file.
Its only job is to wait for the aforementioned Space key press, to allow the player to see the dungeon at the moment of their untimely passing.

After the Space key is pressed, the player is whisked away to the `GameOverMode` defined in `src/modes/game_over.rs` to be shown the game over screen with the cause of death, final stats and tallied damage and kills.
Proceeding from the game over screen takes the player back to the title screen.
