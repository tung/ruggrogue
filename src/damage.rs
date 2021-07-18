use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;
use shipyard::{
    AllStoragesViewMut, EntitiesView, EntityId, Get, IntoIter, IntoWithId, UniqueView,
    UniqueViewMut, View, ViewMut, World,
};
use std::hash::Hasher;
use wyhash::WyHash;

use crate::{
    components::{
        Asleep, BlocksTile, CombatBonus, CombatStats, Coord, Equipment, Experience,
        GivesExperience, HurtBy, Name, Tally,
    },
    magicnum,
    map::Map,
    message::Messages,
    player::{PlayerAlive, PlayerId},
    GameSeed, TurnCount,
};

pub fn melee_attack(world: &World, attacker: EntityId, defender: EntityId) {
    let mut msgs = world.borrow::<UniqueViewMut<Messages>>().unwrap();
    let entities = world.borrow::<EntitiesView>().unwrap();
    let asleeps = world.borrow::<View<Asleep>>().unwrap();
    let combat_bonuses = world.borrow::<View<CombatBonus>>().unwrap();
    let mut combat_stats = world.borrow::<ViewMut<CombatStats>>().unwrap();
    let equipments = world.borrow::<View<Equipment>>().unwrap();
    let mut hurt_bys = world.borrow::<ViewMut<HurtBy>>().unwrap();
    let names = world.borrow::<View<Name>>().unwrap();
    let att_name = &names.get(attacker).unwrap().0;
    let def_name = &names.get(defender).unwrap().0;
    let mut rng = {
        let coords = world.borrow::<View<Coord>>().unwrap();
        let mut hasher = WyHash::with_seed(magicnum::MELEE_ATTACK);
        hasher.write_u64(world.borrow::<UniqueView<GameSeed>>().unwrap().0);
        hasher.write_u64(world.borrow::<UniqueView<TurnCount>>().unwrap().0);
        if let Ok(attacker_coord) = coords.get(attacker) {
            hasher.write_i32(attacker_coord.0.x);
            hasher.write_i32(attacker_coord.0.y);
        }
        if let Ok(defender_coord) = coords.get(defender) {
            hasher.write_i32(defender_coord.0.x);
            hasher.write_i32(defender_coord.0.y);
        }
        Pcg32::seed_from_u64(hasher.finish())
    };

    if !asleeps.contains(defender) && rng.gen_range(0, 10) == 0 {
        msgs.add(format!("{} misses {}.", att_name, def_name));
        return;
    }

    let attack_value = combat_stats.get(attacker).unwrap().attack
        + equipments.get(attacker).map_or(0.0, |equip| {
            equip
                .weapon
                .iter()
                .chain(equip.armor.iter())
                .filter_map(|&e| combat_bonuses.get(e).ok())
                .map(|b| b.attack)
                .sum()
        });
    let defense_value = combat_stats.get(defender).unwrap().defense
        + equipments.get(defender).map_or(0.0, |equip| {
            equip
                .weapon
                .iter()
                .chain(equip.armor.iter())
                .filter_map(|&e| combat_bonuses.get(e).ok())
                .map(|b| b.defense)
                .sum()
        });
    let mut damage = attack_value - defense_value;

    // Average incoming damage should equal defense; buff it if it falls short.
    if damage < defense_value && defense_value.abs() > f32::EPSILON {
        // Ease damage towards zero using tanh, so e.g. what would have been zero damage before is
        // now ~24% of defense_value worth of damage instead.
        damage = defense_value * (1.0 + ((damage - defense_value) / defense_value).tanh());
    }

    // Fluctuate damage by a random amount.
    damage = rng.gen_range(damage * 0.8, damage * 1.2);

    // Randomly round to nearest integer, e.g. 3.1 damage has a 10% chance to round to 4.
    let damage = damage.trunc() as i32
        + if rng.gen_range(0, 100) < (damage.fract() * 100.0) as u32 {
            1
        } else {
            0
        };

    if damage > 0 {
        let mut tallies = world.borrow::<ViewMut<Tally>>().unwrap();

        (&mut combat_stats).get(defender).unwrap().hp -= damage;
        entities.add_component(defender, &mut hurt_bys, HurtBy::Someone(attacker));
        if let Ok(mut att_tally) = (&mut tallies).get(attacker) {
            att_tally.damage_dealt += damage.max(0) as u64;
        }
        if let Ok(mut def_tally) = (&mut tallies).get(defender) {
            def_tally.damage_taken += damage.max(0) as u64;
        }
        msgs.add(format!("{} hits {} for {} hp.", att_name, def_name, damage));
    } else {
        msgs.add(format!(
            "{} hits {}, but does no damage.",
            att_name, def_name
        ));
    }
}

/// Check for dead entities, do any special handling for them and delete them.
pub fn handle_dead_entities(mut all_storages: AllStoragesViewMut) {
    loop {
        let mut entities = [EntityId::dead(); 10];
        let mut num_entities = 0;

        // Fill buffer with dead entities.
        all_storages
            .run(|combat_stats: View<CombatStats>| {
                for ((id, _), entity) in combat_stats
                    .iter()
                    .with_id()
                    .into_iter()
                    .filter(|(_, stats)| stats.hp <= 0)
                    .zip(entities.iter_mut())
                {
                    *entity = id;
                    num_entities += 1;
                }
            })
            .unwrap();

        for &entity in entities.iter().take(num_entities) {
            all_storages
                .run(|mut msgs: UniqueViewMut<Messages>, names: View<Name>| {
                    msgs.add(format!("{} dies!", &names.get(entity).unwrap().0));
                })
                .unwrap();

            all_storages
                .run(
                    |mut exps: ViewMut<Experience>,
                     gives_exps: View<GivesExperience>,
                     hurt_bys: View<HurtBy>,
                     mut tallies: ViewMut<Tally>| {
                        if let Ok(&HurtBy::Someone(receiver)) = hurt_bys.get(entity) {
                            // Credit kill to whoever last hurt this entity.
                            if let Ok(mut receiver_tally) = (&mut tallies).get(receiver) {
                                receiver_tally.kills += 1;
                            }

                            // Give experience to whoever last hurt this entity.
                            if let Ok(mut receiver_exp) = (&mut exps).get(receiver) {
                                if let Ok(gives_exp) = gives_exps.get(entity) {
                                    receiver_exp.exp += gives_exp.0;
                                }
                            }
                        }
                    },
                )
                .unwrap();

            if entity == all_storages.borrow::<UniqueView<PlayerId>>().unwrap().0 {
                // The player has died.
                all_storages
                    .run(
                        |mut msgs: UniqueViewMut<Messages>,
                         mut player_alive: UniqueViewMut<PlayerAlive>| {
                            msgs.add("Press SPACE to continue...".into());
                            player_alive.0 = false;
                        },
                    )
                    .unwrap();

                // Don't handle any more dead entities.
                num_entities = 0;
                break;
            } else {
                // Remove dead entity from the map.
                all_storages
                    .run(
                        |mut map: UniqueViewMut<Map>,
                         blocks_tile: View<BlocksTile>,
                         coords: View<Coord>| {
                            map.remove_entity(
                                entity,
                                coords.get(entity).unwrap().0.into(),
                                blocks_tile.contains(entity),
                            );
                        },
                    )
                    .unwrap();

                // Delete the dead entity.
                all_storages.delete_entity(entity);
            }
        }

        if num_entities == 0 {
            break;
        }
    }
}

/// Clear all HurtBy components off of all entities.
pub fn clear_hurt_bys(mut hurt_bys: ViewMut<HurtBy>) {
    hurt_bys.clear();
}
