use shipyard::{EntitiesView, EntityId, Get, Remove, UniqueViewMut, ViewMut, World};

use crate::{
    components::{Inventory, Position, RenderOnFloor},
    map::Map,
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
