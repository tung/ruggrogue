# Entity Component System

Up until now, most of the data that has been covered in this book has been about technical things that the game needs just to run, like input events, surfaces, textures and timing information.
But beyond that is the data that defines RuggRogue as a game, such as the player, the map, monsters and items.
This game-specific data is all managed by a crate named [Shipyard](https://crates.io/crates/shipyard), and this chapter is all about Shipyard and how RuggRogue uses it.

By its own description:

> Shipyard is an Entity Component System focused on usability and speed.

Here, an *entity* is a lightweight ID whose role is to associate groups of *components* that hold the data describing the entity.
The main benefit of this is that it avoids the "talking sword" problem that you'd run into with an object-oriented approach: if you have NPCs that you can talk to, and a sword you can pick up and swing, how do you represent a talking sword?
In the object-oriented style of modelling game data, problems like this end up poking holes in the encapsulation the classes are supposed to have, and functionality drifts up the inheritance tree into a gigantic all-encompassing mega-class.
Game data modelled with entities and components instead avoids both of those issues; see Catherine West's RustConf 2018 closing keynote ([video](https://www.youtube.com/watch?v=aKLntZcp27M) and [notes](https://kyren.github.io/2018/09/14/rustconf-talk.html)) for more information.

In a game built fully in the ECS-style, *systems* are just functions that manipulate groups of entities according to what components they have.
<!-- However, RuggRogue mostly does *not* use Shipyard's systems, for reasons that will be discussed later. -->

## Shipyard 0.4

RuggRogue uses Shipyard **0.4**, but at the time of writing it is *not* the most recent version of Shipyard, which is **0.5**.
So what gives?
Well, 0.4 was the most up-to-date version of Shipyard when I started work on RuggRogue, and when 0.5 came out I ported the game over to it.
Unfortunately, [this broke the web build](https://github.com/tung/ruggrogue/commit/76454e69aa5734d98bda91869bdcec75f8152732), so it had to be reverted.
Therefore, RuggRogue uses Shipyard 0.4 and not 0.5.

In order to understand how RuggRogue reads and modifies its own game data, you'll need to understand the basics of Shipyard 0.4.
This is the point where I would link to the Shipyard 0.4 User's Guide that existed when I started writing the game, except it was replaced wholesale when Shipyard 0.5 came out, which has a bunch of differences.
I could build and host that old guide myself, but putting up documentation for an older version of somebody else's library with no indication that it's stale would be problematic.
As such, most of this chapter is going to serve as a crash course on Shipyard 0.4, which should provide a foundation for understanding the code in RuggRogue that works with game data.

If you have Rust installed and you have the RuggRogue source code, you can peruse a detailed reference to Shipyard 0.4's API, along with all of the other crates used by RuggRogue, by typing `cargo doc --open`.
Shipyard's source code also contains its user guide that can be built with [mdBook](https://crates.io/crates/mdbook), so you can check out older versions of its source code and run it through mdBook to read it.

## The World

All data in Shipyard is stored in a `World` that consists of:

- **Entities**: They're just IDs, but the world tracks which ones are still alive.
- **Components**: Associated with entities; Shipyard stores components of each unique type in separate storages.
- **Uniques**: Like components, but not associated with any entity; often called *resources* in other Rust ECS crates.

RuggRogue creates one as the very first thing in the `main` function in the `src/main.rs` file:

```rust,ignore
let world = World::new();
```

Every bit of data specific to RuggRogue as a game is stored in the world, such as the map, the player, monsters and items.

## Uniques

As mentioned above, a *unique* is some data stored in the world that isn't associated with an entity like a component would be.
They're not technically required in an ECS, but many Rust ECS crates provide something like them as a convenience.
For example, here's how RuggRogue stores the current game seed:

```rust,ignore
pub struct GameSeed(u64);

# let world = World::new();
let game_seed = std::env::args()
    .nth(1)
    .and_then(|arg| arg.as_str().parse().ok())
    .unwrap_or_else(rand::random);

world.add_unique(GameSeed(game_seed)); // <-- adding a unique to the world
```

Since RuggRogue uses a single world to store all game data and passes it everywhere, uniques effectively act like global variables, without a lot of the incidental downsides of actual global variables.

Unique data is accessed by requesting a `UniqueView` or `UniqueViewMut` borrow out of the world with `World::borrow`:

```rust,ignore
// immutable borrow of GameSeed unique
let game_seed = world.borrow::<UniqueView<GameSeed>>();
println!("{:?}", game_seed.0);
```

```rust,ignore
// mutable borrow of GameSeed unique
let game_seed = world.borrow::<UniqueViewMut<GameSeed>>();
game_seed.0 = 1234567890;
```

There is no way to remove or directly replace a unique in Shipyard 0.4.
The ability to remove uniques was only added in Shipyard 0.5, so RuggRogue hacks around this limitation when it needs to.

## Entity and Component Basics

An *entity* is a lightweight ID that's just a number in Shipyard's case.
A *component* is some data associated with an entity.
Each entity can have zero or one component of each type associated with it.

Entities are created with a special borrow of `EntitiesViewMut`, like so:

```rust,ignore
// creating a empty entity with no components
let mut entities = world.borrow::<EntitiesViewMut>();

let entity_id = entities.add_entity((), ());
```

Entities are often made starting out with component data that is modified using `ViewMut`:

```rust,ignore
struct Position {
    x: i32,
    y: i32,
}

struct Renderable {
    ch: char,
}

let mut entities = world.borrow::<EntitiesViewMut>();
let mut positions = world.borrow::<ViewMut<Position>>();
let mut renderables = world.borrow::<ViewMut<Renderable>>();

// creating an entity with a Position component and a Renderable component
let entity_id = entities.add_entity(
    (&mut positions, &mut renderables),
    (
        Position { x: 1, y: 2 },
        Renderable { ch: '@' },
    ),
);
```

Deleting an entity requires clearing it out of every component storage, and thus requires the special `AllStoragesViewMut` borrow:

```rust,ignore
let mut all_storages = world.borrow::<AllStoragesViewMut>();

all_storages.delete(entity_id_to_delete);
```

Components can be added to entities after creation with an immutable `EntitiesView` borrow along with mutable `ViewMut` component borrows of the relevant storages:

```rust,ignore
struct Name(String);

struct GivesExperience(u64);

let entities = world.borrow::<EntitiesView>();
let mut gives_experiences = world.borrow::<ViewMut<GivesExperience>>();
let mut names = world.borrow::<ViewMut<Name>>();

// adding Name and GivesExperience components to goblin_entity_id
entities.add_component(
    (&mut gives_experiences, &mut names),
    (
        GivesExperience(20),
        Name("Goblin".to_string()),
    ),
    goblin_entity_id,
);
```

Components can be deleted from an entity on demand with just a mutable `ViewMut` borrow on the relevant component storage:

```rust,ignore
let mut names = world.borrow::<ViewMut<Name>>();

names.delete(entity_id_to_make_nameless);
```

To check if an entity has a component, we can check if the `View` of the component storage contains the entity ID:

```rust,ignore
struct Monster; // <-- empty tag struct

if world.borrow::<View<Monster>>().contains(entity_id) {
    // entity_id has a Monster component
}
```

A component can be checked for and accessed via a `View` or `ViewMut` as well using Rust's `if let` pattern matching syntax:

```rust,ignore
struct CombatStats {
    hp: i32,
}

let mut combat_stats = world.borrow::<ViewMut>();

if let Ok(combat_stats) = (&mut combat_stats).try_get(entity_id) {
    // entity_id has a CombatStats component, so do a bit of damage to it
    combat_stats.hp -= 1;
}
```

## Iterating Entities and Components

A common operation in RuggRogue is to iterate over all entities that have a certain set of components on them.
That can be achieved with the `iter` function of the `Shipyard::IntoIter` trait:

```rust,ignore
use Shipyard::IntoIter;

struct Name(String);

struct Position {
    x: i32,
    y: i32,
}

let names = world.borrow::<View<Name>>();
let positions = world.borrow::<View<Position>>();

// iterate over all entities that have both Name and Position components
for (name, pos) in (&names, &positions).iter() {
    println!("{} is at ({},{})", name.0, pos.x, pos.y);
}
```

The entity IDs can be retrieved as well using the `with_id` function from `Shipyard::Shiperator`:

```rust,ignore
use Shipyard::IntoIter;
use Shipyard::Shiperator;

for (id, (name, pos)) in (&names, &positions).iter().with_id() {
    // do something with id, name and pos
}
```

I believe `Shipyard::IntoIter` and `Shipyard::Shiperator` are no longer needed in Shipyard 0.5; consult its current documentation if you want to know more.

## The EntityId

Entities are uniquely identified by the `Shipyard::EntityId` type, which, as mentioned before, is just a number internally.
Since it's so lightweight, we can use it to model relationships between different entities.
For example, here's what equipping a player entity with weapon and armor entities might look like:

```rust,ignore
struct Equipment {
    weapon: Option<EntityId>,
    armor: Option<EntityId>,
}

struct AttackBonus(i32);

struct DefenseBonus(i32);

// create the player, weapon and armor entities
let (player_id, weapon_id, armor_id) = {
    let mut entities = world.borrow::<EntitiesViewMut>();
    let mut attack_bonuses = world.borrow::<ViewMut<AttackBonus>>();
    let mut defense_bonuses = world.borrow::<ViewMut<DefenseBonus>>();
    let mut equipments = world.borrow::<ViewMut<Equipment>>();

    // Equipment component for the player
    let player_id = entities.add_entity(
        &mut equipments,
        Equipment {
            weapon: None,
            armor: None,
        },
    );

    // AttackBonus component for the weapon
    let weapon_id = entities.add_entity(&mut attack_bonuses, AttackBonus(2));

    // DefenseBonus component for the armor
    let armor_id = entities.add_entity(&mut defense_bonuses, DefenseBonus(1));

    (player_id, weapon_id, armor_id)
};

// later on...
{
    let mut equipments = world.borrow::<ViewMut<Equipment>>();

    // equip the player if they have an Equipment component
    if let Ok(player_equip) = (&mut equipments).try_get(player_id) {
        // put the weapon and armor in the player's Equipment component
        player_equip.weapon = Some(weapon_id);
        player_equip.armor = Some(armor_id);
    }
}
```

This pretty much covers all of the ways that RuggRogue uses Shipyard to handle its own game data.

<!--
## Why not use Systems?

As mentioned earlier, RuggRogue uses Shipyard for entities and components, but it mostly does *not* use its systems.
From my prior experience of reading the source code of open source roguelikes, and sometimes tinkering with it too, the order and conditions under which logic is supposed to run needs to be precise.
With systems, a lot of synchronizing data is needed to define this precision; for example see how the Rust Roguelike Tutorial uses a ["WantsToAttack" component](https://bfnightly.bracketproductions.com/chapter_7.html#player-attacking-and-killing-things) as an ad-hoc queue to synchronize systems.
(I'd use an actual queue in that case, but that's still extra synchronizing data that's needed.)
Just sticking with functions, branches and loops avoids all of that, and it's what I'm more comfortable with, so it's the approach I chose for RuggRogue.
It seems like I arrived at a similar conclusion as Bob Nystrom in his talk about ECS and roguelikes ([video](https://www.youtube.com/watch?v=JxI3Eu5DPwE)).

However, if you read the source code of RuggRogue for a while, you'll see a fair number of uses of a `World::run` function with a closure:

```rust,ignore
world.run(|names: View<Name>, positions: View<Position>| {
    // Name and Position component storages borrowed in here
});
```

That's equivalent to this:

```rust,ignore
{
    let names = world.borrow::<View<Name>>();
    let positions = world.borrow::<View<Position>>();

    // Name and Position component storages borrowed in here
}
```

`World::run` is also used with functions:

```rust,ignore
fn do_something(names: View<Name>, positions: View<Position>) {
    // Name and Position component storages borrowed in here
}

// somewhere else...
world.run(do_something);
```

So what's `World::run` running here?
The functions and closures being put through `World::run` are *systems*, in Shipyard's terms.
So I lied a bit: RuggRogue does use systems.
What it doesn't use are Shipyard's notion of *workloads*, which are bundles of systems that are designed to be run together.
By avoiding workloads, RuggRogue is able to precisely dictate how and when logic runs without having to manage explicit synchronizing data.

In most of the code samples above, `World::borrow` is preferred over `World::run`.
In the RuggRogue source code in general, older code tended to use a lot of `World::run`, while newer code mostly prefers `World::borrow`.
But in the code we just saw, the code for the `World::run` versions is shorter and more convenient.
So why does RuggRogue prefer `World::borrow` over `World::run`?

Due to Rust's borrowing rules, a system cannot easily be called from within another system.
For instance, the following code will not compile:

```rust,ignore
fn my_subsystem(cs: View<C>) {
    // do stuff from cs...
}

{
    world.run(|bs: View<b>| {
        // do stuff with bs...
        world.run(my_subsystem); // <- Can't borrow 'world' twice!!!
    });
}
```

This could be reworked to compile by borrowing the `C` components *alongside* the `B` components, like so:

```rust,ignore
fn my_subsystem(cs: &View<C>) {
    // do stuff with cs...
}

{
    world.run(|bs: View<B>, cs: View<C>| {
        // do stuff with bs...
        my_subsystem(&cs);
    });
}
```

But it gets clumsier when introducing another system deeper in that wants, say, `D` components:

```rust,ignore
fn my_sub_subsystem(ds: &View<D>) {
    // do stuff with ds...
}

fn my_subsystem(cs: &View<C>, ds: &View<D>) { // <- Getting longer...
    // do stuff with cs...
    my_sub_subsystem(ds);
}

{
    world.run(|bs: View<B>, cs: View<C>, ds: View<D>| { // <- Getting longer...
        // do stuff with bs...
        my_subsystem(&cs, &ds); // <- Getting longer...
    });
}
```

Using `world::borrow` instead of `world::run` allows for fine-grained on-demand component access instead.
Here's the equivalent code the above, but using `world::borrow` instead of `world::run`:

```rust,ignore
fn my_sub_system(world: &World) {
    let ds = world.borrow::<View<D>>();
    // do stuff with ds...
}

fn my_subsystem(world:&World) {
    let cs = world.borrow::<View<C>>();
    // do stuff with cs...
    my_sub_subsystem(world);
}

{
    let bs = world.borrow::<View<B>>();
    // do stuff with bs...
    my_subsystem(world);
}
```

RuggRogue works with many different component types and functions that call each other, so `world:borrow` ends up being much easier to use most of the time.
`world::run` tends to be used with small, self-contained functions that don't call many other functions.
-->

## Conclusion

You should now have a general idea of how RuggRogue stores and accesses its data using Shipyard.
Insofar as Rust ECS crates go, I'm so-so on Shipyard, since it came with a lot of functionality that I never used.
I could use it for future projects, but I can just as easily see myself exploring other options or even cobbling my own data storage to suit my own needs.
