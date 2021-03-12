use shipyard::{
    EntitiesView, EntityId, Get, IntoIter, Shiperator, UniqueView, UniqueViewMut, View, ViewMut,
    World,
};
use std::{cmp::Reverse, collections::BinaryHeap};

use crate::{
    components::{BlocksTile, Coord, FieldOfView, Monster},
    damage, item,
    map::Map,
    player::{self, PlayerId},
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
    coords: View<Coord>,
    monsters: View<Monster>,
) {
    let player_coord = coords.get(player_id.0);

    for (id, (_, coord)) in (&monsters, &coords).iter().with_id() {
        // Monsters close to the player get their turns first.
        monster_turns
            .0
            .push((Reverse(coord.dist(&player_coord)), id));
    }
}

fn do_turn_for_one_monster(world: &World, monster: EntityId) {
    if item::is_asleep(world, monster) {
        item::handle_sleep_turn(world, monster);
    } else if player::can_see_player(world, monster) {
        let (mut map, player_id, blocks, mut coords, mut fovs) = world.borrow::<(
            UniqueViewMut<Map>,
            UniqueView<PlayerId>,
            View<BlocksTile>,
            ViewMut<Coord>,
            ViewMut<FieldOfView>,
        )>();
        let fov = (&mut fovs).get(monster);
        let player_pos: (i32, i32) = coords.get(player_id.0).0.into();
        let coord_mut = (&mut coords).get(monster);
        let pos: (i32, i32) = coord_mut.0.into();

        if let Some(step) = ruggle::find_path(&*map, pos, player_pos, 4, true).nth(1) {
            if step == player_pos {
                damage::melee_attack(world, monster, player_id.0);
            } else {
                map.move_entity(monster, pos, step, blocks.contains(monster));
                coord_mut.0 = step.into();
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
