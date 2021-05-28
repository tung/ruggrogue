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
    experience::{self, Difficulty},
    gamesym::GameSym,
    magicnum,
    map::{Map, Rect},
    player, GameSeed,
};
use ruggle::util::Color;

const EQUIPMENT_SPAWN_PERIOD: u32 = 7;

const MONSTERS: [(GameSym, &str, (u8, u8, u8)); 25] = [
    (GameSym::Blob, "Blob", (89, 162, 191)),
    (GameSym::Bat, "Bat", (128, 128, 128)),
    (GameSym::Crab, "Crab", (255, 0, 0)),
    (GameSym::Snake, "Snake", (0, 153, 0)),
    (GameSym::Goblin, "Goblin", (34, 187, 59)),
    (GameSym::Kobold, "Kobold", (122, 181, 73)),
    (GameSym::Gnome, "Gnome", (134, 204, 199)),
    (GameSym::Orc, "Orc", (202, 100, 39)),
    (GameSym::Unicorn, "Unicorn", (255, 150, 255)),
    (GameSym::Pirate, "Pirate", (0, 134, 255)),
    (GameSym::Lizardon, "Lizardon", (89, 153, 175)),
    (GameSym::Ghost, "Ghost", (254, 255, 255)),
    (GameSym::Skeleton, "Skeleton", (222, 211, 195)),
    (GameSym::Ogre, "Ogre", (202, 101, 39)),
    (GameSym::Naga, "Naga", (211, 205, 137)),
    (GameSym::Warlock, "Warlock", (168, 44, 234)),
    (GameSym::Demon, "Demon", (218, 0, 0)),
    (GameSym::Sentinel, "Sentinel", (168, 44, 234)),
    (GameSym::Robber, "Robber", (82, 84, 255)),
    (GameSym::SkateboardKid, "Skateboard Kid", (255, 127, 0)),
    (GameSym::Jellybean, "Jellybean", (192, 96, 192)),
    (GameSym::Alien, "Alien", (65, 168, 58)),
    (GameSym::Dweller, "Dweller", (58, 149, 140)),
    (GameSym::LittleHelper, "Little Helper", (0, 153, 0)),
    (GameSym::BigHelper, "Big Helper", (255, 99, 99)),
];

const WEAPONS: [(GameSym, &str, (u8, u8, u8)); 10] = [
    (GameSym::Knife, "Knife", (165, 165, 165)),
    (GameSym::Club, "Club", (137, 88, 38)),
    (GameSym::Hatchet, "Hatchet", (165, 165, 165)),
    (GameSym::Spear, "Spear", (137, 88, 38)),
    (GameSym::Rapier, "Rapier", (198, 159, 39)),
    (GameSym::Saber, "Saber", (165, 165, 165)),
    (GameSym::Longsword, "Longsword", (165, 165, 165)),
    (GameSym::Crowbar, "Crowbar", (255, 127, 0)),
    (GameSym::Tonfa, "Tonfa", (82, 84, 255)),
    (GameSym::BeamSword, "Beam Sword", (255, 255, 0)),
];

const ARMORS: [(GameSym, &str, (u8, u8, u8)); 10] = [
    (GameSym::Jerkin, "Jerkin", (170, 97, 32)),
    (GameSym::Coat, "Coat", (170, 97, 32)),
    (GameSym::WoodenShield, "Wooden Shield", (191, 92, 0)),
    (GameSym::TowerShield, "Tower Shield", (165, 165, 165)),
    (GameSym::KiteShield, "Kite Shield", (165, 165, 165)),
    (GameSym::StuddedArmor, "Studded Armor", (170, 97, 32)),
    (GameSym::Hauberk, "Hauberk", (165, 165, 165)),
    (GameSym::Platemail, "Platemail", (165, 165, 165)),
    (GameSym::ArmyHelmet, "Army Helmet", (77, 120, 78)),
    (GameSym::FlakJacket, "Flak Jacket", (77, 120, 78)),
];

/// Spawn an entity whose purpose is to track the total amount of experience points that could
/// theoretically be gained in the game in order to increase difficulty over time.
pub fn spawn_difficulty(mut entities: EntitiesViewMut, mut exps: ViewMut<Experience>) -> EntityId {
    entities.add_entity(
        (&mut exps,),
        (Experience {
            level: 1,
            exp: 0,
            next: 100,
            base: 0,
        },),
    )
}

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

fn spawn_item(world: &World, pos: (i32, i32), name: String, sym: GameSym, fg: Color) -> EntityId {
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
                    Name(name),
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
    let item_id = spawn_item(world, pos, "Ration".into(), GameSym::Ration, Color::BROWN);
    let (entities, mut consumables, mut nutritions) =
        world.borrow::<(EntitiesView, ViewMut<Consumable>, ViewMut<Nutrition>)>();

    entities.add_component(
        (&mut consumables, &mut nutritions),
        (Consumable {}, Nutrition(500)),
        item_id,
    );
}

fn spawn_health_potion(world: &World, pos: (i32, i32)) {
    let item_id = spawn_item(
        world,
        pos,
        "Health Potion".into(),
        GameSym::HealthPotion,
        Color::MAGENTA,
    );
    let (entities, mut consumables, mut provides_healings) =
        world.borrow::<(EntitiesView, ViewMut<Consumable>, ViewMut<ProvidesHealing>)>();

    entities.add_component(
        (&mut consumables, &mut provides_healings),
        (Consumable {}, ProvidesHealing { heal_amount: 15 }),
        item_id,
    );
}

fn spawn_magic_missile_scroll(world: &World, pos: (i32, i32)) {
    let item_id = spawn_item(
        world,
        pos,
        "Magic Missile Scroll".into(),
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
        "Fireball Scroll".into(),
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
        "Sleep Scroll".into(),
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

fn rescale_level<R: Rng>(level: f32, scale: usize, rng: &mut R) -> usize {
    let monsters_range = MONSTERS.len().saturating_sub(1).max(1) as f32;
    let rescaled = ((level - 1.0) / monsters_range).clamp(0.0, 1.0) * scale as f32;

    experience::f32_round_random(rescaled, rng) as usize
}

fn spawn_weapon<R: Rng>(world: &World, rng: &mut R, pos: (i32, i32), level: f32, bonus: i32) {
    let (sym, name, rgb) = WEAPONS[rescale_level(level, WEAPONS.len().saturating_sub(1), rng)];
    let level = experience::f32_round_random(level, rng);
    let item_id = spawn_item(
        world,
        pos,
        format!("+{} {}", level + bonus, name),
        sym,
        rgb.into(),
    );
    let (entities, mut combat_bonuses, mut equip_slots) =
        world.borrow::<(EntitiesView, ViewMut<CombatBonus>, ViewMut<EquipSlot>)>();

    entities.add_component(
        (&mut combat_bonuses, &mut equip_slots),
        (
            CombatBonus {
                attack: experience::calc_weapon_attack(level + bonus),
                defense: 0.0,
            },
            EquipSlot::Weapon,
        ),
        item_id,
    );
}

fn spawn_armor<R: Rng>(world: &World, rng: &mut R, pos: (i32, i32), level: f32, bonus: i32) {
    let (sym, name, rgb) = ARMORS[rescale_level(level, ARMORS.len().saturating_sub(1), rng)];
    let level = experience::f32_round_random(level, rng);
    let item_id = spawn_item(
        world,
        pos,
        format!("+{} {}", level + bonus, name),
        sym,
        rgb.into(),
    );
    let (entities, mut combat_bonuses, mut equip_slots) =
        world.borrow::<(EntitiesView, ViewMut<CombatBonus>, ViewMut<EquipSlot>)>();

    entities.add_component(
        (&mut combat_bonuses, &mut equip_slots),
        (
            CombatBonus {
                attack: 0.0,
                defense: experience::calc_armor_defense(level + bonus),
            },
            EquipSlot::Armor,
        ),
        item_id,
    );
}

fn spawn_monster(world: &World, pos: (i32, i32), level: i32, sym: GameSym, name: &str, fg: Color) {
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
                max_hp: experience::calc_monster_max_hp(level),
                hp: experience::calc_monster_max_hp(level),
                attack: experience::calc_monster_attack(level),
                defense: experience::calc_monster_defense(level),
            },
            Coord(pos.into()),
            Experience {
                level: 1,
                exp: 0,
                next: 0,
                base: 0,
            },
            FieldOfView::new(8),
            GivesExperience(experience::calc_monster_exp(level)),
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
    let mut level = {
        let difficulty = world.borrow::<UniqueView<Difficulty>>();
        let exps = world.borrow::<View<Experience>>();
        difficulty.get_round_random(&exps, rng)
    };
    if rng.gen_ratio(4, 5) {
        level = (level - rng.gen_range(1, 4)).max(1);
        if level > 1 && rng.gen() {
            level = rng.gen_range(1, level);
        }
    }
    let (sym, name, fg) = MONSTERS[(level.max(1) as usize)
        .min(MONSTERS.len())
        .saturating_sub(1)];

    spawn_monster(world, pos, level, sym, name, fg.into());
}

fn spawn_random_item_at<R: Rng>(world: &World, rng: &mut R, pos: (i32, i32)) {
    if rng.gen_ratio(1, 11) {
        // Spawn weapon or armor.
        let level = {
            let difficulty = world.borrow::<UniqueView<Difficulty>>();
            let exps = world.borrow::<View<Experience>>();
            difficulty.as_f32(&exps)
        };
        // Spawn items (really equipment) at a slightly higher level than average.
        let bonus = rng.gen_range(1, 4);

        if rng.gen() {
            spawn_weapon(world, rng, pos, level, bonus);
        } else {
            spawn_armor(world, rng, pos, level, bonus);
        }
    } else {
        // Spawn an item.
        type ItemFn = fn(&World, (i32, i32));
        let choice: Result<&(usize, ItemFn), _> = [
            (3, spawn_health_potion as _),
            (3, spawn_magic_missile_scroll as _),
            (2, spawn_fireball_scroll as _),
            (2, spawn_sleep_scroll as _),
        ]
        .choose_weighted(rng, |&(weight, _)| weight);

        if let Ok((_, item_fn)) = choice {
            item_fn(world, pos);
        }
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

fn pick_random_pos_in_room<R: Rng>(world: &World, rng: &mut R) -> Option<(i32, i32)> {
    let map = world.borrow::<UniqueView<Map>>();
    let items = world.borrow::<View<Item>>();

    map.rooms.choose(rng).and_then(|room| {
        room.iter_xy()
            .filter(|&(x, y)| !map.iter_entities_at(x, y).any(|id| items.contains(id)))
            .choose(rng)
    })
}

fn spawn_guaranteed_equipment<R: Rng>(world: &World, rng: &mut R) {
    let depth = world.borrow::<UniqueView<Map>>().depth;

    // Spawn starting equipment in the first room of Depth 1.
    if depth == 1 {
        let mut start_equips = [(0, 0); 2];
        let num = world
            .borrow::<UniqueView<Map>>()
            .rooms
            .first()
            .map(|room| {
                room.iter_xy()
                    .choose_multiple_fill(rng, &mut start_equips[..])
            })
            .unwrap_or(0);

        if num > 0 {
            start_equips[0..num].shuffle(rng);
            spawn_armor(world, rng, start_equips[0], 1.0, 0);
            spawn_weapon(
                world,
                rng,
                start_equips[if num > 1 { 1 } else { 0 }],
                1.0,
                0,
            );
        }
    }

    // Spawn a weapon and an armor once every so many levels, using separate RNGs for stable,
    // isolated random numbers to decide the exact levels.
    let depth_period_base = depth as u32 / EQUIPMENT_SPAWN_PERIOD * EQUIPMENT_SPAWN_PERIOD;
    let game_seed = world.borrow::<UniqueView<GameSeed>>().0;

    let mut periodic_weapon_rng = {
        // Offset the weapon spawn period based on the low bytes of the game seed.
        let offset = (game_seed & 0xffffffff) as u32 % EQUIPMENT_SPAWN_PERIOD;
        let mut hasher = WyHash::with_seed(magicnum::SPAWN_GUARANTEED_WEAPON);
        hasher.write_u64(game_seed);
        hasher.write_u32((depth as u32 + offset) / EQUIPMENT_SPAWN_PERIOD);
        Pcg32::seed_from_u64(hasher.finish())
    };

    // Pick a random number in a range one-short of the period to guarantee a "gap" level, to make
    // the period less obvious.
    if periodic_weapon_rng.gen_range(0, EQUIPMENT_SPAWN_PERIOD - 1)
        == depth as u32 - depth_period_base
    {
        let weapon_pos = pick_random_pos_in_room(world, &mut periodic_weapon_rng);
        if let Some(pos) = weapon_pos {
            let level = {
                let difficulty = world.borrow::<UniqueView<Difficulty>>();
                let exps = world.borrow::<View<Experience>>();
                difficulty.as_f32(&exps)
            };
            spawn_weapon(world, &mut periodic_weapon_rng, pos, level, 0);
        }
    }

    let mut periodic_armor_rng = {
        // Offset the armor spawn period based on the high bytes of the game seed.
        let offset = ((game_seed >> 32) & 0xffffffff) as u32 % EQUIPMENT_SPAWN_PERIOD;
        let mut hasher = WyHash::with_seed(magicnum::SPAWN_GUARANTEED_ARMOR);
        hasher.write_u64(game_seed);
        hasher.write_u32((depth as u32 + offset) / EQUIPMENT_SPAWN_PERIOD);
        Pcg32::seed_from_u64(hasher.finish())
    };

    // Random number one-short of the period, for the same reason as the weapon spawn.
    if periodic_armor_rng.gen_range(0, EQUIPMENT_SPAWN_PERIOD - 1)
        == depth as u32 - depth_period_base
    {
        let armor_pos = pick_random_pos_in_room(world, &mut periodic_armor_rng);
        if let Some(pos) = armor_pos {
            let level = {
                let difficulty = world.borrow::<UniqueView<Difficulty>>();
                let exps = world.borrow::<View<Experience>>();
                difficulty.as_f32(&exps)
            };
            spawn_armor(world, &mut periodic_armor_rng, pos, level, 0);
        }
    }
}

fn spawn_guaranteed_ration<R: Rng>(world: &World, rng: &mut R) {
    let ration_pos = pick_random_pos_in_room(world, rng);

    if let Some(ration_pos) = ration_pos {
        spawn_ration(world, ration_pos);
    }
}

pub fn fill_rooms_with_spawns(world: &World) {
    let mut rng = {
        let mut hasher = WyHash::with_seed(magicnum::FILL_ROOM_WITH_SPAWNS);
        hasher.write_u64(world.borrow::<UniqueView<GameSeed>>().0);
        hasher.write_i32(world.borrow::<UniqueView<Map>>().depth);
        Pcg32::seed_from_u64(hasher.finish())
    };

    spawn_guaranteed_equipment(world, &mut rng);

    let rooms = world
        .borrow::<UniqueViewMut<Map>>()
        .rooms
        .iter()
        .skip(1)
        .copied()
        .collect::<Vec<_>>();

    for room in &rooms {
        fill_room_with_spawns(world, &mut rng, room);
    }

    spawn_guaranteed_ration(world, &mut rng);
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
