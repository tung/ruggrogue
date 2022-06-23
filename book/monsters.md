# Monsters

This is a mini-chapter that exists mostly for the sake of completeness.
The truth about monsters in RuggRogue is they mostly overlap with the player, with a small handful of differences:

- Their `Monster` tag component gives them turns between player turns.
- They move towards and fight the player if they can see the player.
- They have no `Stomach` component, so they don't eat or regenerate.
- They grant experience when they die to whoever defeated them.
- They do not pick up, drop or use items.

Monsters differ only in name, appearance and stats; they're treated uniformly in every other way.

## Monster List

The following is a list of monsters and their ASCII representations in the approximate order that they'll be encountered by the player:

- (`b`) Blob
- (`B`) Bat
- (`c`) Crab
- (`S`) Snake
- (`g`) Goblin
- (`k`) Kobold
- (`G`) Gnome
- (`o`) Orc
- (`u`) Unicorn
- (`P`) Pirate
- (`L`) Lizardman
- (`G`) Ghost
- (`Z`) Skeleton
- (`O`) Ogre
- (`N`) Naga
- (`W`) Warlock
- (`&`) Demon
- (`E`) Sentinel
- (`R`) Robber
- (`K`) Skateboard Kid
- (`J`) Jellybean
- (`A`) Alien
- (`D`) Dweller
- (`h`) Little Helper
- (`H`) Big Helper

The monster list exists in the form of the `MONSTERS` array near the top of the `src/spawn.rs` file.
The ASCII symbols are mapped to monsters in the `Symbol::text_fallback` function that can be found in the `src/gamesym.rs` file.

## Monsters in Other Chapters

The topic of monsters is covered across other chapters in this book:

- [Map Population](map-population.md): Where monsters are spawned and how many appear.
- [Experience and Difficulty](experience-and-difficulty.md): Choice of appearance and power level of monsters, and granting experience when defeated.
- [Field of View](field-of-view.md): Monsters have their own fields of view, and will pursue the player on sight.
- [Pathfinding](pathfinding.md): Monsters step towards the player by first finding a path to follow.
- [Turn Order and Combat](turn-order-and-combat.md): Monsters get a turn between player turns and fight the player in melee combat.
