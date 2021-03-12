use shipyard::{
    AllStoragesViewMut, EntitiesView, EntityId, Get, Remove, UniqueView, UniqueViewMut, View,
    ViewMut, World,
};

use crate::{
    components::{
        AreaOfEffect, Asleep, CombatStats, Consumable, Coord, FieldOfView, InflictsDamage,
        InflictsSleep, Inventory, Monster, Name, Player, ProvidesHealing, RenderOnFloor,
    },
    map::Map,
    message::Messages,
    player::{self, PlayerId},
};
use ruggle::FovShape;

pub fn add_item_to_map(world: &World, item_id: EntityId, pos: (i32, i32)) {
    let (mut map, entities, mut coords, mut render_on_floors) = world.borrow::<(
        UniqueViewMut<Map>,
        EntitiesView,
        ViewMut<Coord>,
        ViewMut<RenderOnFloor>,
    )>();

    entities.add_component(
        (&mut coords, &mut render_on_floors),
        (Coord(pos.into()), RenderOnFloor {}),
        item_id,
    );
    map.place_entity(item_id, pos, false);
}

pub fn remove_item_from_map(world: &World, item_id: EntityId) {
    let (mut map, mut coords, mut render_on_floors) =
        world.borrow::<(UniqueViewMut<Map>, ViewMut<Coord>, ViewMut<RenderOnFloor>)>();

    map.remove_entity(item_id, coords.get(item_id).0.into(), false);
    Remove::<(Coord, RenderOnFloor)>::remove((&mut coords, &mut render_on_floors), item_id);
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
        |(map, mut msgs): (UniqueView<Map>, UniqueViewMut<Messages>),
         entities: EntitiesView,
         aoes: View<AreaOfEffect>,
         mut asleeps: ViewMut<Asleep>,
         mut combat_stats: ViewMut<CombatStats>,
         coords: View<Coord>,
         inflicts_damages: View<InflictsDamage>,
         inflicts_sleeps: View<InflictsSleep>,
         (monsters, names, players): (View<Monster>, View<Name>, View<Player>),
         provides_healings: View<ProvidesHealing>| {
            let center = target.unwrap_or_else(|| coords.get(user_id).0.into());
            let radius = aoes.try_get(item_id).map_or(0, |aoe| aoe.radius);
            let targets = ruggle::field_of_view(&*map, center, radius, FovShape::CirclePlus)
                .filter(|(_, _, symmetric)| *symmetric)
                .flat_map(|(x, y, _)| map.iter_entities_at(x, y))
                .filter(|id| monsters.contains(*id) || players.contains(*id));
            let user_name = &names.get(user_id).0;
            let item_name = &names.get(item_id).0;

            msgs.add(format!("{} uses {}.", user_name, item_name));

            for target_id in targets {
                let target_name = &names.get(target_id).0;

                if let Ok(stats) = (&mut combat_stats).try_get(target_id) {
                    if let Ok(ProvidesHealing { heal_amount }) = provides_healings.try_get(item_id)
                    {
                        stats.hp = (stats.hp + heal_amount).min(stats.max_hp);
                        msgs.add(format!(
                            "{} heals {} for {} hp.",
                            item_name, target_name, heal_amount,
                        ));
                    }

                    if let Ok(InflictsDamage { damage }) = inflicts_damages.try_get(item_id) {
                        stats.hp -= damage;
                        msgs.add(format!(
                            "{} hits {} for {} hp.",
                            item_name, target_name, damage,
                        ));
                    }

                    if let Ok(InflictsSleep { sleepiness }) = inflicts_sleeps.try_get(item_id) {
                        entities.add_component(
                            &mut asleeps,
                            Asleep {
                                sleepiness: *sleepiness,
                                last_hp: stats.hp,
                            },
                            target_id,
                        );
                        msgs.add(format!("{} sends {} to sleep.", item_name, target_name));
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

pub fn is_asleep(world: &World, who: EntityId) -> bool {
    world.borrow::<View<Asleep>>().contains(who)
}

pub fn handle_sleep_turn(world: &World, who: EntityId) {
    let mut asleeps = world.borrow::<ViewMut<Asleep>>();

    if let Ok(mut asleep) = (&mut asleeps).try_get(who) {
        let (mut msgs, player_id, combat_stats, coords, fovs, names) = world.borrow::<(
            UniqueViewMut<Messages>,
            UniqueView<PlayerId>,
            View<CombatStats>,
            View<Coord>,
            View<FieldOfView>,
            View<Name>,
        )>();

        asleep.sleepiness -= 1;
        if (who == player_id.0 && world.run(player::player_sees_foes))
            || player::can_see_player(world, who)
        {
            asleep.sleepiness -= 1;
        }
        if let Ok(stats) = combat_stats.try_get(who) {
            if stats.hp < asleep.last_hp {
                asleep.sleepiness -= 10;
                asleep.last_hp = stats.hp;
            }
        }

        if asleep.sleepiness <= 0 {
            let show_msg = if who == player_id.0 {
                true
            } else if let Ok(who_coord) = coords.try_get(who) {
                let player_fov = fovs.get(player_id.0);
                player_fov.get(who_coord.0.into())
            } else {
                false
            };

            asleeps.remove(who);
            if show_msg {
                msgs.add(format!("{} wakes up.", names.get(who).0));
            }
        }
    }
}
