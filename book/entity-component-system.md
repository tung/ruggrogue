# Entity Component System

Up until now, most of the data that has been covered has been about technical things that the game needs just to run, like input events, surfaces, textures and timing information.
But beyond that is the data that defines RuggRogue as a game, such as the player, the map, monsters and items.
This game-specific data is all managed by a crate named [Shipyard](https://crates.io/crates/shipyard), and this chapter is all about Shipyard and how RuggRogue uses it.

By its own description:

> Shipyard is an Entity Component System focused on usability and speed.

Here, an *entity* is a lightweight ID whose role is to associate groups of *components* that hold the data describing the entity.
The main benefit of this is that it avoids the "talking sword" problem that you'd run into with an object-oriented approach: if you NPCs that you can talk to, and a sword you can pick up and swing, how do you represent a talking sword?
In the OO-style of modelling game data, problems like this end up poking holes in the encapsulation the classes are supposed to have, and functionality drifts up the inheritance tree into a gigantic all-encompassing mega-class.
Game data modelled with entities and components instead avoids both of those issues; see Catherine West's RustConf 2018 closing keynote ([video](https://www.youtube.com/watch?v=aKLntZcp27M) and [notes](https://kyren.github.io/2018/09/14/rustconf-talk.html)) for more information.

In a game built fully in the ECS-style, *systems* are just functions that manipulate groups of entities according to what components they have.
However, RuggRogue mostly does *not* use Shipyard's systems, for reasons that will be discussed later.

## Shipyard 0.4

RuggRogue uses Shipyard **0.4**, but at the time of writing it is *not* the most recent version of Shipyard, which is **0.5**.
So what gives?
Well, 0.4 was the most up-to-date version of Shipyard when I started work on RuggRogue, and when 0.5 came out I ported the game over to it.
Unfortunately, [this broke the web build](https://github.com/tung/ruggrogue/commit/76454e69aa5734d98bda91869bdcec75f8152732), so it had to be reverted.
Therefore, RuggRogue uses Shipyard 0.4 and not 0.5.

In order to understand how RuggRogue reads and modifies its own game data, you'll need to understand the basics of Shipyard 0.4.
This is the point where I would link to the Shipyard 0.4 User's Guide that existed when I started writing the game, except it was replaced wholesale when Shipyard 0.5 came out, which has a bunch of differences.
I could build and host that old guide myself, but putting up documentation for an older version of somebody else's library with no indication that it's stale causes problems.
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

## Why not use Systems?

As mentioned earlier, RuggRogue uses Shipyard for entities and components, but it mostly does *not* use its systems.
From my prior experience of reading the source code of open source roguelikes, and sometimes tinkering with it too, the order and conditions under which logic is supposed to run needs to be precise.
With systems, a lot of synchronizing data is needed to define this precision; for example see how the Rust Roguelike Tutorial uses a ["WantsToAttack" component](http://bfnightly.bracketproductions.com/chapter_7.html#player-attacking-and-killing-things) as an ad-hoc queue to synchronize systems.
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
But in the code we just saw, the code for the `World::run` versions are shorter and more convenient.
So why does RuggRogue prefer `World::borrow` over `World::run`?
In order to understand the answer to this question, we'll need to touch on three things:

1. Rust's stance on mutable and immutable borrowing, and why it matters.
2. How Shipyard loans out borrows of its component storages.
3. How the nested logic of RuggRogue interacts with Rust's borrows and Shipyard's component storages.

## Mutable and Immutable Borrowing in Rust

I recommend reading the [Understanding Ownership](https://doc.rust-lang.org/stable/book/ch04-00-understanding-ownership.html) chapter of the Rust Book, particularly the part on references and borrowing.
However, I also understand that it's bit unreasonable to make people read a chapter of another book to understand the contents of this book.
So, for those of you already familiar with Rust's borrowing rules, feel free to skip to the end of this section for a discussion of why this matters in the context of RuggRogue.
For those of you who aren't, strap yourselves in for a crash course on ownership and borrowing in Rust.

Consider the following code where we create a vector and print it out:

```rust
# fn main() {
    let my_vector = vec![1, 2, 3];

    for e in my_vector.iter() {
        println!("{}", e);
    }
# }
```

If we pay attention to the first line, there's two things that have been created here:

1. A *value* that lives in memory; in this case a vector containing three numbers.
2. An *owner* of that value; in this case the `my_vector` variable.

Every value in Rust has exactly one owner, no more, no less.
When the owner of a value goes out of scope, that value is freed from memory.
The time when a value is first given memory to the time that it's freed is called the value's *lifetime*, which we won't go too much into here.

The `my_vector` variable as declared above cannot change (or mutate) its value, as it lacks the `mut` keyword as part of its declaration.
We call this an *immutable owner*.
An immutable owner can do the following with the value that it owns:

1. read the value
2. loan out *immutable references* (or borrows)
3. give up its ownership to someone else
4. drop the value, freeing it from memory

We care about the "reference" bit here.
An *immutable reference* is a way for an owner to grant the ability to read the value to other variables.
(Owners can be more than variables, like fields of structs or elements of a collection such as a vector or hash map, but we'll stick to variables here for simplicity.)
Here's an example of an immutable reference:

```rust
# fn main() {
    let my_vector = vec![1, 2, 3];
    let my_immutable_reference = &my_vector;

    for e in my_immutable_reference.iter() {
        println!("{}", e);
    }
# }
```

Here, `my_vector`, the owner, is creating an immutable reference with the ampersand (`&`) operator and giving it to the `my_immutable_reference` variable, who is then able to read the vector.

If we want to modify this vector, we instead need a *mutable owner*.
The following code creates a mutable owner of a vector using the `mut` keyword so it can be modified (or mutated) and printed:

```rust
# fn main() {
    let mut my_mutable_vector = vec![1, 2, 3];

    // modify the values in the vector
    for e in my_mutable_vector.iter_mut() {
        *e += 1;
    }

    // print "2", "3" and "4" on separate lines
    for e in my_mutable_vector.iter() {
        println!("{}", e);
    }
# }
```

A mutable owner does everything an immutable owner can do, plus the following:

1. modify its owned value
2. loan out *mutable references*

A mutable reference can read and modify the value that it refers to.

There is a special limitation that is enforced by Rust when handing out mutable and immutable references; I'll quote the [References and Borrowing](https://doc.rust-lang.org/stable/book/ch04-02-references-and-borrowing.html#the-rules-of-references) chapter of the Rust Book here:

> **At any given time, you can have *either* one mutable reference *or* any number of immutable references.**

This rule is enforced by Rust, even across function boundaries.
When code uses references as above and avoids allocating and freeing memory in loops, the *borrow checker* in the Rust compiler enforces this rule at compile-time.
This gives it a reputation for being rather fussy and needlessly complicated to work with.
So why would we want to work with it?

The reason is that Rust's borrow checker helps to protect *data integrity*.
For example, you don't want the player to have more hit points than their maximum or to spawn out of bounds.
The phenomenon of *screen tearing* when a monitor displays a video is a useful analogy here.
When a monitor lights physical pixels in its hardware, it needs to read memory that tells it what color each pixel should be.
When the computer wants a certain pattern of pixels to be displayed, it needs to write to the memory that the monitor will read from.
Screen tearing happens when the monitor reads this memory before the computer has finished writing to it.
What appears on the physical monitor is that part of the screen will show up-to-date pixels, a visible seam and the rest of the screen will show stale pixels.
Solving screen tearing involves controlling when this chunk of memory is written to and when it is read from.

Screen tearing is an example of what Rust calls a *data race*; the monitor hardware will read different data from memory depending on how fast or slow it is relative to the computer writing to that memory.
It turns out that data races can occur with *any* memory whenever there's more than one bit of code reading from or writing to the same chunk of memory.
When a data race happens with video memory, you get screen tearing.
When a data race happens with game data, you get silent data corruption; imagine a case where there's an outer loop going through some game data, and an inner loop that does something the outer loop isn't prepared to handle.
Silent data corruption is a class of errors that are some of the hardest and most time-consuming to track down and fix.

Rust's borrow checker eliminates data races, and thus silent data corruption, by enforcing either one writer and no readers, or multiple readers and no writers.
Working with it takes practice and patience, but I believe the upside of avoiding data races in RuggRogue's game data outweighs the downsides of taking the time and effort to satisfy the system.

## Borrowing Shipyard Component Storages

So we now understand Rust's borrowing rules, and hopefully I've made a case for Rust's borrow checker being a net positive for protecting the integrity of game data.
But what does any of this have to do with Shipyard?

Shipyard is a Rust crate and thus its usage is also subject to Rust's borrowing rules.
However, remember when I said this earlier (with new emphasis)?

> When code uses references as above and **avoids allocating and freeing memory in loops**, the *borrow checker* in the Rust compiler enforces this rule at compile-time.

Most RuggRogue's code exists in the context of a *game loop*, and game data has to be allocated and freed within Shipyard while that game loop is running, often in response to player actions.
Therefore, Shipyard cannot use Rust's compile-time borrow checking to protect our game data from data races.
However, Shipyard is still a Rust crate, so it is still subject to Rust's borrowing rules somehow.
But how?

I haven't read Shipyard's code too closely, but based on my observations, Shipyard is using Rust's [`RefCell`](https://doc.rust-lang.org/std/cell/struct.RefCell.html) type to enforce Rust's borrowing rules.
A `RefCell` takes whatever data it stores and changes the enforcement of Rust's borrow checking from compile-time to runtime.
If the `RefCell` detects a violation of Rust's borrowing rules, it will cause the code to panic instead of failing to compile.

I'd make a safe bet that Shipyard is using a `RefCell` to manage each distinct type of component and each distinct type of unique stored within it.
It maps out as follows:

- `View` and `UniqueView` are immutable borrows on a component storage and a unique respectively.
- `ViewMut` and `UniqueViewMut` are the mutable versions of the same things.

If we consider Rust's borrowing rules with respect to Shipyard, taking one or more `View<SomeType>` borrows and then a `ViewMut<SomeType>` would trigger a runtime panic.
Similarly, taking a `ViewMut<SomeType>` and then trying to take a `View<SomeType>` would also trigger a runtime panic.
We have to run the code to check that it follows Rust's borrow checking rules instead of being told straight away, so it's not quite as good as the compile-time borrow checker, but it's still the same idea.

It's worth noting here that how borrowing works in Shipyard may not necessarily match how borrowing works in other Rust ECS crates.
Shipyard stores component data in what it calls *sparse sets*, which I won't go into here, but it means that each distinct component type has its own storage memory.
Other Rust ECS crates may store their component data in other ways, such as by grouping entities by *archetype*, so their component data storage and borrowing patterns would be very different from what's being described here.
If you're exploring Rust ECS crates, the way that components are stored will have the biggest impact on how borrowing happens and therefore how your code will be structured, so pay special attention to it.

## Challenges of Borrowing with Nested Logic in RuggRogue

Okay, so we've established that Rust's borrowing rules are being enforced at runtime for each component and unique type in Shipyard using its `View`, `ViewMut`, `UniqueView` and `UniqueViewMut` types.
We finally know everything we need to know to understand why RuggRogue prefers using `World::borrow` over `World::run`.
To refresh your memory, here's how `World::borrow` is used:

```rust,ignore
{
    let xs = world.borrow::<View<X>>();
    let ys = world.borrow::<View<Y>>();

    // "X" and "Y" component storages borrowed in here.
}
```

Here's how `World::run` runs a closure as a system:

```rust,ignore
world.run(|xs: View<X>, ys: View<Y>| {
    // "X" and "Y" component storages borrowed in here.
});
```

Here's `World::run` again, this time using a function as a system:

```rust,ignore
fn do_something(xs: View<X>, ys: View<Y>) {
    // "X" and "Y" component storages borrowed in here.
}

// somewhere else...
world.run(do_something);
```

In the above code samples, it seems that using `World::run` results in shorter and simpler code than `World::borrow`, so why prefer `World::borrow`?
The answer is that `World::run` results in a number of issues due to RuggRogue's extensive use of helper functions, resulting in nested logic.
When I talk about nested logic, I'm talking about things like this:

```rust,ignore
fn little_helper() {
    // Do some little helper task.
}

fn big_task() {
    // Do some stuff...

    // Somewhere deep inside the function.
    little_helper();

    // Do some more stuff...
}

// Somewhere else in the code...
big_task();
```

In the above code, calling the `big_task` function will result in a nested call to the `little_helper` function at some point during its execution.
If we wanted these to be systems in the Shipyard sense, maybe borrowing some components, maybe they'd look like this:

```rust,ignore
fn little_helper(ys: View<Y>, zs: View<Z>) {
    // Do stuff with Y and Z components.
}

fn big_task(xs: View<X>) {
    // Stuff...

    world.run(little_helper); // <-- ERROR

    // More stuff...
}

// Somewhere else...
world.run(big_task);
```

Rust cannot compile this code; the error here will be something like: "cannot find value \`world\` in this scope".
The `world` doesn't pass itself to the `big_task` function when running it as a system.
**This is the first problem with systems and nested logic: we can't run `World::run` within `World::run`.**

Okay, so if we want to run the logic in `little_helper`, we'd have to call it like a normal function, maybe like this:

```rust,ignore
fn little_helper(ys: View<Y>, zs: View<Z>) {
    // Do stuff with Y and Z components.
}

fn big_task(xs: View<X>) {
    // Stuff...

    little_helper(/* Uh, what do we put here? */); // <-- ERROR

    // More stuff...
}

// Somewhere else...
world.run(big_task);
```

This won't compile either, since `little_helper` wants `ys` and `zs` to be filled in.
We'll have to request borrows on them in `big_task`, like so:

```rust,ignore
fn little_helper(ys: &View<Y>, zs: &View<Z>) {
    // Do stuff with Y and Z components.
}

fn big_task(xs: View<X>, ys: View<Y>, zs: View<Z>) {
    // Stuff...

    little_helper(&ys, &zs);

    // More stuff...
}

// Somewhere else...
world.run(big_task);
```

Okay, now it compiles, but we had to make the `little_task` function take immutable references, so it's no longer really a Shipyard system, but we can at least still use it as a helper function.
The bigger problem is that we had to put all of the requested components of `little_helper` into the function signature of `big_task`.
It's not so bad in these examples, but imagine if `little_helper` needed, say, a dozen components: the signature of `big_task` would get pretty ugly.
When RuggRogue needs to spawn monsters and items, it definitely needs more than a dozen components!
**This is the next problem with systems and nested logic: systems need to specify all of their transitive dependencies.**

In real game code, we wouldn't just be reading component data; we'd need to modify it too.
Suppose that `little_helper` needed to modify the `Z` component; we'd want a `ViewMut<Z>` instead of `View<Z>`:

```rust,ignore
fn little_helper(ys: &View<Y>, zs: &ViewMut<Z>) {
    // Do stuff with Y and Z components.
}

fn big_task(xs: View<X>, ys: View<Y>, zs: View<Z>) {
    // Stuff...

    little_helper(&ys, &zs); // <-- ERROR

    // More stuff...
}

// Somewhere else...
world.run(big_task);
```

The above code won't compile, since we're trying to pass a `&View<Z>` in, but `little_helper` needs a `&ViewMut<Z>`.
We could bubble up the requirement into the signature of `big_task`:

```rust,ignore
fn little_helper(ys: &View<Y>, mut zs: &ViewMut<Z>) {
    // Do stuff with Y and Z components.
}

fn big_task(xs: View<X>, ys: View<Y>, zs: ViewMut<Z>) {
    // Stuff...

    little_helper(&ys, &zs);

    // More stuff...
}

// Somewhere else...
world.run(big_task);
```

This compiles and technically works, but look at what just happened here: `ViewMut` just infected `big_task`, even though only `little_task` needed its mutability.
If you pay close attention to Rust's borrowing rules, you'll realize that immutable references can be freely shared, but mutable references have a lot of limits on what they can coexist with.
**This is the final problem with systems and nested logic: mutability becomes infectious, which limits how references can be used.**

There's a common theme in all of these problems: *our component borrows are too coarse-grained*.
That is, our `View`s and `ViewMut`s are being claimed right at the start of each function, and only released right at the end.
If we use `World::borrow` instead of `World::run` we can gain fine-grained control over borrows.
The above code example then looks like this:

```rust,ignore
fn little_helper(world: &World) {
    let ys = world.borrow::<View<Y>>();
    let mut zs = world.borrow::<ViewMut<Z>>();

    // Do stuff with Y and Z components.
}

fn big_task(world: &World) {
    let xs = world.borrow::<View<X>>();

    // Stuff...

    little_helper(world);

    // More stuff...
}

// Somewhere else...
big_task(world);
```

In the code above, we pass around `world` explicitly, and claim borrows on component storages that we have fine-grained control over using `World::borrow`.
There is one big advantage to using `World::borrow` like this instead of `World::run`: **every function signature is the same**.
By taking the borrows out of the function signatures, the functions can freely call each other.

Well, mostly freely; we need to respect Rust's borrowing rules, otherwise we'll end up with problems like this:

```rust,ignore
fn little_helper(world: &World) {
    let xs = world.borrow::<View<X>>(); // <-- PANIC
}

fn big_task(world: &World) {
    let mut xs = world.borrow::<ViewMut<X>>();

    // Stuff...

    little_helper(world);

    // More stuff...
}

// Somewhere else...
big_task(world);
```

In the above code, `big_task` will claim a mutable reference to the storage of the `X` components, but `little_helper` later tries to claim an immutable reference on it, causing a runtime panic.
RuggRogue mitigates this issue by using `World::borrow` to claim its references as late as possible, and as short a time as possible.
This level of fine-grained control over borrowing is not possible when using `World::run`.

## Conclusion

You should now have a good general idea of how RuggRogue stores and accesses its data using Shipyard.
Hopefully you'll also have an idea of how RuggRogue works with Shipyard to operate within Rust's borrowing rules while still getting the data that it needs.

Insofar as Rust ECS crates go, I'm so-so on Shipyard, since it came with a lot of functionality I ended up just not using.
I could use it for future projects, but I can just as easily see myself exploring other options or even cobbling my own data storage to suit my own needs.
