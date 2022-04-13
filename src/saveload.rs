use serde::{Deserialize, Serialize};
use serde_json::{Deserializer, Serializer};
use shipyard::{
    AllStoragesViewMut, EntitiesView, EntitiesViewMut, EntityId, Get, IntoIter, Shiperator,
    UniqueView, UniqueViewMut, View, ViewMut, World,
};
use std::{
    collections::{hash_map::HashMap, hash_set::HashSet},
    error, fmt,
    fs::{self, File},
    io::{BufRead, BufReader, BufWriter, Write},
    path::Path,
};

#[cfg(target_os = "emscripten")]
use crate::ruggrogue_sync_idbfs;
use crate::{
    components::*,
    experience::Difficulty,
    map::Map,
    message::Messages,
    player::{PlayerAlive, PlayerId},
    spawn, BaseEquipmentLevel, GameSeed, TurnCount, Wins,
};

#[cfg(target_os = "emscripten")]
const SAVE_FILENAME: &str = "/ruggrogue/savegame.txt";

#[cfg(not(target_os = "emscripten"))]
const SAVE_FILENAME: &str = "savegame.txt";

type BoxedError = Box<dyn error::Error>;

/// Game-specific errors that can occur when loading a save file.
#[derive(Debug)]
pub enum LoadError {
    DuplicateComponent(usize, &'static str),
    DuplicateUnique(usize, &'static str),
    MissingUnique(&'static str),
    UnknownId(EntityId),
    UnrecognizedLine(usize),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::DuplicateComponent(line_num, which) => {
                write!(f, "line {}: duplicate {} component", line_num, which)
            }
            Self::DuplicateUnique(line_num, which) => {
                write!(f, "line {}: duplicate {} unique", line_num, which)
            }
            Self::MissingUnique(which) => write!(f, "missing {} unique", which),
            Self::UnknownId(id) => write!(f, "unknown entity ID {:?}", *id),
            Self::UnrecognizedLine(line_num) => write!(f, "line {}: unrecognized line", line_num),
        }
    }
}

impl error::Error for LoadError {}

pub fn save_file_exists() -> bool {
    Path::new(SAVE_FILENAME).exists()
}

pub fn delete_save_file() {
    if save_file_exists() {
        if let Err(e) = fs::remove_file(SAVE_FILENAME) {
            eprintln!("Warning: saveload::delete_save_file: {}", e);
        }
    }
}

/// Save a unique as an asterisk, a tab, its type, a tab and its serialized data in a single line.
fn save_named_unique<W, T>(world: &World, mut writer: &mut W, name: &str) -> Result<(), BoxedError>
where
    T: 'static + Send + Sync + Serialize,
    W: Write,
{
    write!(writer, "*\t{}\t", name)?;
    world
        .borrow::<UniqueView<T>>()
        .serialize(&mut Serializer::new(&mut writer))?;
    writer.write_all(b"\n")?;
    Ok(())
}

macro_rules! save_unique {
    ($type:ty, $world:expr, $writer:expr) => {
        save_named_unique::<_, $type>($world, $writer, stringify!($type))
    };
}

/// Save components of a storage as an entity ID, a tab, its type, a tab and its serialized data,
/// one per line.
fn save_named_storage<W, T>(world: &World, mut writer: &mut W, name: &str) -> Result<(), BoxedError>
where
    T: 'static + Send + Sync + Serialize,
    W: Write,
{
    for (id, component) in world.borrow::<View<T>>().iter().with_id() {
        id.serialize(&mut Serializer::new(&mut writer))?;
        write!(writer, "\t{}\t", name)?;
        component.serialize(&mut Serializer::new(&mut writer))?;
        writer.write_all(b"\n")?;
    }

    Ok(())
}

macro_rules! save_storage {
    ($type:ty, $world:expr, $writer:expr) => {
        save_named_storage::<_, $type>($world, $writer, stringify!($type))
    };
}

/// Save all data in uniques and component storages to the save file.
pub fn save_game(world: &World) -> Result<(), BoxedError> {
    let mut writer = BufWriter::new(File::create(SAVE_FILENAME)?);

    save_unique!(GameSeed, world, &mut writer)?;
    save_unique!(TurnCount, world, &mut writer)?;
    save_unique!(Wins, world, &mut writer)?;
    save_unique!(BaseEquipmentLevel, world, &mut writer)?;
    save_unique!(Difficulty, world, &mut writer)?;
    save_unique!(Messages, world, &mut writer)?;
    save_unique!(PlayerAlive, world, &mut writer)?;
    save_unique!(PlayerId, world, &mut writer)?;
    save_unique!(Map, world, &mut writer)?;

    save_storage!(AreaOfEffect, world, &mut writer)?;
    save_storage!(Asleep, world, &mut writer)?;
    save_storage!(BlocksTile, world, &mut writer)?;
    save_storage!(CombatBonus, world, &mut writer)?;
    save_storage!(CombatStats, world, &mut writer)?;
    save_storage!(Consumable, world, &mut writer)?;
    save_storage!(Coord, world, &mut writer)?;
    save_storage!(EquipSlot, world, &mut writer)?;
    save_storage!(Equipment, world, &mut writer)?;
    save_storage!(Experience, world, &mut writer)?;
    save_storage!(FieldOfView, world, &mut writer)?;
    save_storage!(GivesExperience, world, &mut writer)?;
    save_storage!(InflictsDamage, world, &mut writer)?;
    save_storage!(InflictsSleep, world, &mut writer)?;
    save_storage!(Inventory, world, &mut writer)?;
    save_storage!(Item, world, &mut writer)?;
    save_storage!(Monster, world, &mut writer)?;
    save_storage!(Name, world, &mut writer)?;
    save_storage!(Nutrition, world, &mut writer)?;
    save_storage!(Player, world, &mut writer)?;
    save_storage!(ProvidesHealing, world, &mut writer)?;
    save_storage!(Ranged, world, &mut writer)?;
    save_storage!(RenderOnFloor, world, &mut writer)?;
    save_storage!(RenderOnMap, world, &mut writer)?;
    save_storage!(Renderable, world, &mut writer)?;
    save_storage!(Stomach, world, &mut writer)?;
    save_storage!(Tally, world, &mut writer)?;
    save_storage!(Victory, world, &mut writer)?;

    writer.flush()?;

    #[cfg(target_os = "emscripten")]
    unsafe {
        ruggrogue_sync_idbfs();
    }

    Ok(())
}

/// Attempt to deserialize a unique of the given named type from a line, inserting the data into
/// the given `dest` on success.
///
/// Returns `Ok(true)` if the line was successfully parsed, `Ok(false)` if the line wasn't parsed
/// but might be something else, and `Err` if the line was parsed as a duplicate of a unique that
/// was already present in `dest`.
fn deserialize_named_unique<'a, T>(
    line: &'a str,
    line_num: usize,
    dest: &mut Option<T>,
    name: &'static str,
) -> Result<bool, LoadError>
where
    T: Deserialize<'a>,
{
    let line = if let Some(unprefixed) = line
        .strip_prefix(name)
        .and_then(|s| s.strip_prefix(char::is_whitespace))
    {
        unprefixed.trim_start()
    } else {
        return Ok(false);
    };
    let mut ds = Deserializer::from_str(line);

    if let Ok(parsed) = T::deserialize(&mut ds) {
        if ds.end().is_ok() {
            if dest.is_none() {
                *dest = Some(parsed);
                Ok(true)
            } else {
                Err(LoadError::DuplicateUnique(line_num, name))
            }
        } else {
            Ok(false)
        }
    } else {
        Ok(false)
    }
}

macro_rules! deserialize_unique {
    ($type:ty, $line:expr, $line_num:expr, $dest:expr) => {
        deserialize_named_unique::<$type>($line, $line_num, $dest, stringify!($type))
    };
}

/// Attempt to deserialize component data of a given named type from part of a line, adding the
/// component to the entity with the given `id` on success.
///
/// Returns `Ok(true)` if the component was successfully parsed, `Ok(false)` if the data wasn't
/// parsed but might be something else, and `Err` if the data was parsed as a duplicate of a
/// component that the entity already had.
fn deserialize_named_component<'a, T>(
    world: &World,
    maybe_data: &'a str,
    line_num: usize,
    id: EntityId,
    name: &'static str,
) -> Result<bool, LoadError>
where
    T: 'static + Send + Sync + Deserialize<'a>,
{
    let maybe_data = if let Some(unprefixed) = maybe_data
        .strip_prefix(name)
        .and_then(|s| s.strip_prefix(char::is_whitespace))
    {
        unprefixed.trim_start()
    } else {
        return Ok(false);
    };
    let mut ds = Deserializer::from_str(maybe_data);

    if let Ok(parsed) = T::deserialize(&mut ds) {
        if ds.end().is_ok() {
            let entities = world.borrow::<EntitiesView>();
            let mut storage = world.borrow::<ViewMut<T>>();

            if !storage.contains(id) {
                entities.add_component(&mut storage, parsed, id);
                Ok(true)
            } else {
                Err(LoadError::DuplicateComponent(line_num, name))
            }
        } else {
            Ok(false)
        }
    } else {
        Ok(false)
    }
}

macro_rules! deserialize_component {
    ($type:ty, $world:expr, $maybe_data:expr, $line_num:expr, $id:expr) => {
        deserialize_named_component::<$type>($world, $maybe_data, $line_num, $id, stringify!($type))
    };
}

/// Load data out of the save file, with a list of entities to be despawned afterwards.
///
/// Loading saved data involves loading and interpreting data line by line; uniques are loaded to
/// temporary space, while components are added to the entities in the world that are created if
/// they don't yet exist.  After some basic validation, the unique data is committed to the world,
/// and old entities are marked for despawning.
///
/// Loaded entities are created on a line-by-line basis, but loading can fail in a lot of ways,
/// which would normally leave a partially-loaded group of entities behind.  To prevent this,
/// freshly-created entities are added to the `despawn_ids` list, which is only cleared and
/// replaced with old existing entities after final validation of the loaded data.
fn load_save_file(world: &World, despawn_ids: &mut Vec<EntityId>) -> Result<(), BoxedError> {
    let mut game_seed: Option<GameSeed> = None;
    let mut turn_count: Option<TurnCount> = None;
    let mut wins: Option<Wins> = None;
    let mut base_equipment_level: Option<BaseEquipmentLevel> = None;
    let mut difficulty: Option<Difficulty> = None;
    let mut messages: Option<Messages> = None;
    let mut player_alive: Option<PlayerAlive> = None;
    let mut player_id: Option<PlayerId> = None;
    let mut map: Option<Map> = None;
    let mut old_to_new_ids: HashMap<EntityId, EntityId> = HashMap::new();
    let reader = BufReader::new(File::open(SAVE_FILENAME)?);

    for (line_num, line_bytes) in reader.lines().enumerate() {
        let line_num = line_num + 1;
        let line = line_bytes?;

        // A line starting with an asterisk should hold data for a unique.
        if let Some(maybe_unique) = line
            .strip_prefix('*')
            .and_then(|s| s.strip_prefix(char::is_whitespace))
        {
            let maybe_unique = maybe_unique.trim_start();

            // Try parsing the line as a unique.
            if deserialize_unique!(GameSeed, maybe_unique, line_num, &mut game_seed)?
                || deserialize_unique!(TurnCount, maybe_unique, line_num, &mut turn_count)?
                || deserialize_unique!(Wins, maybe_unique, line_num, &mut wins)?
                || deserialize_unique!(
                    BaseEquipmentLevel,
                    maybe_unique,
                    line_num,
                    &mut base_equipment_level
                )?
                || deserialize_unique!(Difficulty, maybe_unique, line_num, &mut difficulty)?
                || deserialize_unique!(Messages, maybe_unique, line_num, &mut messages)?
                || deserialize_unique!(PlayerAlive, maybe_unique, line_num, &mut player_alive)?
                || deserialize_unique!(PlayerId, maybe_unique, line_num, &mut player_id)?
                || deserialize_unique!(Map, maybe_unique, line_num, &mut map)?
            {
                continue;
            }
        }

        // Most lines should contain component data for an entity.
        if let Some((maybe_id, maybe_data)) = line.split_once(char::is_whitespace) {
            let save_id = EntityId::deserialize(&mut Deserializer::from_str(maybe_id))?;

            // Map entity_id into the current world, creating a new entity if needed.
            let live_id = if let Some(id) = old_to_new_ids.get(&save_id) {
                *id
            } else {
                // Add new entity to despawn_ids and old_to_new_ids.
                let new_id = world.borrow::<EntitiesViewMut>().add_entity((), ());
                despawn_ids.push(new_id);
                old_to_new_ids.insert(save_id, new_id);
                new_id
            };

            let maybe_data = maybe_data.trim_start();

            // Try parsing maybe_data and add it to the entity on success.
            if deserialize_component!(AreaOfEffect, world, maybe_data, line_num, live_id)?
                || deserialize_component!(Asleep, world, maybe_data, line_num, live_id)?
                || deserialize_component!(BlocksTile, world, maybe_data, line_num, live_id)?
                || deserialize_component!(CombatBonus, world, maybe_data, line_num, live_id)?
                || deserialize_component!(CombatStats, world, maybe_data, line_num, live_id)?
                || deserialize_component!(Consumable, world, maybe_data, line_num, live_id)?
                || deserialize_component!(Coord, world, maybe_data, line_num, live_id)?
                || deserialize_component!(EquipSlot, world, maybe_data, line_num, live_id)?
                || deserialize_component!(Equipment, world, maybe_data, line_num, live_id)?
                || deserialize_component!(Experience, world, maybe_data, line_num, live_id)?
                || deserialize_component!(FieldOfView, world, maybe_data, line_num, live_id)?
                || deserialize_component!(GivesExperience, world, maybe_data, line_num, live_id)?
                || deserialize_component!(InflictsDamage, world, maybe_data, line_num, live_id)?
                || deserialize_component!(InflictsSleep, world, maybe_data, line_num, live_id)?
                || deserialize_component!(Inventory, world, maybe_data, line_num, live_id)?
                || deserialize_component!(Item, world, maybe_data, line_num, live_id)?
                || deserialize_component!(Monster, world, maybe_data, line_num, live_id)?
                || deserialize_component!(Name, world, maybe_data, line_num, live_id)?
                || deserialize_component!(Nutrition, world, maybe_data, line_num, live_id)?
                || deserialize_component!(Player, world, maybe_data, line_num, live_id)?
                || deserialize_component!(ProvidesHealing, world, maybe_data, line_num, live_id)?
                || deserialize_component!(Ranged, world, maybe_data, line_num, live_id)?
                || deserialize_component!(RenderOnFloor, world, maybe_data, line_num, live_id)?
                || deserialize_component!(RenderOnMap, world, maybe_data, line_num, live_id)?
                || deserialize_component!(Renderable, world, maybe_data, line_num, live_id)?
                || deserialize_component!(Stomach, world, maybe_data, line_num, live_id)?
                || deserialize_component!(Tally, world, maybe_data, line_num, live_id)?
                || deserialize_component!(Victory, world, maybe_data, line_num, live_id)?
            {
                continue;
            }

            // No other kinds of lines are valid.
            return Err(Box::new(LoadError::UnrecognizedLine(line_num)));
        }

        return Err(Box::new(LoadError::UnrecognizedLine(line_num)));
    }

    // Check that all uniques are present.
    let game_seed = game_seed.ok_or(LoadError::MissingUnique("GameSeed"))?;
    let turn_count = turn_count.ok_or(LoadError::MissingUnique("TurnCount"))?;
    let wins = wins.ok_or(LoadError::MissingUnique("Wins"))?;
    let base_equipment_level =
        base_equipment_level.ok_or(LoadError::MissingUnique("BaseEquipmentLevel"))?;
    let mut difficulty = difficulty.ok_or(LoadError::MissingUnique("Difficulty"))?;
    let messages = messages.ok_or(LoadError::MissingUnique("Messages"))?;
    let player_alive = player_alive.ok_or(LoadError::MissingUnique("PlayerAlive"))?;
    let mut player_id = player_id.ok_or(LoadError::MissingUnique("PlayerId"))?;
    let mut map = map.ok_or(LoadError::MissingUnique("Map"))?;

    // Replace old save-internal entity IDs with new loaded entity IDs.
    difficulty.id = old_to_new_ids
        .get(&difficulty.id)
        .copied()
        .ok_or(LoadError::UnknownId(difficulty.id))?;
    player_id.0 = old_to_new_ids
        .get(&player_id.0)
        .copied()
        .ok_or(LoadError::UnknownId(player_id.0))?;

    // Ensure that we're only working with freshly-loaded entities below.
    let new_ids = old_to_new_ids
        .values()
        .copied()
        .collect::<HashSet<EntityId>>();

    // Replace entity IDs in equipment.
    for (_, equipment) in IntoIter::iter(&mut world.borrow::<ViewMut<Equipment>>())
        .with_id()
        .filter(|(id, _)| new_ids.contains(id))
    {
        if let Some(weapon) = &mut equipment.weapon {
            *weapon = old_to_new_ids
                .get(weapon)
                .copied()
                .ok_or(LoadError::UnknownId(*weapon))?;
        }
        if let Some(armor) = &mut equipment.armor {
            *armor = old_to_new_ids
                .get(armor)
                .copied()
                .ok_or(LoadError::UnknownId(*armor))?;
        }
    }

    // Replace entity IDs in inventories.
    for (_, inventory) in IntoIter::iter(&mut world.borrow::<ViewMut<Inventory>>())
        .with_id()
        .filter(|(id, _)| new_ids.contains(id))
    {
        for item in inventory.items.iter_mut() {
            *item = old_to_new_ids
                .get(item)
                .copied()
                .ok_or(LoadError::UnknownId(*item))?;
        }
    }

    // Place all Coord-carrying entities on the map.
    for (id, coord) in IntoIter::iter(&world.borrow::<View<Coord>>()).with_id() {
        let blocks_tile = world.borrow::<View<BlocksTile>>().try_get(id).is_ok();
        map.place_entity(id, coord.0.into(), blocks_tile);
    }

    // Commit loaded entities and mark old existing entities for despawning.
    despawn_ids.clear();
    despawn_ids.push(world.borrow::<UniqueView<Difficulty>>().id);
    despawn_ids.push(world.borrow::<UniqueView<PlayerId>>().0);

    // Commit uniques.
    world.borrow::<UniqueViewMut<GameSeed>>().0 = game_seed.0;
    world.borrow::<UniqueViewMut<TurnCount>>().0 = turn_count.0;
    world.borrow::<UniqueViewMut<Wins>>().0 = wins.0;
    world.borrow::<UniqueViewMut<BaseEquipmentLevel>>().0 = base_equipment_level.0;
    world
        .borrow::<UniqueViewMut<Difficulty>>()
        .replace(difficulty);
    world.borrow::<UniqueViewMut<Messages>>().replace(messages);
    world.borrow::<UniqueViewMut<PlayerAlive>>().0 = player_alive.0;
    world.borrow::<UniqueViewMut<PlayerId>>().0 = player_id.0;
    world.borrow::<UniqueViewMut<Map>>().replace(map);

    Ok(())
}

/// Load the game state stored in the save file and despawn entities that need despawning after the
/// process of loading succeeds or fails.
pub fn load_game(world: &World) -> Result<(), BoxedError> {
    let mut delete_ids = Vec::new();
    let result = load_save_file(world, &mut delete_ids);

    for id in delete_ids {
        spawn::despawn_entity(&mut world.borrow::<AllStoragesViewMut>(), id);
    }

    result
}

/// Helper module that converts a list of values into a run-length encoded vector of pairs when
/// serializing and deserializing it with Serde.
pub mod run_length_encoded {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<I, S, T>(source: I, s: S) -> Result<S::Ok, S::Error>
    where
        I: IntoIterator<Item = T>,
        S: Serializer,
        T: Copy + PartialEq + Serialize,
    {
        let mut buffer: Vec<(T, u32)> = Vec::new();

        for it in source {
            let incremented = if let Some(last) = buffer.last_mut() {
                if last.0 == it && last.1 < u32::MAX {
                    last.1 += 1;
                    true
                } else {
                    false
                }
            } else {
                false
            };
            if !incremented {
                buffer.push((it, 1));
            }
        }

        buffer.serialize(s)
    }

    pub fn deserialize<'de, D, T>(d: D) -> Result<Vec<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: Copy + Deserialize<'de>,
    {
        Ok(Vec::<(T, u32)>::deserialize(d)?
            .iter()
            .flat_map(|(it, n)| std::iter::repeat(*it).take(*n as usize))
            .collect::<Vec<T>>())
    }
}

/// Helper module that converts a BitVec into a run-length encoded list of u8s for Serde.
pub mod bit_vec {
    use bitvec::prelude::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(bit_vec: &BitVec, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        super::run_length_encoded::serialize(
            bit_vec.iter().by_val().map(|b| if b { 1u8 } else { 0u8 }),
            s,
        )
    }

    pub fn deserialize<'de, D>(d: D) -> Result<BitVec, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(super::run_length_encoded::deserialize::<_, u8>(d)?
            .iter()
            .map(|n| *n != 0)
            .collect::<BitVec>())
    }
}
