use rand::{
    seq::{IteratorRandom, SliceRandom},
    Rng,
};
use shipyard::{
    AllStoragesViewMut, EntitiesViewMut, EntityId, IntoIter, Shiperator, UniqueView, UniqueViewMut,
    View, ViewMut, World,
};
use std::collections::HashSet;

use crate::{
    components::{
        AreaOfEffect, BlocksTile, CombatStats, Consumable, Coord, FieldOfView, InflictsDamage,
        InflictsSleep, Inventory, Item, Monster, Name, Player, ProvidesHealing, Ranged,
        RenderOnFloor, RenderOnMap, Renderable,
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
        ),
    )
}

fn spawn_health_potion(world: &World, pos: (i32, i32)) {
    world.run(
        |mut map: UniqueViewMut<Map>,
         mut entities: EntitiesViewMut,
         mut consumables: ViewMut<Consumable>,
         mut coords: ViewMut<Coord>,
         mut items: ViewMut<Item>,
         mut names: ViewMut<Name>,
         mut provides_healings: ViewMut<ProvidesHealing>,
         mut render_on_floors: ViewMut<RenderOnFloor>,
         mut renderables: ViewMut<Renderable>| {
            let item_id = entities.add_entity(
                (
                    &mut items,
                    &mut consumables,
                    &mut coords,
                    &mut names,
                    &mut provides_healings,
                    &mut render_on_floors,
                    &mut renderables,
                ),
                (
                    Item {},
                    Consumable {},
                    Coord(pos.into()),
                    Name("Health Potion".into()),
                    ProvidesHealing { heal_amount: 8 },
                    RenderOnFloor {},
                    Renderable {
                        sym: GameSym::HealthPotion,
                        fg: Color::MAGENTA,
                        bg: Color::BLACK,
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
         mut coords: ViewMut<Coord>,
         mut inflicts_damages: ViewMut<InflictsDamage>,
         mut items: ViewMut<Item>,
         mut names: ViewMut<Name>,
         mut rangeds: ViewMut<Ranged>,
         mut render_on_floors: ViewMut<RenderOnFloor>,
         mut renderables: ViewMut<Renderable>| {
            let item_id = entities.add_entity(
                (
                    &mut items,
                    &mut consumables,
                    &mut coords,
                    &mut inflicts_damages,
                    &mut names,
                    &mut rangeds,
                    &mut render_on_floors,
                    &mut renderables,
                ),
                (
                    Item {},
                    Consumable {},
                    Coord(pos.into()),
                    InflictsDamage { damage: 8 },
                    Name("Magic Missile Scroll".into()),
                    Ranged { range: 6 },
                    RenderOnFloor {},
                    Renderable {
                        sym: GameSym::MagicMissileScroll,
                        fg: Color::CYAN,
                        bg: Color::BLACK,
                    },
                ),
            );

            map.place_entity(item_id, pos, false);
        },
    );
}

fn spawn_fireball_scroll(world: &World, pos: (i32, i32)) {
    world.run(
        |mut map: UniqueViewMut<Map>,
         mut entities: EntitiesViewMut,
         mut aoes: ViewMut<AreaOfEffect>,
         mut consumables: ViewMut<Consumable>,
         mut coords: ViewMut<Coord>,
         mut inflicts_damages: ViewMut<InflictsDamage>,
         mut items: ViewMut<Item>,
         mut names: ViewMut<Name>,
         mut rangeds: ViewMut<Ranged>,
         (mut render_on_floors, mut renderables): (ViewMut<RenderOnFloor>, ViewMut<Renderable>)| {
            let item_id = entities.add_entity(
                (
                    &mut items,
                    &mut aoes,
                    &mut consumables,
                    &mut coords,
                    &mut inflicts_damages,
                    &mut names,
                    &mut rangeds,
                    &mut render_on_floors,
                    &mut renderables,
                ),
                (
                    Item {},
                    AreaOfEffect { radius: 3 },
                    Consumable {},
                    Coord(pos.into()),
                    InflictsDamage { damage: 20 },
                    Name("Fireball Scroll".into()),
                    Ranged { range: 6 },
                    RenderOnFloor {},
                    Renderable {
                        sym: GameSym::FireballScroll,
                        fg: Color::ORANGE,
                        bg: Color::BLACK,
                    },
                ),
            );

            map.place_entity(item_id, pos, false);
        },
    );
}

fn spawn_sleep_scroll(world: &World, pos: (i32, i32)) {
    world.run(
        |mut map: UniqueViewMut<Map>,
         mut entities: EntitiesViewMut,
         mut aoes: ViewMut<AreaOfEffect>,
         mut consumables: ViewMut<Consumable>,
         mut coords: ViewMut<Coord>,
         mut inflicts_sleeps: ViewMut<InflictsSleep>,
         mut items: ViewMut<Item>,
         mut names: ViewMut<Name>,
         mut rangeds: ViewMut<Ranged>,
         (mut render_on_floors, mut renderables): (ViewMut<RenderOnFloor>, ViewMut<Renderable>)| {
            let item_id = entities.add_entity(
                (
                    &mut items,
                    &mut aoes,
                    &mut consumables,
                    &mut coords,
                    &mut inflicts_sleeps,
                    &mut names,
                    &mut rangeds,
                    &mut render_on_floors,
                    &mut renderables,
                ),
                (
                    Item {},
                    AreaOfEffect { radius: 1 },
                    Consumable {},
                    Coord(pos.into()),
                    InflictsSleep { sleepiness: 36 },
                    Name("Sleep Scroll".into()),
                    Ranged { range: 6 },
                    RenderOnFloor {},
                    Renderable {
                        sym: GameSym::SleepScroll,
                        fg: Color::PINK,
                        bg: Color::BLACK,
                    },
                ),
            );

            map.place_entity(item_id, pos, false);
        },
    );
}

fn spawn_monster(world: &World, pos: (i32, i32), sym: GameSym, name: String, fg: Color) {
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
                    Name(name),
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
        spawn_monster(world, pos, *sym, name.to_string(), *fg);
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

    for room in rooms {
        fill_room_with_spawns(world, &room);
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
