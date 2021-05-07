use rand::{
    seq::{IteratorRandom, SliceRandom},
    Rng, SeedableRng,
};
use rand_pcg::Pcg32;
use shipyard::{
    AllStoragesViewMut, EntitiesView, EntitiesViewMut, EntityId, IntoIter, Shiperator, UniqueView,
    UniqueViewMut, View, ViewMut, World,
};
use std::{collections::HashSet, hash::Hasher};
use wyhash::WyHash;

use crate::{
    components::*,
    experience,
    gamesym::GameSym,
    magicnum,
    map::{Map, Rect},
    player, GameSeed,
};
use ruggle::util::Color;

/// Spawn a player.
///
/// NOTE: The player must be positioned on the map at some point after this.
pub fn spawn_player(
    mut entities: EntitiesViewMut,
    mut combat_stats: ViewMut<CombatStats>,
    mut coords: ViewMut<Coord>,
    mut equipments: ViewMut<Equipment>,
    mut exps: ViewMut<Experience>,
    mut fovs: ViewMut<FieldOfView>,
    mut inventories: ViewMut<Inventory>,
    mut names: ViewMut<Name>,
    mut players: ViewMut<Player>,
    (mut render_on_maps, mut renderables, mut stomachs): (
        ViewMut<RenderOnMap>,
        ViewMut<Renderable>,
        ViewMut<Stomach>,
    ),
) -> EntityId {
    let id = entities.add_entity(
        (
            &mut players,
            &mut combat_stats,
            &mut coords,
            &mut exps,
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
                max_hp: experience::calc_player_max_hp(1),
                hp: experience::calc_player_max_hp(1),
                attack: experience::calc_player_attack(1),
                defense: experience::calc_player_defense(1),
            },
            Coord((0, 0).into()),
            Experience {
                level: 1,
                exp: 0,
                next: 100,
                base: 0,
            },
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
    );

    entities.add_component(
        (&mut equipments,),
        (Equipment {
            weapon: None,
            armor: None,
        },),
        id,
    );

    id
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

fn spawn_knife(world: &World, pos: (i32, i32)) {
    let item_id = spawn_item(world, pos, "Knife", GameSym::Knife, Color::GRAY);
    let (entities, mut combat_bonuses, mut equip_slots) =
        world.borrow::<(EntitiesView, ViewMut<CombatBonus>, ViewMut<EquipSlot>)>();

    entities.add_component(
        (&mut combat_bonuses, &mut equip_slots),
        (
            CombatBonus {
                attack: experience::calc_weapon_attack(1),
                defense: 0.0,
            },
            EquipSlot::Weapon,
        ),
        item_id,
    );
}

fn spawn_wooden_shield(world: &World, pos: (i32, i32)) {
    let item_id = spawn_item(
        world,
        pos,
        "Wooden Shield",
        GameSym::WoodenShield,
        Color::BROWN,
    );
    let (entities, mut combat_bonuses, mut equip_slots) =
        world.borrow::<(EntitiesView, ViewMut<CombatBonus>, ViewMut<EquipSlot>)>();

    entities.add_component(
        (&mut combat_bonuses, &mut equip_slots),
        (
            CombatBonus {
                attack: 0.0,
                defense: experience::calc_armor_defense(1),
            },
            EquipSlot::Armor,
        ),
        item_id,
    );
}

fn spawn_monster(world: &World, pos: (i32, i32), sym: GameSym, name: &str, fg: Color) {
    let monster_id = world.borrow::<EntitiesViewMut>().add_entity(
        (
            &mut world.borrow::<ViewMut<Monster>>(),
            &mut world.borrow::<ViewMut<BlocksTile>>(),
            &mut world.borrow::<ViewMut<CombatStats>>(),
            &mut world.borrow::<ViewMut<Coord>>(),
            &mut world.borrow::<ViewMut<Experience>>(),
            &mut world.borrow::<ViewMut<FieldOfView>>(),
            &mut world.borrow::<ViewMut<GivesExperience>>(),
            &mut world.borrow::<ViewMut<Name>>(),
            &mut world.borrow::<ViewMut<RenderOnMap>>(),
            &mut world.borrow::<ViewMut<Renderable>>(),
        ),
        (
            Monster {},
            BlocksTile {},
            CombatStats {
                max_hp: experience::calc_monster_max_hp(1),
                hp: experience::calc_monster_max_hp(1),
                attack: experience::calc_monster_attack(1),
                defense: experience::calc_monster_defense(1),
            },
            Coord(pos.into()),
            Experience {
                level: 1,
                exp: 0,
                next: 0,
                base: 0,
            },
            FieldOfView::new(8),
            GivesExperience(10),
            Name(name.into()),
            RenderOnMap {},
            Renderable {
                sym,
                fg,
                bg: Color::BLACK,
            },
        ),
    );

    world
        .borrow::<UniqueViewMut<Map>>()
        .place_entity(monster_id, pos, true);
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
    type ItemFn = fn(&World, (i32, i32));

    let choice: Result<&(usize, ItemFn), _> = [
        (3, spawn_health_potion as _),
        (3, spawn_magic_missile_scroll as _),
        (2, spawn_fireball_scroll as _),
        (2, spawn_sleep_scroll as _),
        (1, spawn_knife as _),
        (1, spawn_wooden_shield as _),
    ]
    .choose_weighted(rng, |&(weight, _)| weight);

    if let Ok((_, item_fn)) = choice {
        item_fn(world, pos);
    }
}

fn fill_room_with_spawns<R: Rng>(world: &World, rng: &mut R, room: &Rect) {
    let depth = world.borrow::<UniqueView<Map>>().depth;
    let depth = if depth < 1 { 1usize } else { depth as usize };

    if rng.gen_ratio(1, 4) {
        let num = rng.gen_range(1, 2);

        for pos in room.iter_xy().choose_multiple(rng, num) {
            spawn_random_item_at(world, rng, pos);
        }
    }

    if rng.gen_ratio(1, 2) {
        let num = rng.gen_range(1, 1 + ((depth + 1) / 2).min(3));

        for pos in room.iter_xy().choose_multiple(rng, num) {
            spawn_random_monster_at(world, rng, pos);
        }
    }
}

pub fn fill_rooms_with_spawns(world: &World) {
    let rooms = world
        .borrow::<UniqueViewMut<Map>>()
        .rooms
        .iter()
        .skip(1)
        .copied()
        .collect::<Vec<_>>();
    let mut rng = {
        let mut hasher = WyHash::with_seed(magicnum::FILL_ROOM_WITH_SPAWNS);
        hasher.write_u64(world.borrow::<UniqueView<GameSeed>>().0);
        hasher.write_i32(world.borrow::<UniqueView<Map>>().depth);
        Pcg32::seed_from_u64(hasher.finish())
    };

    for room in &rooms {
        fill_room_with_spawns(world, &mut rng, room);
    }

    let ration_pos = {
        let (map, items) = world.borrow::<(UniqueView<Map>, View<Item>)>();
        rooms.choose(&mut rng).and_then(|ration_room| {
            ration_room
                .iter_xy()
                .filter(|&(x, y)| !map.iter_entities_at(x, y).any(|id| items.contains(id)))
                .choose(&mut rng)
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
