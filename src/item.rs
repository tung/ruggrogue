use shipyard::{
    AllStoragesViewMut, EntitiesView, EntityId, Get, Remove, UniqueView, UniqueViewMut, View,
    ViewMut, World,
};
use std::cmp::Ordering;

use crate::{
    components::*,
    map::Map,
    message::Messages,
    player::{self, PlayerId},
};
use ruggle::FovShape;

pub fn add_item_to_map(world: &World, item_id: EntityId, pos: (i32, i32)) {
    let (mut map, entities, mut coords, mut render_on_floors) = world
        .borrow::<(
            UniqueViewMut<Map>,
            EntitiesView,
            ViewMut<Coord>,
            ViewMut<RenderOnFloor>,
        )>()
        .unwrap();

    entities.add_component(
        item_id,
        (&mut coords, &mut render_on_floors),
        (Coord(pos.into()), RenderOnFloor {}),
    );
    map.place_entity(item_id, pos, false);
}

pub fn remove_item_from_map(world: &World, item_id: EntityId) {
    let (mut map, mut coords, mut render_on_floors) = world
        .borrow::<(UniqueViewMut<Map>, ViewMut<Coord>, ViewMut<RenderOnFloor>)>()
        .unwrap();

    map.remove_entity(item_id, coords.get(item_id).unwrap().0.into(), false);
    (&mut coords, &mut render_on_floors).remove(item_id);
}

pub fn add_item_to_inventory(world: &World, picker_id: EntityId, item_id: EntityId) {
    let mut inventories = world.borrow::<ViewMut<Inventory>>().unwrap();
    let mut picker_inv = (&mut inventories).get(picker_id).unwrap();

    picker_inv.items.insert(0, item_id);
}

pub fn remove_item_from_inventory(world: &World, holder_id: EntityId, item_id: EntityId) {
    let mut inventories = world.borrow::<ViewMut<Inventory>>().unwrap();
    let mut holder_inv = (&mut inventories).get(holder_id).unwrap();

    if let Some(inv_pos) = holder_inv.items.iter().position(|id| *id == item_id) {
        holder_inv.items.remove(inv_pos);
    }
}

fn unequip_item(world: &World, unequipper_id: EntityId, item_id: EntityId) {
    let mut equipments = world.borrow::<ViewMut<Equipment>>().unwrap();
    let mut equipment = (&mut equipments).get(unequipper_id).unwrap();

    match world
        .borrow::<View<EquipSlot>>()
        .unwrap()
        .get(item_id)
        .unwrap()
    {
        EquipSlot::Weapon => equipment.weapon = None,
        EquipSlot::Armor => equipment.armor = None,
    };
}

pub fn remove_equipment(world: &World, remover_id: EntityId, item_id: EntityId) {
    if world
        .borrow::<View<Inventory>>()
        .unwrap()
        .contains(remover_id)
    {
        unequip_item(world, remover_id, item_id);
        add_item_to_inventory(world, remover_id, item_id);

        let mut msgs = world.borrow::<UniqueViewMut<Messages>>().unwrap();
        let names = world.borrow::<View<Name>>().unwrap();

        msgs.add(format!(
            "{} removes {}.",
            &names.get(remover_id).unwrap().0,
            &names.get(item_id).unwrap().0
        ));
    } else {
        // Remover has no inventory, so attempt dropping the equipment instead.
        drop_equipment(world, remover_id, item_id);
    }
}

pub fn drop_equipment(world: &World, dropper_id: EntityId, item_id: EntityId) {
    let dropper_pos: (i32, i32) = {
        let coords = world.borrow::<View<Coord>>().unwrap();
        coords.get(dropper_id).unwrap().0.into()
    };

    unequip_item(world, dropper_id, item_id);
    add_item_to_map(world, item_id, dropper_pos);

    let mut msgs = world.borrow::<UniqueViewMut<Messages>>().unwrap();
    let names = world.borrow::<View<Name>>().unwrap();

    msgs.add(format!(
        "{} drops {}.",
        &names.get(dropper_id).unwrap().0,
        &names.get(item_id).unwrap().0
    ));
}

pub fn equip_item(world: &World, equipper_id: EntityId, item_id: EntityId) {
    let mut equipments = world.borrow::<ViewMut<Equipment>>().unwrap();
    let mut equipment = (&mut equipments).get(equipper_id).unwrap();
    let equip_field = match world
        .borrow::<View<EquipSlot>>()
        .unwrap()
        .get(item_id)
        .unwrap()
    {
        EquipSlot::Weapon => &mut equipment.weapon,
        EquipSlot::Armor => &mut equipment.armor,
    };

    if equip_field.is_some() {
        let mut inventories = world.borrow::<ViewMut<Inventory>>().unwrap();
        let mut equipper_inv = (&mut inventories).get(equipper_id).unwrap();
        let item_pos = equipper_inv
            .items
            .iter()
            .position(|id| *id == item_id)
            .unwrap();

        // Swap the item IDs of the equip field and inventory position.
        equipper_inv.items[item_pos] = equip_field.replace(item_id).unwrap();
    } else {
        *equip_field = Some(item_id);
        remove_item_from_inventory(world, equipper_id, item_id);
    }

    let mut msgs = world.borrow::<UniqueViewMut<Messages>>().unwrap();
    let names = world.borrow::<View<Name>>().unwrap();

    msgs.add(format!(
        "{} equips {}.",
        &names.get(equipper_id).unwrap().0,
        &names.get(item_id).unwrap().0
    ));
}

pub fn sort_inventory(world: &World, holder: EntityId) {
    let aoes = world.borrow::<View<AreaOfEffect>>().unwrap();
    let combat_bonuses = world.borrow::<View<CombatBonus>>().unwrap();
    let inflicts_damages = world.borrow::<View<InflictsDamage>>().unwrap();
    let inflicts_sleeps = world.borrow::<View<InflictsSleep>>().unwrap();
    let names = world.borrow::<View<Name>>().unwrap();
    let provides_healings = world.borrow::<View<ProvidesHealing>>().unwrap();
    let nutritions = world.borrow::<View<Nutrition>>().unwrap();
    let rangeds = world.borrow::<View<Ranged>>().unwrap();
    let item_order = |&a: &EntityId, &b: &EntityId| -> Ordering {
        // Ration
        {
            let a_is_ration = nutritions.contains(a);
            let b_is_ration = nutritions.contains(b);

            if a_is_ration && b_is_ration {
                return Ordering::Equal;
            } else if a_is_ration {
                return Ordering::Less;
            } else if b_is_ration {
                return Ordering::Greater;
            }
        }

        // Health Potion
        {
            let a_is_heal = provides_healings.contains(a);
            let b_is_heal = provides_healings.contains(b);

            if a_is_heal && b_is_heal {
                return Ordering::Equal;
            } else if a_is_heal {
                return Ordering::Less;
            } else if b_is_heal {
                return Ordering::Greater;
            }
        }

        // Magic Missile Scroll
        {
            let a_is_mms = rangeds.contains(a) && !aoes.contains(a) && inflicts_damages.contains(a);
            let b_is_mms = rangeds.contains(b) && !aoes.contains(b) && inflicts_damages.contains(b);

            if a_is_mms && b_is_mms {
                return Ordering::Equal;
            } else if a_is_mms {
                return Ordering::Less;
            } else if b_is_mms {
                return Ordering::Greater;
            }
        }

        // Sleep Scroll
        {
            let a_is_sleep = inflicts_sleeps.contains(a);
            let b_is_sleep = inflicts_sleeps.contains(b);

            if a_is_sleep && b_is_sleep {
                return Ordering::Equal;
            } else if a_is_sleep {
                return Ordering::Less;
            } else if b_is_sleep {
                return Ordering::Greater;
            }
        }

        // Fireball Scroll
        {
            let a_is_fs = rangeds.contains(a) && aoes.contains(a) && inflicts_damages.contains(a);
            let b_is_fs = rangeds.contains(b) && aoes.contains(b) && inflicts_damages.contains(b);

            if a_is_fs && b_is_fs {
                return Ordering::Equal;
            } else if a_is_fs {
                return Ordering::Less;
            } else if b_is_fs {
                return Ordering::Greater;
            }
        }

        // Equipment
        {
            let a_cb = combat_bonuses.get(a);
            let b_cb = combat_bonuses.get(b);

            if let (Ok(a_cb), Ok(b_cb)) = (a_cb, b_cb) {
                let a_is_weapon = a_cb.attack > a_cb.defense;
                let b_is_weapon = b_cb.attack > b_cb.defense;

                // Weapon
                if a_is_weapon && b_is_weapon {
                    return a_cb
                        .attack
                        .partial_cmp(&b_cb.attack)
                        .unwrap_or(Ordering::Equal)
                        .then(
                            a_cb.defense
                                .partial_cmp(&b_cb.defense)
                                .unwrap_or(Ordering::Equal),
                        )
                        .reverse();
                } else if a_is_weapon {
                    return Ordering::Less;
                } else if b_is_weapon {
                    return Ordering::Greater;
                }

                // Armor
                return a_cb
                    .defense
                    .partial_cmp(&b_cb.defense)
                    .unwrap_or(Ordering::Equal)
                    .then(
                        a_cb.attack
                            .partial_cmp(&b_cb.attack)
                            .unwrap_or(Ordering::Equal),
                    )
                    .reverse();
            } else if a_cb.is_ok() {
                return Ordering::Less;
            } else if b_cb.is_ok() {
                return Ordering::Greater;
            }
        }

        // Fall back to name comparison.
        names.get(a).unwrap().0.cmp(&names.get(b).unwrap().0)
    };

    let mut inventories = world.borrow::<ViewMut<Inventory>>().unwrap();
    let mut holder_inv = (&mut inventories).get(holder).unwrap();

    holder_inv.items.sort_unstable_by(item_order);
}

pub fn use_item(world: &World, user_id: EntityId, item_id: EntityId, target: Option<(i32, i32)>) {
    {
        let map = world.borrow::<UniqueView<Map>>().unwrap();
        let mut msgs = world.borrow::<UniqueViewMut<Messages>>().unwrap();
        let entities = world.borrow::<EntitiesView>().unwrap();
        let aoes = world.borrow::<View<AreaOfEffect>>().unwrap();
        let mut asleeps = world.borrow::<ViewMut<Asleep>>().unwrap();
        let mut combat_stats = world.borrow::<ViewMut<CombatStats>>().unwrap();
        let coords = world.borrow::<View<Coord>>().unwrap();
        let mut hurt_bys = world.borrow::<ViewMut<HurtBy>>().unwrap();
        let inflicts_damages = world.borrow::<View<InflictsDamage>>().unwrap();
        let inflicts_sleeps = world.borrow::<View<InflictsSleep>>().unwrap();
        let monsters = world.borrow::<View<Monster>>().unwrap();
        let names = world.borrow::<View<Name>>().unwrap();
        let nutritions = world.borrow::<View<Nutrition>>().unwrap();
        let players = world.borrow::<View<Player>>().unwrap();
        let provides_healings = world.borrow::<View<ProvidesHealing>>().unwrap();
        let mut stomachs = world.borrow::<ViewMut<Stomach>>().unwrap();
        let mut tallies = world.borrow::<ViewMut<Tally>>().unwrap();

        let center = target.unwrap_or_else(|| coords.get(user_id).unwrap().0.into());
        let radius = aoes.get(item_id).map_or(0, |aoe| aoe.radius);
        let targets = ruggle::field_of_view(&*map, center, radius, FovShape::CirclePlus)
            .filter(|(_, _, symmetric)| *symmetric)
            .flat_map(|(x, y, _)| map.iter_entities_at(x, y))
            .filter(|id| monsters.contains(*id) || players.contains(*id));
        let user_name = &names.get(user_id).unwrap().0;
        let item_name = &names.get(item_id).unwrap().0;

        msgs.add(format!("{} uses {}.", user_name, item_name));

        for target_id in targets {
            let target_name = &names.get(target_id).unwrap().0;

            if let Ok(mut stomach) = (&mut stomachs).get(target_id) {
                if let Ok(nutrition) = nutritions.get(item_id) {
                    stomach.fullness = (stomach.fullness + nutrition.0).min(stomach.max_fullness);
                }
            }

            if let Ok(mut stats) = (&mut combat_stats).get(target_id) {
                if let Ok(ProvidesHealing { heal_amount }) = provides_healings.get(item_id) {
                    if stats.hp < stats.max_hp {
                        stats.hp = (stats.hp + heal_amount).min(stats.max_hp);
                        msgs.add(format!(
                            "{} heals {} for {} hp.",
                            item_name, target_name, heal_amount,
                        ));
                    } else {
                        let amount = 5;
                        stats.hp += amount;
                        stats.max_hp += amount;
                        msgs.add(format!(
                            "{} grants {} max hp to {}.",
                            item_name, amount, target_name,
                        ));
                    }
                }

                if let Ok(InflictsDamage { damage }) = inflicts_damages.get(item_id) {
                    stats.hp -= damage;
                    entities.add_component(target_id, &mut hurt_bys, HurtBy::Someone(user_id));
                    if let Ok(mut user_tally) = (&mut tallies).get(user_id) {
                        user_tally.damage_dealt += *damage.max(&0) as u64;
                    }
                    if let Ok(mut target_tally) = (&mut tallies).get(target_id) {
                        target_tally.damage_taken += *damage.max(&0) as u64;
                    }
                    msgs.add(format!(
                        "{} hits {} for {} hp.",
                        item_name, target_name, damage,
                    ));
                }

                if let Ok(InflictsSleep { sleepiness }) = inflicts_sleeps.get(item_id) {
                    entities.add_component(
                        target_id,
                        &mut asleeps,
                        Asleep {
                            sleepiness: *sleepiness,
                            last_hp: stats.hp,
                        },
                    );
                    msgs.add(format!("{} sends {} to sleep.", item_name, target_name));
                }
            }
        }
    }

    if world
        .borrow::<View<Consumable>>()
        .unwrap()
        .contains(item_id)
    {
        remove_item_from_inventory(world, user_id, item_id);
        world
            .borrow::<AllStoragesViewMut>()
            .unwrap()
            .delete_entity(item_id);
    }
}

pub fn is_asleep(world: &World, who: EntityId) -> bool {
    world.borrow::<View<Asleep>>().unwrap().contains(who)
}

pub fn handle_sleep_turn(world: &World, who: EntityId) {
    let mut asleeps = world.borrow::<ViewMut<Asleep>>().unwrap();

    if let Ok(mut asleep) = (&mut asleeps).get(who) {
        let (mut msgs, player_id, combat_stats, coords, fovs, names) = world
            .borrow::<(
                UniqueViewMut<Messages>,
                UniqueView<PlayerId>,
                View<CombatStats>,
                View<Coord>,
                View<FieldOfView>,
                View<Name>,
            )>()
            .unwrap();

        asleep.sleepiness -= 1;
        if (who == player_id.0 && world.run(player::player_sees_foes).unwrap())
            || player::can_see_player(world, who)
        {
            asleep.sleepiness -= 1;
        }
        if let Ok(stats) = combat_stats.get(who) {
            if stats.hp < asleep.last_hp {
                asleep.sleepiness -= 10;
                asleep.last_hp = stats.hp;
            }
        }

        if asleep.sleepiness <= 0 {
            let show_msg = if who == player_id.0 {
                true
            } else if let Ok(who_coord) = coords.get(who) {
                let player_fov = fovs.get(player_id.0).unwrap();
                player_fov.get(who_coord.0.into())
            } else {
                false
            };

            asleeps.remove(who);
            if show_msg {
                msgs.add(format!("{} wakes up.", names.get(who).unwrap().0));
            }
        }
    }
}
