use shipyard::{
    AllStoragesViewMut, EntitiesView, EntityId, Get, Remove, UniqueView, UniqueViewMut, View,
    ViewMut, World,
};

use crate::{
    components::{
        AreaOfEffect, CombatStats, Consumable, InflictsDamage, Inventory, Monster, Name, Player,
        Position, ProvidesHealing, RenderOnFloor,
    },
    map::Map,
    message::Messages,
};
use ruggle::FovShape;

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

pub fn use_item(world: &World, user_id: EntityId, item_id: EntityId, target: Option<(i32, i32)>) {
    world.run(
        |map: UniqueView<Map>,
         mut msgs: UniqueViewMut<Messages>,
         aoes: View<AreaOfEffect>,
         mut combat_stats: ViewMut<CombatStats>,
         inflicts_damages: View<InflictsDamage>,
         monsters: View<Monster>,
         names: View<Name>,
         players: View<Player>,
         positions: View<Position>,
         provides_healings: View<ProvidesHealing>| {
            let center = target.unwrap_or_else(|| positions.get(user_id).into());
            let radius = aoes.try_get(item_id).map_or(0, |aoe| aoe.radius);
            let targets = ruggle::field_of_view(&*map, center, radius, FovShape::CirclePlus)
                .flat_map(|(x, y, _)| map.iter_entities_at(x, y))
                .filter(|id| monsters.contains(*id) || players.contains(*id));
            let user_name = &names.get(user_id).0;
            let item_name = &names.get(item_id).0;

            msgs.add(format!("{} uses {}.", user_name, item_name));

            for target_id in targets {
                let target_name = &names.get(target_id).0;

                if let Ok(stats) = (&mut combat_stats).try_get(target_id) {
                    if let Ok(ProvidesHealing { heal_amount }) = &provides_healings.try_get(item_id)
                    {
                        stats.hp = (stats.hp + heal_amount).min(stats.max_hp);
                        msgs.add(format!(
                            "{} heals {} for {} hp.",
                            item_name, target_name, heal_amount,
                        ));
                    }

                    if let Ok(InflictsDamage { damage }) = &inflicts_damages.try_get(item_id) {
                        stats.hp -= damage;
                        msgs.add(format!(
                            "{} hits {} for {} hp.",
                            item_name, target_name, damage,
                        ));
                    }
                }
            }
        },
    );

    if world.borrow::<View<Consumable>>().contains(item_id) {
        remove_item_from_inventory(world, user_id, item_id);
        world.borrow::<AllStoragesViewMut>().delete(item_id);
    }
}
