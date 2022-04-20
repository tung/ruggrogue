use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro128PlusPlus as GameRng;
use shipyard::{
    AllStoragesViewMut, EntitiesView, EntityId, Get, IntoIter, Shiperator, UniqueView,
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
    saveload, spawn, GameSeed, TurnCount,
};

pub fn melee_attack(world: &World, attacker: EntityId, defender: EntityId) {
    let mut msgs = world.borrow::<UniqueViewMut<Messages>>();
    let entities = world.borrow::<EntitiesView>();
    let asleeps = world.borrow::<View<Asleep>>();
    let combat_bonuses = world.borrow::<View<CombatBonus>>();
    let mut combat_stats = world.borrow::<ViewMut<CombatStats>>();
    let equipments = world.borrow::<View<Equipment>>();
    let mut hurt_bys = world.borrow::<ViewMut<HurtBy>>();
    let names = world.borrow::<View<Name>>();
    let att_name = &names.get(attacker).0;
    let def_name = &names.get(defender).0;
    let mut rng = {
        let coords = world.borrow::<View<Coord>>();
        let mut hasher = WyHash::with_seed(magicnum::MELEE_ATTACK);
        hasher.write_u64(world.borrow::<UniqueView<GameSeed>>().0);
        hasher.write_u64(world.borrow::<UniqueView<TurnCount>>().0);
        if let Ok(attacker_coord) = coords.try_get(attacker) {
            hasher.write_i32(attacker_coord.0.x);
            hasher.write_i32(attacker_coord.0.y);
        }
        if let Ok(defender_coord) = coords.try_get(defender) {
            hasher.write_i32(defender_coord.0.x);
            hasher.write_i32(defender_coord.0.y);
        }
        GameRng::seed_from_u64(hasher.finish())
    };

    if !asleeps.contains(defender) && rng.gen_ratio(1, 10) {
        msgs.add(format!("{} misses {}.", att_name, def_name));
        return;
    }

    let attack_value = combat_stats.get(attacker).attack
        + equipments.try_get(attacker).map_or(0.0, |equip| {
            equip
                .weapon
                .iter()
                .chain(equip.armor.iter())
                .filter_map(|&e| combat_bonuses.try_get(e).ok())
                .map(|b| b.attack)
                .sum()
        });
    let defense_value = combat_stats.get(defender).defense
        + equipments.try_get(defender).map_or(0.0, |equip| {
            equip
                .weapon
                .iter()
                .chain(equip.armor.iter())
                .filter_map(|&e| combat_bonuses.try_get(e).ok())
                .map(|b| b.defense)
                .sum()
        });
    // Attack is twice defense most of the time.
    let mut damage = if attack_value >= defense_value * 2.0 {
        attack_value - defense_value
    } else {
        attack_value * (0.25 + (0.125 * attack_value / defense_value.max(1.0)).min(0.25))
    };

    // Fluctuate damage by a random amount.
    let mut suffix = '!';
    if rng.gen() {
        if rng.gen() {
            damage *= 1.5;
            suffix = '‼';
        } else {
            damage *= 0.5;
            suffix = '.';
        }
    }

    // Randomly round to nearest integer, e.g. 3.1 damage has a 10% chance to round to 4.
    let damage = damage.trunc() as i32
        + if rng.gen::<f32>() < damage.fract() {
            1
        } else {
            0
        };

    if damage > 0 {
        let mut tallies = world.borrow::<ViewMut<Tally>>();

        (&mut combat_stats).get(defender).hp -= damage;
        entities.add_component(&mut hurt_bys, HurtBy::Someone(attacker), defender);
        if let Ok(att_tally) = (&mut tallies).try_get(attacker) {
            att_tally.damage_dealt += damage.max(0) as u64;
        }
        if let Ok(def_tally) = (&mut tallies).try_get(defender) {
            def_tally.damage_taken += damage.max(0) as u64;
        }
        msgs.add(format!(
            "{} hits {} for {} hp{}",
            att_name, def_name, damage, suffix
        ));
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
        all_storages.run(|combat_stats: View<CombatStats>| {
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
        });

        for &entity in entities.iter().take(num_entities) {
            all_storages.run(|mut msgs: UniqueViewMut<Messages>, names: View<Name>| {
                msgs.add(format!("{} dies!", &names.get(entity).0));
            });

            all_storages.run(
                |mut exps: ViewMut<Experience>,
                 gives_exps: View<GivesExperience>,
                 hurt_bys: View<HurtBy>,
                 mut tallies: ViewMut<Tally>| {
                    if let Ok(&HurtBy::Someone(receiver)) = hurt_bys.try_get(entity) {
                        // Credit kill to whoever last hurt this entity.
                        if let Ok(receiver_tally) = (&mut tallies).try_get(receiver) {
                            receiver_tally.kills += 1;
                        }

                        // Give experience to whoever last hurt this entity.
                        if let Ok(receiver_exp) = (&mut exps).try_get(receiver) {
                            if let Ok(gives_exp) = gives_exps.try_get(entity) {
                                receiver_exp.exp += gives_exp.0;
                            }
                        }
                    }
                },
            );

            if entity == all_storages.borrow::<UniqueView<PlayerId>>().0 {
                // The player has died.
                all_storages.run(
                    |mut msgs: UniqueViewMut<Messages>,
                     mut player_alive: UniqueViewMut<PlayerAlive>| {
                        msgs.add("Press SPACE to continue...".into());
                        player_alive.0 = false;
                    },
                );

                saveload::delete_save_file();

                // Don't handle any more dead entities.
                num_entities = 0;
                break;
            } else {
                // Remove dead entity from the map.
                all_storages.run(
                    |mut map: UniqueViewMut<Map>,
                     blocks_tile: View<BlocksTile>,
                     coords: View<Coord>| {
                        map.remove_entity(
                            entity,
                            coords.get(entity).0.into(),
                            blocks_tile.contains(entity),
                        );
                    },
                );

                // Delete the dead entity.
                spawn::despawn_entity(&mut all_storages, entity);
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
