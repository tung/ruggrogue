use shipyard::{
    AllStoragesViewMut, EntityId, Get, IntoIter, Shiperator, UniqueView, UniqueViewMut, View,
    ViewMut, World,
};

use crate::{
    components::{BlocksTile, CombatStats, Coord, Name},
    map::Map,
    message::Messages,
    player::{PlayerAlive, PlayerId},
};

pub fn melee_attack(world: &World, attacker: EntityId, defender: EntityId) {
    let (mut msgs, mut combat_stats, names) =
        world.borrow::<(UniqueViewMut<Messages>, ViewMut<CombatStats>, View<Name>)>();
    let damage = combat_stats.get(attacker).power - combat_stats.get(defender).defense;
    let att_name = &names.get(attacker).0;
    let def_name = &names.get(defender).0;

    if damage > 0 {
        msgs.add(format!("{} hits {} for {} hp.", att_name, def_name, damage));
        (&mut combat_stats).get(defender).hp -= damage;
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
