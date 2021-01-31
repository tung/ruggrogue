use shipyard::{AllStoragesViewMut, EntityId, Get, UniqueViewMut, View, ViewMut};
use std::collections::VecDeque;

use crate::{
    components::{BlocksTile, CombatStats, Name, Player, Position},
    map::Map,
    message::Messages,
    player::PlayerAlive,
};

pub struct MeleeEvent {
    pub attacker: EntityId,
    pub defender: EntityId,
}

pub struct MeleeQueue(VecDeque<MeleeEvent>);

impl MeleeQueue {
    pub fn new() -> Self {
        Self(VecDeque::new())
    }

    pub fn push_back(&mut self, attacker: EntityId, defender: EntityId) {
        self.0.push_back(MeleeEvent { attacker, defender });
    }

    pub fn pop_front(&mut self) -> Option<MeleeEvent> {
        self.0.pop_front()
    }
}

impl Default for MeleeQueue {
    fn default() -> Self {
        Self::new()
    }
}

pub struct DamageEvent {
    target: EntityId,
    amount: i32,
}

pub struct DamageQueue(VecDeque<DamageEvent>);

impl DamageQueue {
    pub fn new() -> Self {
        Self(VecDeque::new())
    }

    pub fn push_back(&mut self, target: EntityId, amount: i32) {
        self.0.push_back(DamageEvent { target, amount });
    }

    pub fn pop_front(&mut self) -> Option<DamageEvent> {
        self.0.pop_front()
    }
}

impl Default for DamageQueue {
    fn default() -> Self {
        Self::new()
    }
}

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

/// Convert MeleeEvents into DamageEvents and log hit messages.
pub fn melee_combat(
    mut damage_queue: UniqueViewMut<DamageQueue>,
    mut melee_queue: UniqueViewMut<MeleeQueue>,
    mut messages: UniqueViewMut<Messages>,
    combat_stats: View<CombatStats>,
    names: View<Name>,
) {
    while let Some(MeleeEvent { attacker, defender }) = melee_queue.pop_front() {
        let damage = combat_stats.get(attacker).power - combat_stats.get(defender).defense;
        let att_name = &names.get(attacker).0;
        let def_name = &names.get(defender).0;

        if damage > 0 {
            messages.add(format!("{} hits {} for {} hp.", att_name, def_name, damage));
            damage_queue.push_back(defender, damage);
        } else {
            messages.add(format!("{} fails to hurt {}.", att_name, def_name));
        }
    }
}

/// Convert DamageEvents into hp changes to CombatStats and add any entities that die to the
/// DeadEntities queue, logging them as they die.
pub fn inflict_damage(
    mut damage_queue: UniqueViewMut<DamageQueue>,
    mut dead_entities: UniqueViewMut<DeadEntities>,
    mut messages: UniqueViewMut<Messages>,
    mut combat_stats: ViewMut<CombatStats>,
    names: View<Name>,
) {
    while let Some(DamageEvent { target, amount }) = damage_queue.pop_front() {
        let target_stats = (&mut combat_stats).get(target);

        if target_stats.hp > 0 && amount >= target_stats.hp {
            messages.add(format!("{} dies!", names.get(target).0));
            dead_entities.push_back(target);
        }

        target_stats.hp -= amount;
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
