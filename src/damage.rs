use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;
use shipyard::{
    AllStoragesViewMut, EntitiesView, EntityId, Get, IntoIter, Shiperator, UniqueView,
    UniqueViewMut, View, ViewMut, World,
};
use std::hash::Hasher;
use wyhash::WyHash;

use crate::{
    components::{
        Asleep, BlocksTile, CombatBonus, CombatStats, Coord, Equipment, Experience,
        GivesExperience, HurtBy, Name,
    },
    magicnum,
    map::Map,
    message::Messages,
    player::{PlayerAlive, PlayerId},
    GameSeed, TurnCount,
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
        Pcg32::seed_from_u64(hasher.finish())
    };

    if !asleeps.contains(defender) && rng.gen_range(0, 10) == 0 {
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
    let damage = attack_value - defense_value;
    let damage = rng.gen_range(damage * 0.8, damage * 1.2);
    let damage = damage.trunc() as i32
        + if rng.gen_range(0, 100) < (damage.fract() * 100.0) as u32 {
            1
        } else {
            0
        };

    if damage > 0 {
        msgs.add(format!("{} hits {} for {} hp.", att_name, def_name, damage));
        (&mut combat_stats).get(defender).hp -= damage;
        entities.add_component(&mut hurt_bys, HurtBy::Someone(attacker), defender);
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

            // Give experience to whoever last hurt this entity, if applicable.
            all_storages.run(
                |mut exps: ViewMut<Experience>,
                 gives_exps: View<GivesExperience>,
                 hurt_bys: View<HurtBy>| {
                    if let Ok(&HurtBy::Someone(receiver)) = hurt_bys.try_get(entity) {
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

                // Don't handle any more dead entities.
                num_entities = 0;
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
                all_storages.delete(entity);
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
