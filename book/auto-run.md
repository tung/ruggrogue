# Auto-Run

Most of the time playing RuggRogue is spent moving around the dungeon.
Instead of repeatedly pressing the movement keys, the player can tell the game to move in a direction for them until something interesting is seen.
This feature is known as *auto-running*.

Auto-run is activated by holding the Shift key while pressing a movement key.
There are three types of auto-run:

1. *Resting in place*: Pressing Shift+Space will wait in place until the player is fully healed.
2. *Straight auto-run*: Pressing Shift+direction in open space or against a wall will move in a straight line until the open space or wall ends.
3. *Corridor auto-run*: Pressing Shift+direction in a corridor will follow that corridor until it branches or opens up.

Once auto-run starts the game will move the player until it's interrupted by:

- the player pressing a key
- a monster appearing in the player's field of view
- the player moving next to an item or the downstairs tile
- the player getting hungrier or losing hit points to starvation

## Auto-Run Data Structures

Auto-running requires the game to perform actions across multiple frames.
This means it needs to remember that auto-run was requested, as well as type of auto-run and desired movement direction.
This means that the state of auto-running needs to be stored in data structures.

Everything that auto-run needs to operate is stored in the `Player` struct in the `src/components.rs` file:

```rust,ignore
pub struct Player {
    pub auto_run: Option<AutoRun>,
}
```

If the value of the `auto_run` field here is `None` it means that auto-run is not happening and the game will wait on the player for input.
On the other hand, a `Some(...)` value here means that auto-run is active, and the data within keeps track of the state of auto-running.

The rest of the data structures relating to auto-run, as well as most of the logic, live in the `src/player.rs` file.
The `AutoRun` struct is what's held in the aforementioned `auto_run` field, and appears as follows:

```rust,ignore
pub struct AutoRun {
    limit: i32,
    dir: (i32, i32),
    run_type: AutoRunType,
}
```

The `limit` field is the maximum number of steps that auto-run can proceed by itself, and exists mostly as a fail-safe against the possibility of any bugs that could cause infinite auto-running.

The `dir` field holds the last direction that the player moved, which determines the direction that auto-running will proceed.
This direction is interpreted as a `(dx, dy)` pair where the two values are one of `-1`, `0` or `1`.

The `run_type` field determines the type of auto-running that should occur; it's set when auto-run is requested and doesn't change.
It holds one of the three values of the `AutoRunType` enum that looks like this:

```rust,ignore
enum AutoRunType {
    RestInPlace,
    Corridor,
    Straight { expect_wall: AutoRunWallSide },
}
```

These enum variants each correspond to the three types of auto-running that were described earlier.

The `AutoRunType::Straight` variant holds extra data that it needs to remember that the other two `AutoRunType` variants don't need.
This `expect_wall` field holds one of the variants of the `AutoRunWallSide` enum that looks like this:

```rust,ignore
enum AutoRunWallSide {
    Neither,
    Left,
    Right,
}
```

Straight auto-run keeps track of walls and open tiles to the left and right of the direction that the player is auto-running.

- `AutoRunWallSide::Neither`: Expect neither left nor right walls, i.e. both sides of the player should be fully open.
- `AutoRunWallSide::Left`: Expect a solid wall on the left and open space on the right.
- `AutoRunWallSide::Right`: Expect a solid wall on the right and open space on the left.

Straight auto-run is stopped if the arrangement of tiles falls into a different category than expected, or doesn't fit into any of these categories.

## Auto-Run Control Flow

Auto-run integrates with the input and control flow logic as follows:

1. The player holds Shift when pressing a movement key, handling the turn as usual but also filling in the `auto_run` field of the `Player` struct to start auto-running.
2. While auto-running, the end of the `DungeonMode::update` function tells the game loop to run on the next frame instead of waiting for an event as it normally would.
3. While auto-running, auto-run checks if it should proceed; if so, it automatically moves the player instead of doing the usual input handling logic.

The Shift key during normal input handling is picked up in the `player::player_input` function in the `src/player.rs` file.
This is handed off as a boolean value to either the `try_move_player` or `wait_player` functions, which fills in the `auto_run` field of the `Player` struct after doing their usual business.
This kicks off the auto-run process.

The `player::player_input` function is called from the `DungeonMode::update` function in the `src/modes/dungeon.rs` file.
Its return value controls when the next update should happen, and looks roughly like this:

```rust,ignore
impl DungeonMode {
    fn update(/* ... */) -> (ModeControl, ModeUpdate) {
        if world.run(player::player_is_alive) {
            //
            // A *lot* of stuff skipped here...
            //

            (
                ModeControl::Stay,
                if world.run(player::player_is_alive) && world.run(player::player_is_auto_running) {
                    ModeUpdate::Update // <-- !!!
                } else {
                    ModeUpdate::WaitForEvent
                },
            )
        } else if player::player_is_dead_input(inputs) {
            // ...
        } else {
            // ...
        }
    }
}
```

In the above code, if the player is alive and auto-running, the `DungeonMode::update` function returns `ModeUpdate::Update` instead of `ModeUpdate::WaitForEvent`.
This causes the main loop further up in the call stack to run the `DungeonMode::update` function again on the next frame, even if the input buffer is empty.

On subsequent updates while auto-run is active, the control flow through the `player::player_input` function looks different.
Here's a rough outline of that function:

```rust,ignore
pub fn player_input(/* ... */) -> PlayerInputResult {
    let player_id = world.borrow::<UniqueView<PlayerId>>();

    inputs.prepare_input();

    if item::is_asleep(world, player_id.0) {
        //
        // sleep input handling...
        //
    } else if world.run(player_is_auto_running) {
        //
        // --> AUTO-RUN LOGIC HERE <--
        //
    } else if let Some(InputEvent::AppQuit) = inputs.get_input() {
        PlayerInputResult::AppQuit
    } else if let Some(InputEvent::Press(keycode)) = inputs.get_input() {
        //
        // normal player input handling here...
        //
    } else {
        PlayerInputResult::NoResult
    }
}
```

In the above code, the process of auto-running overrides normal player input handling.

The first thing that the auto-run logic in the `player::player_input` function does is check for reasons to stop, such as:

- receiving the `AppQuit` input event
- receiving any keyboard input event from the player
- the player stepping onto or next to something interesting (checked by the `player_check_frontier` function)
- the player seeing any monsters (checked by the `player_sees_foes` function)

Auto-run is stopped by the `player::player_stop_auto_run` function, which simply clears the `auto_run` field of the `Player` struct to `None`.

Auto-run logic then decrements the `limit_reached` field of the `AutoRun` struct, and when it hits zero also stops auto-run.

At this point, auto-run logic needs to perform final checks that vary based on the different auto-run types: resting in place, straight auto-run and corridor auto-run.
This is the job of the `auto_run_next_step` function, which works as follows:

- Resting in place returns `Some((0, 0))` if the player can still heal by resting (i.e. below maximum hit points and isn't starving) or `None` otherwise.
- Straight auto-run and corridor auto-run check the tiles around the player and return `Some((dx, dy))` to run in the desired direction or `None` to stop.

In the case that it returns `Some(...)`, the direction value within is unpacked and causes the auto-run logic to call either the `try_move_player` or `wait_player` functions to perform the auto-run step.
Note that these two functions are exactly the same ones called during normal input handling, so auto-run effectively acts like smart automatic input handling.

## Resting in Place

RuggRogue has a regeneration mechanic based on hunger level.
The hunger levels are "Full", "Normal", "Hungry", "Very Hungry", "Famished" and "Starving".
The player will gradually regenerate lost hit points over time as long as the player is "Full", "Normal" or "Hungry".

Sometimes the player may wish to simply pass turns in order to recover hit points.
Instead of passing turns manually, the player can press Shift+Space to *rest in place*.
Resting in place will pass turns automatically until the player can no longer recover (due to maximum hit points or hunger) or is interrupted by a monster.

The choice of the Shift+Space key combination might seem odd, given that the usual key to wait a turn is Period.
However, Shift+Period produces the greater-than sign, which is the ASCII character for the downstairs tile and therefore (one of) the inputs that lets the player move downstairs.
Since Shift+Period is already spoken for, and the Space key is the alternate key for waiting a turn, resting in place is thus triggered by Shift+Space.

In terms of code, resting in place starts when the normal input logic detects the Shift+Space key combination.
The Space key triggers the usual logic for waiting a turn, so the `wait_player` function is called as usual.
The pressing of the Shift key causes the `rest_in_place` argument of that function to be set to `true`.

The first thing that the `wait_player` function does in this case is check for any reason that resting in place should not start.
If any monsters are present in the player's field of view, the player gets a message and no turns are spent waiting.
The game then calls the `hunger::can_regen` function (defined in the `src/hunger.rs` file) to perform hunger-related checks; any hunger-related reason to not rest appears as a message and prevents any waiting from taking place.

Assuming that there is no reason to prevent it, resting in place is started by setting the `auto_run` field of the `Player` struct with a `run_type` of `AutoRunType::RestInPlace`.
Auto-run then takes over as described earlier in the auto-run control flow section.

For each turn that auto-run is about to spend resting in place, the `auto_run_next_step` function consults the result of the `hunger::can_regen` function to determine if it should continue.

The final unique detail of resting in place is that, unlike other forms of auto-run, it ignores the presence of items and downstairs in adjacent tiles.
The `player_check_frontier` function that normally performs this check contains an early return if the `run_type` is `AutoRunType::RestInPlace`.

## Straight Auto-Run

Oftentimes the player will find themselves in a room with no monsters in sight.
By holding Shift while pressing a direction, the player will perform a *straight auto-run*, moving in a single direction until they're blocked or something interesting appears.
This allows the player to quickly cross empty and cleared-out rooms.

Straight auto-run that starts in the open will advance until the player finds themselves adjacent to any walls.
Straight auto-run that starts with a wall to one side of the player will advance until that wall ends.

The logic for auto-running in a direction starts when normal movement code detects that the Shift key is being held down.
As per usual, the move is handled with a call to the `try_move_player` function, but the Shift key sets the `start_run` argument to `true`.

If the player tries to start auto-running but a monster is in their field of view, the move is cancelled with a corresponding message.

After the usual movement logic, if the player indeed moved, the game must decide between engaging straight auto-run or corridor auto-run.
This decision is made by checking the pattern of walls and open tiles around the player.
Corridor wall patterns are checked for first by the `auto_run_corridor_check` function; we'll assume that its check fails for the sake of this section.

With the corridor check out of the way, the game decides if straight auto-run should be engaged with the result of the `auto_run_straight_check` function.
This function wants to check for patterns of walls and open tiles, but the exact tiles to check differ based on the player's movement direction.
The player can move in eight directions, but we don't want to repeat similar code for every direction.
This repetition can be avoided with the help of *rotation*.

### Rotating by Movement Direction

The `auto_run_straight_check` function needs to check the tile straight ahead of the player, as well as the tiles to either side.
To cut down on repeated code it uses logical dx and dy values such that it only needs to check tiles for the player moving right (cardinally) and up-right (diagonally).
Every other direction can be represented by rotating the logical dx and dy values to match the player's movement direction, resulting in real map coordinates.

To perform the correct rotation, the `auto_run_straight_check` function feeds the player's movement direction into another function named `rotate_view`, filling in these variables:

- `real_x_from_x`
- `real_x_from_y`
- `real_y_from_x`
- `real_y_from_y`

With the convention of positive y pointing upwards, these variables are filled in based on the player's movement direction (`dx` and `dy`) as follows:

`dx` | `dy` | `real_x_from_x` | `real_x_from_y` | `real_y_from_x` | `real_y_from_y`
---- | ---- | --------------- | --------------- | --------------- | ---------------
1 | 0 | 1 | 0 | 0 | 1
1 | 1 | 1 | 0 | 0 | 1
0 | 1 | 0 | -1 | 1 | 0
-1 | 1 | 0 | -1 | 1 | 0
-1 | 0 | -1 | 0 | 0 | -1
-1 | -1 | -1 | 0 | 0 | -1
0 | -1 | 0 | 1 | -1 | 0
1 | -1 | 0 | 1 | -1 | 0

If `player_x` and `player_y` represent the player's map coordinates, the following helper closures can then rotate logical dx and dy values to get real map coordinates:

```rust,ignore
let real_x = |dx, dy| player_x + dx * real_x_from_x + dy * real_x_from_y;
let real_y = |dx, dy| player_y + dx * real_y_from_x + dy * real_y_from_y;
```

The `dx` and `dy` values here are the x and y deltas of the tile we want to check relative to the player's position, assuming the player is moving right or up-right.

For example, if the player is moving right or up-right, `real_x_from_x` and `real_y_from_y` are both set to 1 by the `rotate_view` function, leading to results similar to this:

```rust,ignore
let real_x = |dx, dy| player_x + dx * 1 + dy * 0;
let real_y = |dx, dy| player_y + dx * 0 + dy * 1;

// simplified
let real_x = |dx, _| player_x + dx * 1;
let real_y = |_, dy| player_y + dy * 1;
```

Above, changes to `dx` and `dy` match one-to-one to the same changes in real map coordinates.

As another example, suppose the player is moving up or up-left.
This requires a 90-degree counter-clockwise rotation for the `dx` and `dy` values.
The values returned by the `rotate_view` function in this case fill `real_x_from_y` with -1 and `real_y_from_x` with 1, leading to something like this:

```rust,ignore
let real_x = |dx, dy| player_x + dx * 0 + dy * -1;
let real_y = |dx, dy| player_y + dx * 1 + dy * 0;

// simplified
let real_x = |_, dy| player_x + dy * -1;
let real_y = |dx, _| player_y + dx * 1;
```

Above, you'll notice that changes to the logical `dy` value lead to *reversed* changes to real x, e.g. logical upwards steps travel left in reality.
Meanwhile, logical `dx` value changes lead to non-reversed changes to real y, e.g. logical rightward steps travel up in reality.

Savvy readers will notice that conversions of logical `dx` and `dy` values into real map coordinates like this are *affine transformations*, involving rotating according to player movement direction and translation to the player's real map coordinates.

### Checking Walls and Open Space

Armed with the `real_x` and `real_y` helper closures, the `auto_run_straight_check` function can now tie them together into a single helper closure that checks real map coordinates for the presence of walls:

```rust,ignore
let check_wall = |dx, dy| map.wall_or_oob(real_x(dx, dy), real_y(dx, dy));
```

The `Map::wall_or_oob` function defined in the `src/map.rs` file is just a little helper function that returns `true` if the given tile coordinates are a wall or out-of-bounds.

`check_wall` is used like this: If the player is moving cardinally, `check_wall(1, 0)` checks the tile in front of the player, `check_wall(0, 1)` checks the tile to their left and `check_wall(0, -1)` the tile to their right.
For a diagonally moving player, the tile in front is checked using `check_wall(1, 1)`.

This brings us to the whole purpose of the `auto_run_straight_check` function: to look at tiles adjacent to the player according to their movement direction to yield one of these possible return values:

- `Some(AutoRunWallSide::Left)` - A complete wall to the left of the player and completely open tiles on the right.
- `Some(AutoRunWallSide::Right)` - A complete wall to the right of the player and completely open tiles on the left.
- `Some(AutoRunWallSide::Neither)` - Completely open tiles to the left and right of the player.
- `None` - Anything else.

If the first call to `auto_run_straight_check` made in the `try_move_player` function returns one of the `Some(...)` values, that value is stored in the `expect_wall` field of the `AutoRunType::Straight` enum variant.
Each step of straight auto-run calls `auto_run_straight_check` again in the `auto_run_next_step` function and compares the result with the `expect_wall` field, only continuing if they match.
This is the crux of straight auto-run.

For cardinal movement, the exact tiles that need to be checked are marked below, where `@` is the player and `f`/`1`/`2` are the tiles to check:

```plaintext
.11
.@f
.22
```

`f` is the tile in front of the player (`check_wall(1, 0)`) and must be open for straight auto-run to proceed.

The `1` tiles are to the left of the player (`check_wall(0, 1)` and `check_wall(1, 1)`), as if the player is logically moving right.
These tiles must be either both walls (implying `AutoRunWallSide::Left`) or both open (implying `AutoRunWallSide::Right` or `AutoRunWallSide::Neither`).
Mismatching tiles here means that there is a partial wall to the left, so the `auto_run_straight_check` function should return `None` to prevent straight auto-running.

The `2` tiles are to the right of the player (`check_wall(0, -1)` and `check_wall(1, -1)`), and are checked the same way as the tiles to the left.

For diagonal movement, the pattern is different, but otherwise all the checks are the same as for cardinal movement.
Again, `@` is the player, and `f`/`1`/`2` are the tiles to check:

```plaintext
11f
.@2
..2
```

That's it for the `auto_run_straight_check` function.

### Checking for Items and Downstairs

Recall that that the auto-run control flow in the `player::player_input` function needs to stop auto-running if the player finds themselves on top of or next to an item or the downstairs.
This is the job of the `player_check_frontier` function, which returns `true` if either of these things are found or `false` otherwise.

The `player_check_frontier` function does pretty much the same rotation trick as the `auto_run_straight_check` function.
This time the checks are for items and downstairs (or rather, any tile that isn't a wall or floor).

For cardinal movement, the `@` and `!` tiles are checked, where the `!` tiles are newly adjacent:

```plaintext
..!
.@!
..!
```

Likewise, for diagonal movement:

```plaintext
!!!
.@!
..!
```

## Corridor Auto-Run

The rooms of any given dungeon map are connected with corridors that are single tile wide.
If the player holds Shift while moving in a corridor, *corridor auto-run* will engage, automatically moving them along the corridor until it opens up or ends.
This allows the player to quickly move between rooms on the current dungeon level.

In order to implement corridor auto-run, the game must check the tiles near and around the player according to their movement direction, much like straight auto-run.
At each step, corridor auto-run checks for a single open tile in the direction of movement for the player to step into, and walls for other surrounding tiles.
This means corridor auto-run *changes* the player's movement direction at each step; the job of the `auto_run_corridor_check` function is thus to check for a pattern of corridor-like surrounding walls and produce this direction.
The direction change is dealt with in the `auto_run_next_step` function under the handling for `AutoRunType::Corridor`.

The same idea of rotation from straight auto-run carries into the logic for corridor auto-run in the `auto_run_corridor_check` function.
However, corridor auto-run needs to check for many more patterns of walls and open tiles.
To make this easier, the state of each of these tiles is represented by a single bit in an unsigned 16-bit integer variable named `nearby_walls`.

The code near the top of the `auto_run_corridor_check` function populates the bits of the `nearby_walls` variable using a different helper closure to that used by the straight auto-run logic:

```rust,ignore
let check_unknown_or_wall = |dx, dy| {
    !player_fov.get((real_x(dx, dy), real_y(dx, dy)))
        || map.wall_or_oob(real_x(dx, dy), real_y(dx, dy))
};
```

The `check_wall` helper closure has transformed into this new `check_unknown_or_wall` that treats tiles outside the player's field of view like walls for corridor-testing purposes.
This is used to test tiles that are two tiles away from the player, which is needed to test for some of the more unusual corridor wall tile patterns.

### Single-Tile Cases

A lot of the time, corridor auto-run logic is looking for a single tile to advance into that requires no more than a 90-degree turn to the left or right, with walls for all other possible steps.
If the player is moving cardinally, the code exploits rotation so it only has to work as if the player is moving right.
In the ASCII diagrams below, `@` is the player moving right, `#` is a wall and the numbers are the direction of the next step to take:

```plaintext
#1#  .#2  .##  .##  .##
.@#  .@#  .@3  .@#  .@#
.##  .##  .##  .#4  #5#
```

Note the extra wall tile needed for cases 1 and 5 to ensure that the open tile is enclosed by walls and is thus corridor-like.

If the player is moving diagonally, rotation allows the code to treat all diagonals as if the player is moving up-right.
The diagonal cases look like this:

```plaintext
1##  #2#  ##3  ###  ###
#@#  .@#  .@#  .@4  .@#
..#  ..#  ..#  ..#  .#5
```

Each of these cases can be represented as data as follows:

- *Mask bits* that confine our wall-or-open tile pattern matching to only the tiles we care about.
- *Open bits* for any tiles that must be open.
- A *direction* that corridor auto-run should take the player on their next step.

Each of the cases above is stored in a table of mask bits, open bits and directions, with separate tables for cardinal versus diagonal movement.
Each pattern is tested as follows:

1. Obtain a masked version of the nearby tiles by applying the bitwise AND operation to `nearby_walls` and the pattern's mask bits.
2. Separately, obtain the desired pattern by applying the bitwise AND operation to the pattern's mask bits and open bits.
3. If the two values are equal, we have a match, so return with the corresponding direction.

The final wrinkle in all of this is that the dx and dy values of the direction are hard-coded in the table for the player moving right or up-right.
They need to be converted to real map directions, which involves the parameters retrieved from the `rotate_view` function call based on the player's current movement direction.
Thus the returned direction is processed for final consumption like this:

```rust,ignore
return Some((
    move_dx * real_x_from_x + move_dy * real_x_from_y,
    move_dx * real_y_from_x + move_dy * real_y_from_y,
));
```

### Handling Corridor Corners

Most of the time when the player is auto-running in a corridor, there will be one obvious tile to move into as described in the previous section.
However, what happens when the player encounters the corner of a corridor?
Suppose the player is moving to the right and encounters the corner of a corridor:

```plaintext
  #.#
###!#
..@!#
#####
```

There are now *two* adjacent open tiles for the player to step into, marked as `!` above.
The simple single-tile cases fail to recognize that the player is still in a corridor, so we need more patterns to handle this.
What's more, these patterns need to check tiles that are *two* steps away from the player, which is why the `nearby_walls` variable needed them earlier.

If the player is moving cardinally, the corresponding rightward-moving patterns for corridor corners look like this:

```plaintext
##
#66  #7   ###   ##
 @#  @7#  @8#   @#
 ##  ###  #8   #99
               ##
```

The cases for 7 and 8 are the most common, corresponding to a corridor corner taking a 90 degree turn either left or right respectively.
The more exotic cases for 6 and 9 handle little zig-zags that a corridor might choose to take.

If a corner case is matched, corridor auto-run could choose either of the two numbered open tiles to step into.
RuggRogue prefers to step into the corner, but it could just as easily cut the corner with just minor changes to the pattern tables.

If the player is moving diagonally, the corresponding up-right-moving patterns are as follows:

```plaintext
 ##  ##
66#  #77  #8   ###
#@#   @#  @8#  @9#
           ##  #9
```

These cases are rather exotic, and are mostly triggered when the player chooses to start auto-running when stepping diagonally into the entrance of a corridor.

The choice of which tiles to check for corners tries to strike a balance of permissiveness to allow corridor auto-run as often as possible, and strictness to prevent it when it isn't wanted.
Settling on these patterns involved some trial-and-error, so improvements and simplifications to them might exist.

## Interrupting Auto-Run for Hunger

Aside from player input, tile layout, items and monsters, there are two final reasons that the game may interrupt auto-run, both related to hunger:

1. The player drops down a hunger level.
2. The player loses hit points due to starvation.

Both of these cases are handled in the `tick_hunger` function in the `src/hunger.rs` file.

The player's current hunger level is shown in the sidebar as one of these labels: "Full", "Normal", "Hungry", "Very Hungry", "Famished" and "Starving".
If the hunger level drops and the new hunger level is not "Normal" then auto-run is interrupted by setting the `auto_run` field of the `Player` struct to `None`.

If the player's hunger level is "Starving" then they'll periodically lose hit points ("Player aches with hunger!").
Each time this happens auto-run will also be interrupted to prevent the player auto-running themselves into a starvation death.
