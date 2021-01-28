use rand::Rng;
use shipyard::{EntitiesViewMut, EntityId, IntoIter, Shiperator, UniqueViewMut, ViewMut};

use crate::{
    components::{
        BlocksTile, CombatStats, FieldOfView, Monster, Name, Player, Position, Renderable,
    },
    map::Map,
    RuggleRng,
};

pub fn spawn_player(
    mut map: UniqueViewMut<Map>,
    mut entities: EntitiesViewMut,
    mut combat_stats: ViewMut<CombatStats>,
    mut fovs: ViewMut<FieldOfView>,
    mut names: ViewMut<Name>,
    mut players: ViewMut<Player>,
    mut positions: ViewMut<Position>,
    mut renderables: ViewMut<Renderable>,
) -> EntityId {
    let player_id = entities.add_entity(
        (
            &mut players,
            &mut combat_stats,
            &mut fovs,
            &mut names,
            &mut positions,
            &mut renderables,
        ),
        (
            Player {},
            CombatStats {
                max_hp: 30,
                hp: 30,
                defense: 2,
                power: 5,
            },
            FieldOfView::new(8),
            Name("Player".into()),
            Position { x: 0, y: 0 },
            Renderable {
                ch: '@',
                fg: [1., 1., 0., 1.],
                bg: [0., 0., 0., 1.],
            },
        ),
    );

    map.place_entity(player_id, (0, 0), false);

    player_id
}

pub fn spawn_monsters_in_rooms(
    mut map: UniqueViewMut<Map>,
    mut rng: UniqueViewMut<RuggleRng>,
    mut entities: EntitiesViewMut,
    mut blocks: ViewMut<BlocksTile>,
    mut combat_stats: ViewMut<CombatStats>,
    mut fovs: ViewMut<FieldOfView>,
    mut monsters: ViewMut<Monster>,
    mut names: ViewMut<Name>,
    mut positions: ViewMut<Position>,
    mut renderables: ViewMut<Renderable>,
) {
    for room in map.rooms.iter().skip(1) {
        let (ch, name, fg) = match rng.0.gen_range(0, 2) {
            0 => ('g', "Goblin", [0.5, 0.9, 0.2, 1.]),
            1 => ('o', "Orc", [0.9, 0.3, 0.2, 1.]),
            _ => ('X', "???", [1., 0., 0., 1.]),
        };

        entities.add_entity(
            (
                &mut monsters,
                &mut blocks,
                &mut combat_stats,
                &mut fovs,
                &mut names,
                &mut positions,
                &mut renderables,
            ),
            (
                Monster {},
                BlocksTile {},
                CombatStats {
                    max_hp: 16,
                    hp: 16,
                    defense: 1,
                    power: 4,
                },
                FieldOfView::new(8),
                Name(name.into()),
                room.center().into(),
                Renderable {
                    ch,
                    fg,
                    bg: [0., 0., 0., 1.],
                },
            ),
        );
    }

    for (id, (_, _, pos)) in (&monsters, &blocks, &positions).iter().with_id() {
        map.place_entity(id, pos.into(), true);
    }
}
