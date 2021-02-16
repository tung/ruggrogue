use shipyard::{
    AllStoragesViewMut, EntityId, Get, IntoIter, Shiperator, UniqueViewMut, View, ViewMut, World,
};
use std::collections::VecDeque;

use crate::{
    components::{BlocksTile, CombatStats, Name, Player, Position},
    map::Map,
    message::Messages,
    player::PlayerAlive,
};

pub struct DeadEntities(VecDeque<EntityId>);

impl DeadEntities {
    pub fn new() -> Self {
        Self(VecDeque::new())
    }

    pub fn push_back(&mut self, e: EntityId) {
        self.0.push_back(e);
    }

    pub fn pop_front(&mut self) -> Option<EntityId> {
        self.0.pop_front()
    }
}

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

pub fn check_for_dead(
    mut dead_entities: UniqueViewMut<DeadEntities>,
    mut msgs: UniqueViewMut<Messages>,
    combat_stats: View<CombatStats>,
    names: View<Name>,
) {
    for (id, stats) in combat_stats.iter().with_id() {
        if stats.hp <= 0 {
            msgs.add(format!("{} dies!", names.get(id).0));
            dead_entities.push_back(id);
        }
    }
}

fn pop_dead_entity(mut dead_entities: UniqueViewMut<DeadEntities>) -> Option<EntityId> {
    dead_entities.pop_front()
}

/// Delete entities in the DeadEntities queue, clearing them from the map in the process.
pub fn delete_dead_entities(mut all_storages: AllStoragesViewMut) {
    while let Some(dead_entity) = all_storages.run(pop_dead_entity) {
        if all_storages.run(|players: View<Player>| players.contains(dead_entity)) {
            all_storages.run(
                |mut messages: UniqueViewMut<Messages>,
                 mut player_alive: UniqueViewMut<PlayerAlive>| {
                    messages.add("Press SPACE to continue...".into());
                    player_alive.0 = false;
                },
            );
        } else {
            all_storages.run(
                |mut map: UniqueViewMut<Map>,
                 blocks_tile: View<BlocksTile>,
                 positions: View<Position>| {
                    map.remove_entity(
                        dead_entity,
                        positions.get(dead_entity).into(),
                        blocks_tile.contains(dead_entity),
                    );
                },
            );
            all_storages.delete(dead_entity);
        }
    }
}
