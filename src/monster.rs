use shipyard::{
    EntitiesView, EntityId, Get, IntoIter, Shiperator, UniqueView, UniqueViewMut, View,
};
use std::{cmp::Reverse, collections::BinaryHeap};

use crate::components::{FieldOfView, Monster, Name, PlayerId, Position};

pub struct MonsterTurns(BinaryHeap<(Reverse<i32>, EntityId)>);

impl MonsterTurns {
    pub fn new() -> MonsterTurns {
        MonsterTurns(BinaryHeap::new())
    }
}

impl Default for MonsterTurns {
    fn default() -> Self {
        Self::new()
    }
}

pub fn monster_turns_empty(monster_turns: UniqueView<MonsterTurns>) -> bool {
    monster_turns.0.is_empty()
}

pub fn enqueue_monster_turns(
    mut monster_turns: UniqueViewMut<MonsterTurns>,
    player: UniqueView<PlayerId>,
    monsters: View<Monster>,
    positions: View<Position>,
) {
    let player_position = positions.get(player.0);

    for (id, (_, pos)) in (&monsters, &positions).iter().with_id() {
        // Monsters close to the player get their turns first.
        monster_turns
            .0
            .push((Reverse(pos.dist(&player_position)), id));
    }
}

fn do_turn_for_one_monster(
    monster: EntityId,
    player: &UniqueView<PlayerId>,
    fovs: &View<FieldOfView>,
    names: &View<Name>,
    positions: &View<Position>,
) {
    let fov = fovs.get(monster);
    let player_pos = positions.get(player.0);

    if fov.tiles.contains_key(&player_pos.into()) {
        println!("{} shouts insults", names.get(monster).0);
    }
}

pub fn do_monster_turns(
    entities: EntitiesView,
    mut monster_turns: UniqueViewMut<MonsterTurns>,
    player: UniqueView<PlayerId>,
    fovs: View<FieldOfView>,
    names: View<Name>,
    positions: View<Position>,
) {
    while let Some((_, monster)) = monster_turns.0.pop() {
        if entities.is_alive(monster) {
            do_turn_for_one_monster(monster, &player, &fovs, &names, &positions);
        }
    }
}
