use rand::{
    seq::{IteratorRandom, SliceRandom},
    Rng,
};
use shipyard::{EntitiesViewMut, EntityId, UniqueViewMut, ViewMut, World};

use crate::{
    components::{
        BlocksTile, CombatStats, Consumable, FieldOfView, InflictsDamage, Inventory, Item, Monster,
        Name, Player, Position, ProvidesHealing, Ranged, RenderOnFloor, RenderOnMap, Renderable,
    },
    map::Map,
    rect::Rect,
    ui, RuggleRng,
};

pub fn spawn_player(
    mut map: UniqueViewMut<Map>,
    mut entities: EntitiesViewMut,
    mut combat_stats: ViewMut<CombatStats>,
    mut fovs: ViewMut<FieldOfView>,
    mut inventories: ViewMut<Inventory>,
    mut names: ViewMut<Name>,
    mut players: ViewMut<Player>,
    mut positions: ViewMut<Position>,
    mut render_on_maps: ViewMut<RenderOnMap>,
    mut renderables: ViewMut<Renderable>,
) -> EntityId {
    let player_id = entities.add_entity(
        (
            &mut players,
            &mut combat_stats,
            &mut fovs,
            &mut inventories,
            &mut names,
            &mut positions,
            &mut render_on_maps,
            &mut renderables,
        ),
        (
            Player { auto_run: None },
            CombatStats {
                max_hp: 30,
                hp: 30,
                defense: 2,
                power: 5,
            },
            FieldOfView::new(8),
            Inventory { items: Vec::new() },
            Name("Player".into()),
            Position { x: 0, y: 0 },
            RenderOnMap {},
            Renderable {
                ch: '@',
                fg: ui::color::YELLOW,
                bg: ui::color::BLACK,
            },
        ),
    );

    map.place_entity(player_id, (0, 0), false);

    player_id
}

fn spawn_health_potion(world: &World, pos: (i32, i32)) {
    world.run(
        |mut map: UniqueViewMut<Map>,
         mut entities: EntitiesViewMut,
         mut consumables: ViewMut<Consumable>,
         mut items: ViewMut<Item>,
         mut names: ViewMut<Name>,
         mut positions: ViewMut<Position>,
         mut provides_healings: ViewMut<ProvidesHealing>,
         mut render_on_floors: ViewMut<RenderOnFloor>,
         mut renderables: ViewMut<Renderable>| {
            let item_id = entities.add_entity(
                (
                    &mut items,
                    &mut consumables,
                    &mut names,
                    &mut positions,
                    &mut provides_healings,
                    &mut render_on_floors,
                    &mut renderables,
                ),
                (
                    Item {},
                    Consumable {},
                    Name("Health Potion".into()),
                    pos.into(),
                    ProvidesHealing { heal_amount: 8 },
                    RenderOnFloor {},
                    Renderable {
                        ch: '!',
                        fg: ui::color::MAGENTA,
                        bg: ui::color::BLACK,
                    },
                ),
            );

            map.place_entity(item_id, pos, false);
        },
    );
}

fn spawn_magic_missile_scroll(world: &World, pos: (i32, i32)) {
    world.run(
        |mut map: UniqueViewMut<Map>,
         mut entities: EntitiesViewMut,
         mut consumables: ViewMut<Consumable>,
         mut inflicts_damages: ViewMut<InflictsDamage>,
         mut items: ViewMut<Item>,
         mut names: ViewMut<Name>,
         mut positions: ViewMut<Position>,
         mut rangeds: ViewMut<Ranged>,
         mut render_on_floors: ViewMut<RenderOnFloor>,
         mut renderables: ViewMut<Renderable>| {
            let item_id = entities.add_entity(
                (
                    &mut items,
                    &mut consumables,
                    &mut inflicts_damages,
                    &mut names,
                    &mut positions,
                    &mut rangeds,
                    &mut render_on_floors,
                    &mut renderables,
                ),
                (
                    Item {},
                    Consumable {},
                    InflictsDamage { damage: 8 },
                    Name("Magic Missile Scroll".into()),
                    pos.into(),
                    Ranged { range: 6 },
                    RenderOnFloor {},
                    Renderable {
                        ch: '?',
                        fg: ui::color::CYAN,
                        bg: ui::color::BLACK,
                    },
                ),
            );

            map.place_entity(item_id, pos, false);
        },
    );
}

fn spawn_monster(world: &World, pos: (i32, i32), ch: char, name: String, fg: &[f32; 4]) {
    world.run(
        |mut map: UniqueViewMut<Map>,
         mut entities: EntitiesViewMut,
         mut blocks: ViewMut<BlocksTile>,
         mut combat_stats: ViewMut<CombatStats>,
         mut fovs: ViewMut<FieldOfView>,
         mut monsters: ViewMut<Monster>,
         mut names: ViewMut<Name>,
         mut positions: ViewMut<Position>,
         mut render_on_maps: ViewMut<RenderOnMap>,
         mut renderables: ViewMut<Renderable>| {
            let monster_id = entities.add_entity(
                (
                    &mut monsters,
                    &mut blocks,
                    &mut combat_stats,
                    &mut fovs,
                    &mut names,
                    &mut positions,
                    &mut render_on_maps,
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
                    Name(name),
                    pos.into(),
                    RenderOnMap {},
                    Renderable {
                        ch,
                        fg: *fg,
                        bg: ui::color::BLACK,
                    },
                ),
            );

            map.place_entity(monster_id, pos, true);
        },
    );
}

fn spawn_random_monster_at(world: &World, pos: (i32, i32)) {
    let choice = world.run(|mut rng: UniqueViewMut<RuggleRng>| {
        [
            ('g', "Goblin", [0.5, 0.9, 0.2, 1.]),
            ('o', "Orc", [0.9, 0.3, 0.2, 1.]),
        ]
        .choose(&mut rng.0)
    });

    if let Some((ch, name, fg)) = choice {
        spawn_monster(world, pos, *ch, name.to_string(), fg);
    }
}

fn spawn_random_item_at(world: &World, pos: (i32, i32)) {
    match world.borrow::<UniqueViewMut<RuggleRng>>().0.gen_range(0, 2) {
        1 => spawn_magic_missile_scroll(world, pos),
        _ => spawn_health_potion(world, pos),
    }
}

fn fill_room_with_spawns(world: &World, room: &Rect) {
    if world.run(|mut rng: UniqueViewMut<RuggleRng>| rng.0.gen_ratio(1, 4)) {
        let positions = world.run(|mut rng: UniqueViewMut<RuggleRng>| {
            let num = rng.0.gen_range(1, 2);
            room.iter_xy().choose_multiple(&mut rng.0, num)
        });

        for pos in positions {
            spawn_random_item_at(world, pos);
        }
    }

    if world.run(|mut rng: UniqueViewMut<RuggleRng>| rng.0.gen_ratio(1, 2)) {
        let positions = world.run(|mut rng: UniqueViewMut<RuggleRng>| {
            let num = rng.0.gen_range(1, 4);
            room.iter_xy().choose_multiple(&mut rng.0, num)
        });

        for pos in positions {
            spawn_random_monster_at(world, pos);
        }
    }
}

pub fn fill_rooms_with_spawns(world: &World) {
    let rooms =
        world.run(|map: UniqueViewMut<Map>| map.rooms.iter().skip(1).copied().collect::<Vec<_>>());

    for room in rooms {
        fill_room_with_spawns(world, &room);
    }
}
