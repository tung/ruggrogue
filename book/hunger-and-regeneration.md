# Hunger and Regeneration

RuggRogue features a *hunger* mechanic: the player has a 'stomach fullness' level that slowly depletes over time.
This fullness can be restored by finding and eating *rations* that spawn as consumable items on each level.
If the player stays well-fed they will slowly regenerate hit points; conversely, being very hungry stops hit point regeneration.
If fullness drops to zero, the player is considered starving and will *lose* hit points over time; this can kill the player.

The hunger mechanic serves as a soft timer that urges the player to move forward instead of staying in any one place for too long.
In addition, rations act as an extra incentive to explore unseen areas.

## Stomach and Nutrition

The fullness level of the player is stored in a `Stomach` component that looks like this in the `src/components.rs` file:

```rust,ignore
pub struct Stomach {
    pub fullness: i32,
    pub max_fullness: i32,
    pub sub_hp: i32,
}
```

The `fullness` field is the exact internal fullness level of the player; zero means the player is starving, while the `max_fullness` field simply caps its value.

The `sub_hp` field is used to determine when the player should regenerate a hit point (or lose one, in case of starvation).
It can be thought of as a 'fractional hit point', serving as the numerator with the denominator decided elsewhere.
This is explained later in this chapter.

The player is the only entity in the game with a `Stomach` component and is thus the only entity that can regenerate hit points or starve.
The player is created with a `Stomach` component with `fullness` and `max_fullness` both set to 1500 in the `spawn_player` function in the `src/spawn.rs` file.

The player can restore the fullness of their stomach by eating a ration.
The `Nutrition` component attached to rations is what enables this:

```rust,ignore
pub struct Nutrition(pub i32);
```

A ration provides 750 points of nutrition, as per its creation in the `spawn_ration` function in the `src/spawn.rs` file.
Rations work like any other consumable item, restoring nutrition as an effect in the `item::use_item` function in the `src/item.rs` file.

## Hunger States

Fullness is tracked internally as a continuous value, like hit points, but all of the visible effects of hunger involve first converting that value into a discrete *hunger state*.
This is represented by the `HungerState` enum defined in the `src/hunger.rs` file:

```rust,ignore
enum HungerState {
    Starving,
    Famished,
    VeryHungry,
    Hungry,
    Normal,
    Full,
}
```

Each variant maps to a range of `i32` values according to the `HungerState::from` function as follows:

- `Starving`: 0
- `Famished`: 1 to 150
- `VeryHungry`: 151 to 300
- `Hungry`: 301 to 750
- `Normal`: 751 to 1200
- `Full`: 1201 and above

These values are hard-coded and probably should have been calculated relative to the `max_fullness` field of the `Stomach` struct, but since there's only one `Stomach` component in the whole game, it's been left as-is.

The hunger state is what appears in the sidebar, not the raw value.
The hunger state determines whether the player regenerates or loses hit points, and messages appear in the message log when it changes.

## Sidebar Hunger Display

The hunger state appears in the sidebar in a label that looks like "Hunger: Normal".
This hunger state label is drawn along with other player status information by the `draw_status` function in the `src/ui.rs` file.

The label and its foreground and background colors are determined by consulting the `hunger::player_hunger_label` function back in the `src/hunger.rs` file.
Different foreground and background colors are used to draw the player's attention to their hunger state according to urgency.

## Regeneration and Starvation

There are two hunger-related tasks that must be handled on a per-turn basis: decrementing fullness and changing hit points based on hunger state.
These tasks are handled by the `hunger::tick_hunger` function found at the bottom of the `src/hunger.rs` file.
This function is called by the `DungeonMode::update` function in the `src/modes/dungeon.rs` file after handling of monster turns.

The `hunger::tick_hunger` function depletes one point of fullness per turn.
If the player's hunger state and hit points allow them to regenerate, an extra point of fullness is deducted from their stomach.

The `hunger::tick_hunger` function is also responsible for changing hit points, either raising them for regeneration or depleting them for starvation.
However, even though the function is called every turn, we don't want to alter hit points every turn.
Instead, the function deals with what we could consider a 'partial' hit point in the form of the `sub_hp` field of the `Stomach` component.
The field is used by the `hunger::tick_hunger` function to regenerate hit points as follows:

1. The player's fullness level is converted into a `HungerState`.
2. The `HungerState::turns_to_regen_to_max_hp` function is consulted.
3. If it returned a number, add the maximum hit points of the player to the `sub_hp` field.
4. If the `sub_hp` field is at least the value from step 2, integer divide `sub_hp` by the step 2 value, add the quotient to the player's hit points and keep the remainder in `sub_hp`.

For example, if the player has 60 maximum hit points, is hurt and full, the `HungerState::turns_to_regen_to_max_hp` function will return `Some(300)`.
This adds 60 to the `sub_hp` field each turn.
When it reaches 300, the player regenerates a single hit point and 300 is subtracted from the `sub_hp` field.
Since 300 divided by 60 produces 5, this player will regenerate a hit point every five turns, and indeed will be able to regenerate their full 60 hit points in 300 turns.

The hunger states that permit regeneration are decided by whether or not the `HungerState::turns_to_regen_to_max_hp` function returns a number.
The player can only regenerate when their hunger state is "Full", "Normal" or "Hungry".

When the player's hunger state is "Starving", they will *lose* hit points instead of regenerating them.
The process plays in reverse: the `HungerState::turns_to_starve_from_max_hp` function is used instead, while the `sub_hp` and player's hit points are subtracted from instead of added to.
According to the `HungerState::turns_to_starve_from_max_hp` function, the player will lose their maximum worth of hit points in 400 turns spent in the "Starving" hunger state.

Since starving causes damage, the `hunger::tick_hunger` function is responsible for tracking the damage taken in the player's `Tally` component.
It is possible for starvation to kill the player, so there's a `HurtBy::Starvation` cause attached to the player entity in case of death to show on the game over screen.

## Hunger Messages

The `hunger::tick_hunger` function compares hunger state before and after deducting fullness.
If a change is detected and the new hunger state isn't just "Normal", a message chosen by the `HungerState::reduced_to` function is logged, producing something like "Player is getting hungry."

If the player loses hit points due to starvation, they're informed with a message: "Player aches with hunger!"

## Hunger and Auto-Run

Changes to hunger state and losing hits points to starvation not only produce messages, but also interrupt all forms of auto-run.

Hunger impacts the ability for the player to begin or continue resting in place, in the `wait_player` and `auto_run_next_step` functions found in the `src/player.rs` file respectively.
They both hinge on the value returned by the `hunger::can_regen` function back in the `src/hunger.rs` file.
It returns a variant of the `hunger::CanRegenResult` enum that can be found at the top of the `src/hunger.rs` file, which looks like this:

```rust,ignore
pub enum CanRegenResult {
    CanRegen,
    NoRegen,
    FullyRested,
    TooHungry,
}
```

The `CanRegen` variant allows resting in place to begin or continue; the other results represent reasons the player cannot regenerate hit points.
The `NoRegen` variant means the player has no `Stomach` component, which shouldn't happen in normal play.
`FullyRested` means the player's hit points are already at their maximum.
`TooHungry` means that the `HungerState::turns_to_regen_to_max_hp` function is producing `None` because the player's fullness is too low to allow for hit point regeneration.
