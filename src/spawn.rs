use rand::{
    seq::{IteratorRandom, SliceRandom},
    Rng,
};
use shipyard::{
    AllStoragesViewMut, EntitiesView, EntitiesViewMut, EntityId, IntoIter, Shiperator, UniqueView,
    UniqueViewMut, View, ViewMut, World,
};
use std::collections::HashSet;

use crate::{
    components::{
        AreaOfEffect, BlocksTile, CombatStats, Consumable, Coord, FieldOfView, InflictsDamage,
        InflictsSleep, Inventory, Item, Monster, Name, Nutrition, Player, ProvidesHealing, Ranged,
        RenderOnFloor, RenderOnMap, Renderable, Stomach,
    },
    gamesym::GameSym,
    map::{Map, Rect},
    player, RuggleRng,
};
use ruggle::util::Color;

/// Spawn a player.
///
/// NOTE: The player must be positioned on the map at some point after this.
pub fn spawn_player(
    mut entities: EntitiesViewMut,
    mut combat_stats: ViewMut<CombatStats>,
    mut coords: ViewMut<Coord>,
    mut fovs: ViewMut<FieldOfView>,
    mut inventories: ViewMut<Inventory>,
    mut names: ViewMut<Name>,
    mut players: ViewMut<Player>,
    mut render_on_maps: ViewMut<RenderOnMap>,
    mut renderables: ViewMut<Renderable>,
    mut stomachs: ViewMut<Stomach>,
) -> EntityId {
    entities.add_entity(
        (
            &mut players,
            &mut combat_stats,
            &mut coords,
            &mut fovs,
            &mut inventories,
            &mut names,
            &mut render_on_maps,
            &mut renderables,
            &mut stomachs,
        ),
        (
            Player { auto_run: None },
            CombatStats {
                max_hp: 30,
                hp: 30,
                defense: 2,
                power: 5,
            },
            Coord((0, 0).into()),
            FieldOfView::new(8),
            Inventory { items: Vec::new() },
            Name("Player".into()),
            RenderOnMap {},
            Renderable {
                sym: GameSym::Player,
                fg: Color::YELLOW,
                bg: Color::BLACK,
            },
            Stomach {
                fullness: 1000,
                max_fullness: 1000,
                sub_hp: 0,
            },
        ),
    )
}

fn spawn_item(world: &World, pos: (i32, i32), name: &str, sym: GameSym, fg: Color) -> EntityId {
    world.run(
        |mut map: UniqueViewMut<Map>,
         mut entities: EntitiesViewMut,
         mut coords: ViewMut<Coord>,
         mut items: ViewMut<Item>,
         mut names: ViewMut<Name>,
         mut render_on_floors: ViewMut<RenderOnFloor>,
         mut renderables: ViewMut<Renderable>| {
            let item_id = entities.add_entity(
                (
                    &mut items,
                    &mut coords,
                    &mut names,
                    &mut render_on_floors,
                    &mut renderables,
                ),
                (
                    Item {},
                    Coord(pos.into()),
                    Name(name.into()),
                    RenderOnFloor {},
                    Renderable {
                        sym,
                        fg,
                        bg: Color::BLACK,
                    },
                ),
            );

            map.place_entity(item_id, pos, false);

            item_id
        },
    )
}

fn spawn_ration(world: &World, pos: (i32, i32)) {
    let item_id = spawn_item(world, pos, "Ration", GameSym::Ration, Color::BROWN);
    let (entities, mut consumables, mut nutritions) =
        world.borrow::<(EntitiesView, ViewMut<Consumable>, ViewMut<Nutrition>)>();

    entities.add_component(
        (&mut consumables, &mut nutritions),
        (Consumable {}, Nutrition(400)),
        item_id,
    );
}

fn spawn_health_potion(world: &World, pos: (i32, i32)) {
    let item_id = spawn_item(
        world,
        pos,
        "Health Potion",
        GameSym::HealthPotion,
        Color::MAGENTA,
    );
    let (entities, mut consumables, mut provides_healings) =
        world.borrow::<(EntitiesView, ViewMut<Consumable>, ViewMut<ProvidesHealing>)>();

    entities.add_component(
        (&mut consumables, &mut provides_healings),
        (Consumable {}, ProvidesHealing { heal_amount: 8 }),
        item_id,
    );
}

fn spawn_magic_missile_scroll(world: &World, pos: (i32, i32)) {
    let item_id = spawn_item(
        world,
        pos,
        "Magic Missile Scroll",
        GameSym::MagicMissileScroll,
        Color::CYAN,
    );
    let (entities, mut consumables, mut inflicts_damages, mut rangeds) = world.borrow::<(
        EntitiesView,
        ViewMut<Consumable>,
        ViewMut<InflictsDamage>,
        ViewMut<Ranged>,
    )>();

    entities.add_component(
        (&mut consumables, &mut inflicts_damages, &mut rangeds),
        (
            Consumable {},
            InflictsDamage { damage: 8 },
            Ranged { range: 6 },
        ),
        item_id,
    );
}

fn spawn_fireball_scroll(world: &World, pos: (i32, i32)) {
    let item_id = spawn_item(
        world,
        pos,
        "Fireball Scroll",
        GameSym::FireballScroll,
        Color::ORANGE,
    );
    let (entities, mut aoes, mut consumables, mut inflicts_damages, mut rangeds) = world.borrow::<(
        EntitiesView,
        ViewMut<AreaOfEffect>,
        ViewMut<Consumable>,
        ViewMut<InflictsDamage>,
        ViewMut<Ranged>,
    )>();

    entities.add_component(
        (
            &mut aoes,
            &mut consumables,
            &mut inflicts_damages,
            &mut rangeds,
        ),
        (
            AreaOfEffect { radius: 3 },
            Consumable {},
            InflictsDamage { damage: 20 },
            Ranged { range: 6 },
        ),
        item_id,
    );
}

fn spawn_sleep_scroll(world: &World, pos: (i32, i32)) {
    let item_id = spawn_item(
        world,
        pos,
        "Sleep Scroll",
        GameSym::SleepScroll,
        Color::PINK,
    );
    let (entities, mut aoes, mut consumables, mut inflicts_sleeps, mut rangeds) = world.borrow::<(
        EntitiesView,
        ViewMut<AreaOfEffect>,
        ViewMut<Consumable>,
        ViewMut<InflictsSleep>,
        ViewMut<Ranged>,
    )>();

    entities.add_component(
        (
            &mut aoes,
            &mut consumables,
            &mut inflicts_sleeps,
            &mut rangeds,
        ),
        (
            AreaOfEffect { radius: 1 },
            Consumable {},
            InflictsSleep { sleepiness: 36 },
            Ranged { range: 6 },
        ),
        item_id,
    );
}

fn spawn_monster(world: &World, pos: (i32, i32), sym: GameSym, name: &str, fg: Color) {
    world.run(
        |mut map: UniqueViewMut<Map>,
         mut entities: EntitiesViewMut,
         mut blocks: ViewMut<BlocksTile>,
         mut combat_stats: ViewMut<CombatStats>,
         mut coords: ViewMut<Coord>,
         mut fovs: ViewMut<FieldOfView>,
         mut monsters: ViewMut<Monster>,
         mut names: ViewMut<Name>,
         mut render_on_maps: ViewMut<RenderOnMap>,
         mut renderables: ViewMut<Renderable>| {
            let monster_id = entities.add_entity(
                (
                    &mut monsters,
                    &mut blocks,
                    &mut combat_stats,
                    &mut coords,
                    &mut fovs,
                    &mut names,
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
                    Coord(pos.into()),
                    FieldOfView::new(8),
                    Name(name.into()),
                    RenderOnMap {},
                    Renderable {
                        sym,
                        fg,
                        bg: Color::BLACK,
                    },
                ),
            );

            map.place_entity(monster_id, pos, true);
        },
    );
}

fn spawn_random_monster_at<R: Rng>(world: &World, rng: &mut R, pos: (i32, i32)) {
    let choice = [
        (
            GameSym::Goblin,
            "Goblin",
            Color {
                r: 128,
                g: 230,
                b: 51,
            },
        ),
        (
            GameSym::Orc,
            "Orc",
            Color {
                r: 230,
                g: 77,
                b: 51,
            },
        ),
    ]
    .choose(rng);

    if let Some((sym, name, fg)) = choice {
        spawn_monster(world, pos, *sym, name, *fg);
    }
}

fn spawn_random_item_at<R: Rng>(world: &World, rng: &mut R, pos: (i32, i32)) {
    let choice = [
        spawn_health_potion,
        spawn_magic_missile_scroll,
        spawn_fireball_scroll,
        spawn_sleep_scroll,
    ]
    .choose(rng);

    if let Some(item_fn) = choice {
        item_fn(world, pos);
    }
}

fn fill_room_with_spawns(world: &World, room: &Rect) {
    let mut rng = world.borrow::<UniqueViewMut<RuggleRng>>();
    let depth = world.borrow::<UniqueView<Map>>().depth;
    let depth = if depth < 1 { 1usize } else { depth as usize };

    if rng.0.gen_ratio(1, 4) {
        let num = rng.0.gen_range(1, 2);

        for pos in room.iter_xy().choose_multiple(&mut rng.0, num) {
            spawn_random_item_at(world, &mut rng.0, pos);
        }
    }

    if rng.0.gen_ratio(1, 2) {
        let num = rng.0.gen_range(1, 1 + ((depth + 1) / 2).min(3));

        for pos in room.iter_xy().choose_multiple(&mut rng.0, num) {
            spawn_random_monster_at(world, &mut rng.0, pos);
        }
    }
}

pub fn fill_rooms_with_spawns(world: &World) {
    let rooms =
        world.run(|map: UniqueViewMut<Map>| map.rooms.iter().skip(1).copied().collect::<Vec<_>>());

    for room in &rooms {
        fill_room_with_spawns(world, room);
    }

    let ration_pos = {
        let (map, mut rng, items) =
            world.borrow::<(UniqueView<Map>, UniqueViewMut<RuggleRng>, View<Item>)>();
        rooms.choose(&mut rng.0).and_then(|ration_room| {
            ration_room
                .iter_xy()
                .filter(|&(x, y)| !map.iter_entities_at(x, y).any(|id| items.contains(id)))
                .choose(&mut rng.0)
        })
    };

    if let Some(ration_pos) = ration_pos {
        spawn_ration(world, ration_pos);
    }
}

fn extend_despawn<T>(
    world: &World,
    despawn_ids: &mut Vec<EntityId>,
    preserve_ids: &HashSet<EntityId>,
) where
    T: 'static + Send + Sync,
{
    world.run(|storage: View<T>| {
        despawn_ids.extend(
            storage
                .iter()
                .with_id()
                .map(|(id, _)| id)
                .filter(|id| !preserve_ids.contains(id)),
        );
    });
}

pub fn despawn_all_but_player(world: &World) {
    let preserve_ids = world.run(player::all_player_associated_ids);
    let mut despawn_ids = Vec::new();

    // I really wish Shipyard had some way to iterate over all entity IDs...
    extend_despawn::<Item>(world, &mut despawn_ids, &preserve_ids);
    extend_despawn::<Monster>(world, &mut despawn_ids, &preserve_ids);

    world.run(|mut all_storages: AllStoragesViewMut| {
        for id in despawn_ids {
            all_storages.delete(id);
        }
    });
}
