# Pathfinding

As the player explores the dungeon in RuggRogue they'll encounter monsters.
When a monster sees the player they will move towards the player to attack them.
The path taken by that monster is determined by *pathfinding*, which makes up the most important part of their AI.
This chapter touches on the algorithm used for pathfinding, then covers the details needed to make it work in the context of the game.

## The A\* Search Algorithm

Getting a monster to approach the player is about finding a path between their positions on the map.
The traditional approach to calculating a path in this context is the *A\* search algorithm* (pronounced "a-star").
Unlike shadow casting approaches in the [Field of View chapter](field-of-view.md), all A\* search implementations have a similar structure.
Instead of painstakingly describing it here, I'll link to Red Blob Games instead:

> [Introduction to the A\* Algorithm at Red Blob Games](https://www.redblobgames.com/pathfinding/a-star/introduction.html)

**The A\* search algorithm is an enhancement to breadth-first search that prioritizes fringe nodes by the sum of the distance travelled plus estimated distance to the destination based on a heuristic function.**
If any part of that sentence did not make sense, you should read the Red Blob Games link first and come back here later.
The rest of this chapter will address implementation details surrounding the use of the A\* search algorithm in RuggRogue and less the algorithm itself.

## Overview of Pathfinding in RuggRogue

Pathfinding in RuggRogue can be broken down into several parts:

- The monster AI that uses pathfinding.
- Deciding which map tiles are blocked and which are walkable for the purposes of pathfinding.
- The front-end pathfinding function and the iterator it prepares and returns.
- The back-end function that forms the core of the pathfinding implementation.

The monster AI lives in the `do_turn_for_one_monster` function in the `src/monster.rs` file.
It calls the `ruggrogue::find_path` function to find a path to the player and takes a single step towards them.

The map informs the pathfinding system about which of its tiles are blocked and which are walkable by implementing the `ruggrogue::PathableMap` trait defined in the `src/lib/path_find.rs` file.
The map defines a single `is_blocked` function for this trait in the `src/map.rs` file to do this, the result of which is based on the type of tile at that position, along with any entities there.

The `ruggrogue::find_path` function is the front-end function for pathfinding, defined in the `src/lib/path_find.rs` file.
It calls the back-end `a_star` function in the same file to calculate the raw path data, then prepares the path it finds into an `AStarIter` struct, which is an iterator describing that path.

## Pathfinding in Monster AI

The `do_turn_for_one_monster` function in the `src/monster.rs` file handles the AI for a single monster turn.
The following code runs when the monster can see the player:

```rust,ignore
if let Some(step) = ruggrogue::find_path(&*map, pos, player_pos, 4, true).nth(1) {
    if step == player_pos {
        damage::melee_attack(world, monster, player_id.0);
    } else {
        let blocks = world.borrow::<View<BlocksTile>>();
        let mut coords = world.borrow::<ViewMut<Coord>>();
        let mut fovs = world.borrow::<ViewMut<FieldOfView>>();

        map.move_entity(monster, pos, step, blocks.contains(monster));
        (&mut coords).get(monster).0 = step.into();
        (&mut fovs).get(monster).dirty = true;
    }
}
```

The call to the `ruggrogue::find_path` function retrieves the path from the monster to the player as an `AStarIter` instance, which is an iterator that returns the position of each step in that path.
Since this handles only single turn, we want the monster to only take the next step towards the player.
The first step of the `AStarIter` iterator is the starting position of the monster, so we need to use `nth(1)` for the next step.
This whole thing is wrapped in an `if let` block, so if the monster cannot find a path to the player it will simply do nothing.

Assuming that the monster finds a path, if that next step is the player's position, the monster performs a melee attack, otherwise it takes a step.
When the monster moves, its position is updated for the map by the `Map::move_entity` function defined in the `src/map.rs` file.
The `Coord` component of the monster entity needs to be similarly updated.
Finally, the monster's field of view needs to be recalculated based on its new position, so its `FieldOfView` component is marked dirty.

## Blocked Map Tiles

RuggRogue uses the A\* search algorithm to find a path from a monster to the player.
This path has two main concerns:

1. Don't walk through walls.
2. Don't walk through other monsters.

The `Map` struct (defined in `src/map.rs`) implements the `ruggrogue::PathableMap` trait (defined in `src/lib/path_find.rs`) to tell the pathfinding code about these two things.
The implementation itself can be found in the lower half of the `src/map.rs` file, and looks like this:

```rust,ignore
impl ruggrogue::PathableMap for Map {
    fn is_blocked(&self, x: i32, y: i32) -> bool {
        matches!(self.get_tile(x, y), &Tile::Wall)
            || self
                .tile_entities
                .get(&(x, y))
                .map_or(false, |(block_count, _)| *block_count > 0)
    }
}
```

The wall check should be self-explanatory, but the part concerning entities warrants some explanation.
The position of all entities placed on the map is stored in their `Coord` component, but it's also redundantly stored in the `tile_entities` field of the `Map` struct, which is the *spatial entity cache*:

```rust,ignore
pub struct Map {
    // ...

    // (x, y) -> (blocking_entity_count, entities_here)
    #[serde(skip)]
    tile_entities: HashMap<(i32, i32), (i32, Vec<EntityId>)>,

    // ...
}
```

The keys of the `tile_entities` hash map are map positions.
The value is made up of two parts: a count and a list of entities present at that map position.
As you might have guessed from the comment in the code above, the count tracks the number of entities in the list that block pathfinding.
All entities that block pathfinding are marked with a `BlocksTile` component, and every monster is spawned with this component in the `spawn_monster` function in the `src/spawn.rs` file.

The maintenance of the spatial entity cache is managed by three functions associated with the `Map` struct:

- `Map::place_entity`
- `Map::remove_entity`
- `Map::move_entity`

These functions are called in numerous places for all entities, such as when starting or loading a game, player movement, and picking up and dropping of items.
They are especially important when called for monsters in order to update the blocking entity count of tile positions in the spatial entity cache when they move or are spawned or despawned.

Based on all of this, the `is_blocked` function from earlier can therefore determine if a tile contains a blocking entity by simply checking the spatial entity cache for a non-zero blocking entity count, saving it from having to scan the whole entity list for entities with a `BlocksTile` component.

## The `ruggrogue::find_path` Function and `AStarIter` Struct

When the monster AI wants to find a path to the player, it requests it by calling the `ruggrogue::find_path` function defined in the `src/lib/path_find.rs` file.
This function returns its result as an instance of the `AStarIter` struct, which is an iterator that yields each step in the path that it finds from the starting position to the destination.

The first thing the `ruggrogue::find_path` function does is prepare a hash map for the back-end `a_star` function to fill in with raw pathfinding data:

```rust,ignore
let mut came_from: HashMap<(i32, i32), (i32, i32)> = HashMap::new();
```

The key of this hash map is a tile position, while the value is the position of the pathfinding step taken to get to this tile.
Following this linkage of steps from any keyed position will eventually lead back to the starting position.
In other words, the path data that will be stored in here will be backwards; that will be dealt with later.

The `ruggrogue::find_path` function calls the `a_star` function defined above it to perform the pathfinding itself:

```rust,ignore
let closest = a_star(map, start, dest, bound_pad, &mut came_from);
```

The first thing to note is the `bound_pad` argument.
In order to avoid excessive path exploration when no path exists between a monster and the player, pathfinding in RuggRogue is not typically performed over the entire map.
Instead, pathfinding is bounded by a rectangle of tiles that includes the monster and player positions as corners.
If the `bound_pad` argument is non-zero, this rectangle is expanded to include `bound_pad`-worth of extra tiles on all sides; a zero `bound_pad` causes pathfinding to explore the whole map if needed.

The call to the `a_star` function populates the `came_from` hash map, but it also returns the position of the tile closest to the destination out of all the tiles that it explored.
If a path is found to the destination, this `closest` tile will be the destination itself.
If there is no such path, the caller of `ruggrogue::find_path` can opt into receiving a path to this closest tile instead by setting the `fallback_closest` argument to true; monster AI uses this to allow multiple monsters to pursue the player down a single-tile-wide corridor for example.

As mentioned earlier, the raw path data in the `came_from` hash map has each tile point *backwards* to the tile it was reached via and is thus backwards from how the caller of `ruggrogue::find_path` needs it.
The links of the path need to be reversed so that each tile on the path points forwards and not backwards.
The code looks like this:

```rust,ignore
// Reverse the path from closest to start.
let mut current = came_from.get(&closest).copied();
let mut prev = closest;

came_from.remove(&closest);

while let Some(c) = current {
    let next = came_from.get(&c).copied();

    came_from.insert(c, prev);
    prev = c;
    current = next;
}
```

This is the same kind of logic used to reverse a linked list.

If you're clever, you might be wondering why the `ruggrogue::find_path` function doesn't just reverse the start and destination positions to avoid having to manually reverse path data.
Doing that would only save work if a path is definitely found between a monster and the player.
If no path exists, the closest tile position is useless since it's only reachable from the destination and not the starting position; a second pathfinding run starting from the starting position would be needed anyway in that case.

## The `a_star` Function

The `a_star` function lives just above the `ruggrogue::find_path` function in the `src/lib/path_find.rs` file; this is where the A\* search algorithm is implemented in the game code.
Its main job is to populate the `came_from` hash map passed into it with path data that hopefully connects the start and destination positions; to do this it needs some supporting data:

```rust,ignore
// (priority, (x, y))
let mut frontier: BinaryHeap<(Reverse<i32>, (i32, i32))> = BinaryHeap::new();
// ((x, y), cost)
let mut cost_so_far: HashMap<(i32, i32), i32> = HashMap::new();
```

The `frontier` holds the set of tiles that the A\* search algorithm will want to explore next; this is sometimes referred to as the *open set*.
To minimize the number of explored tiles, tiles need to be popped out of the frontier based on how long the algorithm *thinks* the final path will be if it were to go through those tiles.
Tiles therefore need to be popped out of the frontier in priority order: the `BinaryHeap` collection from Rust's standard library is perfect for this purpose.
The data held in the `frontier` binary heap is a priority paired with a tile position.
Rust's `BinaryHeap` type pops values out with highest numeric priority first, but we want the *shortest* path, not the longest, hence the `Reverse<i32>` type that reverses comparison order of numbers held within it.

There's one more piece of data that needs to be remembered for each tile explored by the path finding algorithm: the path cost from the starting position to that tile.
This is held in the `cost_so_far` hash map whose keys are tile positions and values are the distance from the starting position.

As part of the A\* search algorithm, the priority of a tile in the frontier is the sum of the distance so far and the estimated distance to the destination.
This estimate is calculated using a *heuristic function*; the one used by the `a_star` function looks like this:

```rust,ignore
let dist100 = |(x1, y1), (x2, y2)| {
    let x_diff = if x1 < x2 { x2 - x1 } else { x1 - x2 };
    let y_diff = if y1 < y2 { y2 - y1 } else { y1 - y2 };
    let (low_diff, high_diff) = if x_diff < y_diff {
        (x_diff, y_diff)
    } else {
        (y_diff, x_diff)
    };

    // Prefer axis-aligning with (x2, y2).
    low_diff * 141 + (high_diff - low_diff) * 99
};
```

This function calculates the approximate distance between two points, times 100 to avoid having to convert between integers and floating point values.
This estimates the path cost where every diagonal step costs 141 (i.e. the square root of 2, multiplied by 100) and every cardinal step thereafter costs 100...

Wait, why is there a multiplication by "99" and not "100"?

To understand why this is, we need to go all the way back to monster AI.
A monster will only pursue the player if they can see the player; if the monster loses sight of the player it will no longer give chase.
If the player retreats down a corridor, a monster's best chance of keeping the player in its sights involves lining up with the player directly on either the horizontal or vertical axes.
In other words, a monster wants to maximize the number of cardinal moves left in its path to the player while minimizing the number of diagonal moves left.
By making cardinal moves cost 99 instead of 100 in the heuristic function, the monster's path will "hoard" cardinal steps by taking diagonal steps as early as possible.
Take a moment to think through why this works: it definitely took me some time to wrap my head around at first.
Just remember that `dist100` is only a heuristic function; it doesn't actually affect the real path cost, just the priority of tiles explored in the frontier.

There's another oddity about this heuristic function that, unlike the quirk above, is also reflected in the real path cost.
Diagonal steps have an extra cost compared to cardinal moves in all of this pathfinding code, but steps in all eight directions cost a single turn during actual gameplay; why the discrepancy?
Using Euclidean distance for pathfinding like this leads to paths that look more like what a human would choose.
Using the exact distance calculations used by the gameplay instead leads to many intermediate frontier tiles with equal priority values, and the tie-breaking involved often leads to technically correct shortest paths that look ugly or bizarre.

The big `while` loop in the `a_star` function performs the main part of the A\* search algorithm: pop a tile from the frontier, terminate if it's the destination and add surrounding tiles to the frontier based on path cost priority.
However, there's a little bit of extra tracking data:

```rust,ignore
let mut closest = start;
let mut closest_cost = 0;
let mut closest_dist = dist100(start, dest);
```

Each tile popped from the frontier is additionally checked to see if it's the closest to the destination.
The big `while` loop updates this so that there's a fallback destination to take a path towards if the real destination cannot be reached.

## Conclusion

The pathfinding code in RuggRogue was written fairly early in its life cycle, so it does things a bit strangely compared to how I would author the code nowadays.

Astute readers may notice that the code calculates the whole path for a monster, takes just a single step and recalculates the path again on its next turn.
This is less wasteful than it seems: the book-keeping data for the A\* search algorithm has to be allocated anyway even for a single step, so discarding it immediately doesn't differ much from creating an iterator, taking a single step and throwing away the iterator.

The tweak to the heuristic function to get monsters to line up with the player to chase them down corridors works pretty well.
It's still possible to juke monsters by leaving their fields of view, but it makes it more likely that monsters in a room that see a player in a corridor will chase the player down that corridor.
