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
    saveload, Wins,
};
use ruggrogue::FovShape;

pub struct PickUpHint(pub bool);

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

fn unequip_item(world: &World, unequipper_id: EntityId, item_id: EntityId) {
    let mut equipments = world.borrow::<ViewMut<Equipment>>();
    let equipment = (&mut equipments).get(unequipper_id);

    match world.borrow::<View<EquipSlot>>().get(item_id) {
        EquipSlot::Weapon => equipment.weapon = None,
        EquipSlot::Armor => equipment.armor = None,
    };
}

pub fn remove_equipment(world: &World, remover_id: EntityId, item_id: EntityId) {
    if world.borrow::<View<Inventory>>().contains(remover_id) {
        unequip_item(world, remover_id, item_id);
        add_item_to_inventory(world, remover_id, item_id);

        let mut msgs = world.borrow::<UniqueViewMut<Messages>>();
        let names = world.borrow::<View<Name>>();

        msgs.add(format!(
            "{} removes {}.",
            &names.get(remover_id).0,
            &names.get(item_id).0
        ));
    } else {
        // Remover has no inventory, so attempt dropping the equipment instead.
        drop_equipment(world, remover_id, item_id);
    }
}

pub fn drop_equipment(world: &World, dropper_id: EntityId, item_id: EntityId) {
    let dropper_pos: (i32, i32) = {
        let coords = world.borrow::<View<Coord>>();
        coords.get(dropper_id).0.into()
    };

    unequip_item(world, dropper_id, item_id);
    add_item_to_map(world, item_id, dropper_pos);

    let mut msgs = world.borrow::<UniqueViewMut<Messages>>();
    let names = world.borrow::<View<Name>>();

    msgs.add(format!(
        "{} drops {}.",
        &names.get(dropper_id).0,
        &names.get(item_id).0
    ));
}

pub fn equip_item(world: &World, equipper_id: EntityId, item_id: EntityId) {
    let mut equipments = world.borrow::<ViewMut<Equipment>>();
    let equipment = (&mut equipments).get(equipper_id);
    let equip_field = match world.borrow::<View<EquipSlot>>().get(item_id) {
        EquipSlot::Weapon => &mut equipment.weapon,
        EquipSlot::Armor => &mut equipment.armor,
    };

    if equip_field.is_some() {
        let mut inventories = world.borrow::<ViewMut<Inventory>>();
        let equipper_inv = (&mut inventories).get(equipper_id);
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

    let mut msgs = world.borrow::<UniqueViewMut<Messages>>();
    let names = world.borrow::<View<Name>>();

    msgs.add(format!(
        "{} equips {}.",
        &names.get(equipper_id).0,
        &names.get(item_id).0
    ));
}

pub fn sort_inventory(world: &World, holder: EntityId) {
    let aoes = world.borrow::<View<AreaOfEffect>>();
    let combat_bonuses = world.borrow::<View<CombatBonus>>();
    let inflicts_damages = world.borrow::<View<InflictsDamage>>();
    let inflicts_sleeps = world.borrow::<View<InflictsSleep>>();
    let names = world.borrow::<View<Name>>();
    let provides_healings = world.borrow::<View<ProvidesHealing>>();
    let nutritions = world.borrow::<View<Nutrition>>();
    let rangeds = world.borrow::<View<Ranged>>();
    let victories = world.borrow::<View<Victory>>();
    let item_order = |&a: &EntityId, &b: &EntityId| -> Ordering {
        // Present
        {
            let a_is_victory = victories.contains(a);
            let b_is_victory = victories.contains(b);

            if a_is_victory && b_is_victory {
                return Ordering::Equal;
            } else if a_is_victory {
                return Ordering::Less;
            } else if b_is_victory {
                return Ordering::Greater;
            }
        }

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
            let a_cb = combat_bonuses.try_get(a);
            let b_cb = combat_bonuses.try_get(b);

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
        names.get(a).0.cmp(&names.get(b).0)
    };

    let mut inventories = world.borrow::<ViewMut<Inventory>>();
    let holder_inv = (&mut inventories).get(holder);

    holder_inv.items.sort_unstable_by(item_order);
}

/// Returns true if the game should end after the item is used.
pub fn use_item(
    world: &World,
    user_id: EntityId,
    item_id: EntityId,
    target: Option<(i32, i32)>,
) -> bool {
    if world.borrow::<View<Player>>().contains(user_id)
        && world.borrow::<View<Victory>>().contains(item_id)
    {
        // Auto-save the game before the victory item is deleted in case an AppQuit causes the game
        // to terminate outside of standard gameplay.
        if let Err(e) = saveload::save_game(world) {
            eprintln!("Warning: saveload::save_game: {}", e);
        }
        remove_item_from_inventory(world, user_id, item_id);
        world.borrow::<AllStoragesViewMut>().delete(item_id);
        world.borrow::<UniqueViewMut<Wins>>().0 += 1;
        return true;
    } else {
        let map = world.borrow::<UniqueView<Map>>();
        let mut msgs = world.borrow::<UniqueViewMut<Messages>>();
        let entities = world.borrow::<EntitiesView>();
        let aoes = world.borrow::<View<AreaOfEffect>>();
        let mut asleeps = world.borrow::<ViewMut<Asleep>>();
        let mut combat_stats = world.borrow::<ViewMut<CombatStats>>();
        let coords = world.borrow::<View<Coord>>();
        let mut hurt_bys = world.borrow::<ViewMut<HurtBy>>();
        let inflicts_damages = world.borrow::<View<InflictsDamage>>();
        let inflicts_sleeps = world.borrow::<View<InflictsSleep>>();
        let monsters = world.borrow::<View<Monster>>();
        let names = world.borrow::<View<Name>>();
        let nutritions = world.borrow::<View<Nutrition>>();
        let players = world.borrow::<View<Player>>();
        let provides_healings = world.borrow::<View<ProvidesHealing>>();
        let mut stomachs = world.borrow::<ViewMut<Stomach>>();
        let mut tallies = world.borrow::<ViewMut<Tally>>();

        let center = target.unwrap_or_else(|| coords.get(user_id).0.into());
        let radius = aoes.try_get(item_id).map_or(0, |aoe| aoe.radius);
        let targets = ruggrogue::field_of_view(&*map, center, radius, FovShape::CirclePlus)
            .filter(|(_, _, symmetric)| *symmetric)
            .flat_map(|(x, y, _)| map.iter_entities_at(x, y))
            .filter(|id| monsters.contains(*id) || players.contains(*id));
        let user_name = &names.get(user_id).0;
        let item_name = &names.get(item_id).0;

        msgs.add(format!("{} uses {}.", user_name, item_name));

        for target_id in targets {
            let target_name = &names.get(target_id).0;

            if let Ok(stomach) = (&mut stomachs).try_get(target_id) {
                if let Ok(nutrition) = nutritions.try_get(item_id) {
                    stomach.fullness = (stomach.fullness + nutrition.0).min(stomach.max_fullness);
                }
            }

            if let Ok(stats) = (&mut combat_stats).try_get(target_id) {
                if let Ok(ProvidesHealing { heal_amount }) = provides_healings.try_get(item_id) {
                    if stats.hp < stats.max_hp {
                        stats.hp = (stats.hp + heal_amount).min(stats.max_hp);
                        msgs.add(format!(
                            "{} heals {} for {} hp.",
                            item_name, target_name, heal_amount,
                        ));
                    } else {
                        let amount = 2;
                        stats.hp += amount;
                        stats.max_hp += amount;
                        msgs.add(format!(
                            "{} grants {} max hp to {}.",
                            item_name, amount, target_name,
                        ));
                    }
                }

                if let Ok(InflictsDamage { damage }) = inflicts_damages.try_get(item_id) {
                    stats.hp -= damage;
                    entities.add_component(&mut hurt_bys, HurtBy::Someone(user_id), target_id);
                    if let Ok(user_tally) = (&mut tallies).try_get(user_id) {
                        user_tally.damage_dealt += *damage.max(&0) as u64;
                    }
                    if let Ok(target_tally) = (&mut tallies).try_get(target_id) {
                        target_tally.damage_taken += *damage.max(&0) as u64;
                    }
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
    }

    if world.borrow::<View<Consumable>>().contains(item_id) {
        remove_item_from_inventory(world, user_id, item_id);
        world.borrow::<AllStoragesViewMut>().delete(item_id);
    }

    false
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
