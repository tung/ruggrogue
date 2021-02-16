use shipyard::{
    EntitiesView, EntityId, Get, IntoIter, Shiperator, UniqueView, UniqueViewMut, View, ViewMut,
    World,
};
use std::{cmp::Reverse, collections::BinaryHeap};

use crate::{
    components::{BlocksTile, FieldOfView, Monster, Position},
    damage,
    map::Map,
    player::PlayerId,
};

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
    player_id: UniqueView<PlayerId>,
    monsters: View<Monster>,
    positions: View<Position>,
) {
    let player_position = positions.get(player_id.0);

    for (id, (_, pos)) in (&monsters, &positions).iter().with_id() {
        // Monsters close to the player get their turns first.
        monster_turns
            .0
            .push((Reverse(pos.dist(&player_position)), id));
    }
}

fn do_turn_for_one_monster(world: &World, monster: EntityId) {
    let (mut map, player_id, blocks, mut fovs, mut positions) = world.borrow::<(
        UniqueViewMut<Map>,
        UniqueView<PlayerId>,
        View<BlocksTile>,
        ViewMut<FieldOfView>,
        ViewMut<Position>,
    )>();

    let fov = (&mut fovs).get(monster);
    let player_pos: (i32, i32) = positions.get(player_id.0).into();

    if fov.get(player_pos) {
        let pos_mut = (&mut positions).get(monster);
        let pos: (i32, i32) = pos_mut.into();

        if let Some(step) = ruggle::find_path(&*map, pos, player_pos, 4, true).nth(1) {
            if step == player_pos {
                damage::melee_attack(world, monster, player_id.0);
            } else {
                map.move_entity(monster, pos, step, blocks.contains(monster));
                *pos_mut = step.into();
                fov.dirty = true;
            }
        }
    }
}

pub fn do_monster_turns(world: &World) {
    let (entities, mut monster_turns) =
        world.borrow::<(EntitiesView, UniqueViewMut<MonsterTurns>)>();

    while let Some((_, monster)) = monster_turns.0.pop() {
        if entities.is_alive(monster) {
            do_turn_for_one_monster(world, monster);
        }
    }
}
