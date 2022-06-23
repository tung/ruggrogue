# Items

Items are scattered throughout the dungeon of RuggRogue; they reward the player for exploration.
The player can pick up and drop items to add and remove them from their inventory.
Weapons and armor are items that can be equipped by the player, conferring bonuses during combat.
Other items can be applied (used) for a variety of effects, such as healing the player, hurting monsters and inflicting status effects.

This chapter starts by giving a run-down of the items that exist in RuggRogue, their effects and how often they're found, with source code references sprinkled throughout.
Following this is an overview of the interface used to interact with items, and how items move between the map, the player's inventory and equipment.
Finally, targeting and effects of items that can be applied are described.

## List of Items

All items are spawned by calling functions named like `spawn_foo` in the `src/spawn.rs` file, where `foo` is the name of the item.
The list is as follows:

- **Present** (`spawn_present`) - The player wins the game when this item is used.
- **Ration** (`spawn_ration`) - Consumable; restores 750 nutrition to the player.
- **Health Potion** (`spawn_healh_potion`) - Consumble; restores 20 hit points if the player is hurt, or increases maximum hit points by 2 otherwise.
- **Magic Missile Scroll** (`spawn_magic_missile_scroll`) - Consumable; inflicts 8 damage to a single target up to 6 tiles away.
- **Fireball Scroll** (`spawn_fireball_scroll`) - Consumble; inflicts 20 damage to targets in a 3-tile area of effect up to 6 tiles away.
- **Sleep Scroll** (`spawn_sleep_scroll`) - Consumable; inflicts the sleep status effect to targets in a 1-tile area of effect up to 6 tiles away.
- **Weapon** (`spawn_weapon`) - Equipped in the "Weapon" slot; provides a bonus to attack.
- **Armor** (`spawn_armor`) - Equipped in the "Armor" slot; provides a bonus to defense.

Note that weapons only vary by appearance and combat bonuses and so are treated as a single item type; likewise for armor.

## Item Distribution

Items spawn in one of two broad ways: by room and by level.

When a map is being populated, one in four rooms will spawn items, following the logic in the `fill_room_with_spawns` function in the `src/spawn.rs` file.
Each such room generally spawns a single item, but may spawn an additional one for every win the player has accumulated.

The distribution of room items is determined by the `spawn_random_item_at` function, and looks like this:

- 1 / 11 - a weapon or armor with an extra +1 to +3 power bonus
- 3 / 11 - Health Potion
- 3 / 11 - Magic Missile Scroll
- 2 / 11 - Fireball Scroll
- 2 / 11 - Sleep Scroll

Each level spawns a single Ration with the help of the `spawn_guaranteed_ration` function.
The `spawn_guaranteed_equipment` function spawns a starting weapon and armor on the first level, and depth-appropriate weapon and armor at irregular depth intervals.
The exact process for all of this is described in detail in the [Map Population](map-population.md) chapter.

## Inventory and Equipment

For the player to pick up and use items, they need an *inventory* to serve as space to hold them.
The list of items carried by the player is represented by the `Inventory` component, defined in the `src/components.rs` file:

```rust,ignore
pub struct Inventory {
    pub items: Vec<EntityId>,
}
```

The game will only allow entities to be picked up if they are marked with the `Item` tag component:

```rust,ignore
pub struct Item;
```

The player can equip certain items from their inventory.
The player's equipment slots are represented by the `Equipment` component:

```rust,ignore
pub struct Equipment {
    pub weapon: Option<EntityId>,
    pub armor: Option<EntityId>,
}
```

Note that equipment slots are separate from the inventory, so for example, equipping a weapon moves it *out* of the inventory and into the player's "Weapon" slot.
Items that can be equipped are marked with an `EquipSlot` component:

```rust,ignore
pub struct EquipSlot {
    Weapon,
    Armor,
}
```

The exact value of the `EquipSlot` component determines which equipment slot the item will be moved into when it is equipped.

## Item User Interface

Most of the time spent playing the game that isn't moving around the dungeon or fighting monsters is spent dealing with items.
As a result, about half of the interface code is dedicated solely to dealing with items.

Although these item-related menus and dialogs allude to actions, note that they *never perform these actions themselves*.
For example, if the player chooses to drop an item from the inventory, the inventory returns a *result* that captures that intent, and the job of dropping the item is handled elsewhere.

### Pick Up Menu

The player presses either the 'g' or the Comma key whenever they want to pick up an item at their position.
This pushes onto the mode stack the `PickUpMenuMode`, defined in the `src/modes/pick_up_menu.rs` file.

If the player isn't standing over any items, they're given a message saying as much and no menu appears.

If the player is standing over at least one item, a menu appears, allowing them to select an item on the map to be picked up.
This menu has cursor-based controls, along with most of the menus and dialogs in the game.
The entity ID of the selected item is returned as part of the `PickUpMenuModeResult`.

### Inventory Menu

The player presses the 'i' key to bring up the inventory menu.
This is the biggest and most advanced of the menus, represented as the `InventoryMode` in the `src/modes/inventory.rs` file.
It shows the player's currently-equipped weapon and armor in a small section at the top, with a larger inventory listing beneath it.
There's also an option to sort the inventory, which sorts inventory items according to hard-coded criteria.

If an inventory item is selected, an *inventory action menu* is presented for it; a similar *equipment action menu* is presented if an equipped item is selected.
Any action returned by either of these menus is relayed back as an `InventoryModeResult` with the item's entity ID, the action and a target location for items usable at range.

### Inventory Action Menu and Equipment Action Menu

Selecting an inventory item presents an inventory action menu, represented by the `InventoryActionMode` in the `src/modes/inventory_action.rs` file.
It shows a list of possible actions that can be performed with the item, such as "Equip", "Apply" and "Drop".
If one of these actions is chosen, it will be returned in the form of an `InventoryActionModeResult`.

Selecting an equipped weapon or armor in the inventory menu brings up the equipment action menu, represented by the `EquipmentActionMode` in the `src/modes/equipment_action.rs` file.
It presents "Remove" and "Drop" as possible actions, but otherwise works much the same as the `InventoryActionMode`.
An `EquipmentActionModeResult` holds the selected action in this case.

### Ranged Item Targeting Mode

If "Apply" is chosen in the inventory action menu for an item that is usable at range, a targeting mode needs to be brought up to choose a target location.
This is represented by the `TargetMode`, defined in the `src/modes/target.rs` file.

Unlike everything listed up to this point, the `TargetMode` is *not* a menu!
Instead, it draws the map and interface much like the `DungeonMode` in which most of the gameplay occurs.
However, instead of performing actions and taking turns, the `TargetMode` highlights valid target tiles for the ranged item it was invoked for.

The movement keys move around a cursor that allows the player to choose a target location out of the valid target tiles.
This selected target location is returned as part of the `TargetModeResult`.

### Shortcut Menus

The inventory menu allows interacting with items in the player's possession through a single centralized menu, but players who already know what they want to do may find this cumbersome.
RuggRogue provides shortcut menus to bypass the inventory in this case, brought up with these shortcut keys:

- 'a' to "Apply"
- 'e' or 'w' to "Equip"
- 'r' to "Remove"
- 'd' to "Drop"

Shortcut keys and actions are associated in the `InventoryAction::from_key` function in the `src/modes/inventory_action.rs` file, separate from the definition of these shortcut menus.

This shortens interaction from "Inventory -> Item -> Action" to just "Action -> Item".
Pressing one of these keys brings up the `InventoryShortcutMode`, defined in the `src/modes/inventory_shortcut.rs`.
This presents a prompt such as "Apply which item?" with a list of inventory items narrowed down to only those with which the action can be performed.
The chosen item is encapsulated in the `InventoryShortcutModeResult` along with the action that brought the shortcut menu up in the first place.

The "Remove" action brings up a similar `EquipmentShortcutMode` that returns an `EquipmentShortcutModeResult`.

Note that the "Drop" shortcut menu only lists inventory items even though equipment can be directly dropped while equipped.
This is a technical limitation due to how this was originally designed.

The shortcut keys are used beyond these shortcut menus.
For example, pressing any of them in the inventory will bring up the inventory action menu with the matching action pre-selected.
Further, pressing them in the inventory action menu will move the cursor to the matching action, or confirm the action if it's already selected.

### Menu Memory

When a menu that deals with items is closed, the position of the cursor will be remembered upon reopening the menu.
This makes it easier to perform the same action on a sequence of items in menus.

This menu cursor memory is stored in the `MenuMemory` unique defined in the `src/menu_memory.rs` file.
It's simply an array with an entry for a cursor position for each of the different item menus that the player will see:

- `InventoryMode`
- `InventoryShortcutMode` for "Equip"
- `InventoryShortcutMode` for "Apply"
- `InventoryShortcutMode` for "Drop"
- `EquipmentShortcutMode` for "Remove"
- `EquipmentShortcutMode` for "Drop" (unused as described in the previous section)
- `PickUpMenuMode`

For `PickUpMenuMode`, the `MenuMemory` will recall the map coordinates of the last time the `PickUpMenuMode` appeared.
If the player's coordinates differ from last time, the `PickUpMenuMode` will reset the cursor memory.

## Moving Items Around

While the user interface permits the player to choose actions to perform with items, the task of performing the actions themselves falls upon mode result handling logic near the top of the `DungeonMode::update` function in the `src/modes/dungeon.rs` file.
The most fundamental of these actions are the ones that simply move items around, such as picking up, dropping and equipping items.

Items exist in one of three places:

1. on the map,
2. in an inventory, or
3. equipped as a weapon or armor.

An item on the map is like any other entity, and thus must have a `Coord` component with its map coordinates, as well as be present in the map's spatial cache (the `tile_entities` field of the `Map` struct).
For the item to be visible but not over the player or any monsters, it must also have a `RenderOnFloor` tag component.

An item in an inventory is listed by its entity ID in the `items` vector of the `Inventory` component.

An item equipped as a weapon has its entity ID set in the `weapon` field of the relevant `Equipment` component.
An equipped armor item is set to the `armor` field instead.

**Picking up** an item moves it from the map to the player's inventory.
The `player::player_pick_up_item` function in the `src/player.rs` file encapsulates this action, calling upon the `item::remove_item_from_map` and `item::add_item_to_inventory` functions defined in the `src/item.rs` file to do the heavy lifting.

**Dropping** an item moves it from the player's inventory to the map.
The `player::player_drop` item function in the `src/player.rs` file handles this with the help of the `item::remove_item_from_inventory` and `item::add_item_to_map` functions defined in the `src/item.rs` file.
Equipment is dropped by the `item::drop_equipment` function with the help of an `unequip_item` helper function, both also in the `src/item.rs` file.

**Equipping** an item moves it from the inventory to an equipment slot.
This is the task of the `item::equip_item` function in the `src/item.rs` file.
The `EquipSlot` component of the item is checked here to determine if the item should be equipped as a weapon or as armor.

**Unequipping** an item moves it from an equipment slot back to the inventory.
This is handled by the `item::remove_equipment` function in the `src/item.rs` file.
This first unequips the item using the aforementioned `unequip_item` helper function, then moves it to the inventory with the `item::add_item_to_inventory` function.

## Using Items

The "Apply" action can be used on an item that is marked with either the `Consumable` or `Victory` tag components.
If one of the item-related menus requests that an item be applied, the `DungeonMode::update` function will handle it by calling the `item::use_item` function defined in the `src/item.rs` file.

### Victory

The first thing the `item::use_item` function checks is if the item in question has a `Victory` tag component and the user is the player.
If these conditions are true, the player has won the game!

The first step of handling victory is to immediately perform an auto-save.
The victory sequence involves *switching* from the `DungeonMode` to the `GameOverMode`.
If the game window closes for whatever reason, there's no confirmation dialog or any chance to save the game, since those tasks would normally be handled by the `DungeonMode`.
Auto-saving in advance mitigates this issue.

The victory item is then deleted and the win counter is incremented.
The `item::use_item` function returns a result that signals to the `DungeonMode::update` function that it should switch to the `GameOverMode`.

The `GameOverMode` defined in the `src/modes/game_over.rs` file detects that the player is still alive and has thus won the game, showing a congratulatory message in response.
Proceeding from the `GameOverMode` switches back to the `DungeonMode` to start a New Game Plus run.

### Gathering Targets

Back in the `item::use_item` function, most items won't win the game outright, so they must have some sort of effect.
The first thing to do in this case is to figure out which entities should be affected by the item.

Items with a `Ranged` component will already have target map coordinates chosen previously.
Items that aren't used at range imply self-use; the coordinates of the entity using the item are used in this case.

Affected entities are gathered by calling the `ruggrogue::field_of_view` function, centered about the target location.
The radius of this field is either zero for just the target tile, or a non-zero value extracted from the `AreaOfEffect` component attached to the item.
Using field of view calculation to determine targets like this prevents items with an area of effect from blasting through walls.

### Applying Item Effects

Items can have any number of effects according to the components attached to the item and the targets.

**Nutrition** is added to the target if the target entity has a `Stomach` component to fill.
The `fullness` field of that `Stomach` component is filled according to the amount stated in the item's `Nutrition` component that found only on rations.

**Healing** is applied if the item has a `ProvidesHealing` component and the target has a `CombatStats` component.
If the target is at less than full health, the hit points of that target are restored by the amount stated in the `ProvidesHealing` component, up to maximum hit points.
If the target is at full health, their hit points and maximum hit points are increased by two.

**Damage** is applied if the item has an `InflictsDamage` component and the target has a `CombatStats` component.
This does a few things:

1. The target's hit points are reduced according to the amount stated in the `InflictsDamage` component.
2. The target is given a `HurtBy::Someone(id)` component, where `id` is the entity ID of the item user so they can be credited if the target dies.
3. If the user has a `Tally` component, add the damage amount to the `damage_dealt` field.
4. If the target has a `Tally` component, add the damage amount to the `damage_taken` field.

**Sleep** is applied if the item has an `InflictsSleep` component and the target has a `CombatStats` component.
It adds the `Asleep` component to the target with a `sleepiness` amount determined by the `InflictsSleep` component.

Once all targets have been processed, if the item is marked with the `Consumable` tag component it is removed from the inventory of its user and then destroyed.

## The Sleep Status Effect

The sleep status effect renders the target unable to do anything other than pass turns until it wears off.
It affects both the player and monsters.

In the case of a sleeping player, there is sleep-related input handling near the top of the `player::player_input` function in the `src/player.rs` file.
In this state, almost every key press will cause the player to automatically pass their turn.
The wearing off of the sleep status is handled by calling the `item::handle_sleep_turn` function back in the `src/item.rs` file.

Sleeping monsters are dealt with by the `do_turn_for_one_monster` function in the `src/monster.rs` file.
A monster that is asleep simply calls the `item::handle_sleep_turn` function and bypasses the usual monster AI.

The `item::handle_sleep_turn` function is responsible for counting down the sleepiness of sleep-afflicted entities and waking them back up.
The sleepiness counter of the entity is decremented according to the following rules:

- -1 per turn.
- -1 if the entity is the player and a monster is in their field of view, or vice versa.
- -10 if the entity lost hit points since their last turn; the `Asleep` component tracks changes to hit points to detect this.

Once the sleepiness counter reaches zero, the `Asleep` component is removed from the entity, waking them up.

The Sleep Scroll inflicts 36 points of sleepiness, by its construction in the `spawn_sleep_scroll` function back in the `src/spawn.rs` file.
This renders one sleeping monster vulnerable to three hits before waking up if the player wastes no turns to attack them.
