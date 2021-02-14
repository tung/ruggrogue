use shipyard::{
    AllStoragesViewMut, EntitiesView, EntityId, Get, Remove, UniqueView, UniqueViewMut, View,
    ViewMut, World,
};

use crate::{
    components::{
        CombatStats, Consumable, Inventory, Name, Position, ProvidesHealing, RenderOnFloor,
    },
    map::Map,
    message::Messages,
    player::PlayerId,
};

pub fn add_item_to_map(world: &World, item_id: EntityId, (x, y): (i32, i32)) {
    let (mut map, entities, mut positions, mut render_on_floors) = world.borrow::<(
        UniqueViewMut<Map>,
        EntitiesView,
        ViewMut<Position>,
        ViewMut<RenderOnFloor>,
    )>();

    entities.add_component(
        (&mut positions, &mut render_on_floors),
        (Position { x, y }, RenderOnFloor {}),
        item_id,
    );
    map.place_entity(item_id, (x, y), false);
}

pub fn remove_item_from_map(world: &World, item_id: EntityId) {
    let (mut map, mut positions, mut render_on_floors) = world.borrow::<(
        UniqueViewMut<Map>,
        ViewMut<Position>,
        ViewMut<RenderOnFloor>,
    )>();

    map.remove_entity(item_id, positions.get(item_id).into(), false);
    Remove::<(Position, RenderOnFloor)>::remove((&mut positions, &mut render_on_floors), item_id);
}

pub fn add_item_to_inventory(world: &World, picker_id: EntityId, item_id: EntityId) {
    let mut inventories = world.borrow::<ViewMut<Inventory>>();
    let picker_inv = (&mut inventories).get(picker_id);

    picker_inv.items.insert(0, item_id);
}

pub fn remove_item_from_inventory(world: &World, holder_id: EntityId, item_id: EntityId) {
    let mut inventories = world.borrow::<ViewMut<Inventory>>();
    let holder_inv = (&mut inventories).get(holder_id);

    if let Some(inv_pos) = holder_inv.items.iter().position(|id| *id == item_id) {
        holder_inv.items.remove(inv_pos);
    }
}

pub fn use_item(world: &World, user_id: EntityId, item_id: EntityId) {
    let player_id = world.run(|player_id: UniqueView<PlayerId>| player_id.0);

    world.run(
        |mut msgs: UniqueViewMut<Messages>,
         mut combat_stats: ViewMut<CombatStats>,
         names: View<Name>,
         provides_healings: View<ProvidesHealing>| {
            if combat_stats.contains(user_id) {
                let stats = (&mut combat_stats).get(user_id);

                if provides_healings.contains(item_id) {
                    let ProvidesHealing { heal_amount } = &provides_healings.get(item_id);

                    stats.hp = (stats.hp + heal_amount).min(stats.max_hp);
                    if user_id == player_id {
                        msgs.add(format!(
                            "{} uses {} and heals {} hp.",
                            names.get(user_id).0,
                            names.get(item_id).0,
                            heal_amount,
                        ));
                    }
                }
            }
        },
    );

    if world.borrow::<View<Consumable>>().contains(item_id) {
        remove_item_from_inventory(world, player_id, item_id);
        world.borrow::<AllStoragesViewMut>().delete(item_id);
    }
}
