# Dependencies

RuggRogue doesn't use any roguelike helper libraries, but that doesn't mean it was made from scratch.
In order to complete the game in a reasonable time frame, I had to make use of some external tools and libraries.

## The Language: Rust

A lot has been said about the benefits of Rust as a programming language, so I'll stick to general points on how it relates to RuggRogue.
When I was starting out, there were two things I wanted out of whatever I was going to build the game out of: correctness and performance, and I was willing to take the extra time to make them happen.
In those ways, Rust was a perfect fit for the project.

On correctness, Rust's strong type system provided a robust foundation for structuring the game.
It also allowed for bold code improvements that I would never have attempted without it; code that would otherwise needed whole rewrites or been left in a sub-par state.
My attitude towards bugs is to catch and eliminate them early, and Rust's type and safety checks detect most low-level bugs pretty much as early as possible.

As for performance, I dislike any software that uses more CPU or memory than it needs to do its job.
There's lots of software like that nowadays everywhere due to developers working under time pressure, but it still feels disrespectful to waste the time and resources of so many users to save some time for a few developers.
But thanks to Rust, RuggRogue doesn't have to join their ranks.
It still takes time and effort to improve performance, but the result is a game that doesn't feel awful to have open.
I don't know if anybody else cares, even most of the players, but that's very satisfying to me.

Aside from correctness and performance, Rust's tooling and standard library served the creation of RuggRogue very well.

## The Libraries

Rust refers to libraries as *crates*, so if I use the word "crate" anywhere, it's safe to mentally substitute it with "library".
RuggRogue uses the following crates to do handle various things it doesn't already handle itself:

### bitflags

[bitflags](https://crates.io/crates/bitflags) enables the creation of compact bitmask values with symbolic names.
RuggRogue uses it to encode the state of the Shift, Ctrl and Alt modifier keys in a single value that the game logic can check later.

### bitvec

[bitvec](https://crates.io/crates/bitvec) provides a memory-dense representation of what would otherwise be a vector of booleans that would be each be a byte and thus be eight times larger in memory.
Reducing memory usage improves cache utilization, which makes the game faster in general.
RuggRogue uses bitvecs to keep track of which map tiles the player has seen on the current dungeon level, as well as the tiles within each entity's field of view.

### rand, rand\_xoshiro

[rand](https://crates.io/crates/rand) provides convenient APIs for extracting and using numbers from a backing random number generator.
[rand\_xoshiro](https://crates.io/crates/rand_xoshiro) is one such backing whose implementation is simple, fast and high quality for non-cryptographic needs, like games.
RuggRogue uses these crates to generate random numbers for level generation, item and monster spawning and combat calculations.

### sdl2

[sdl2](https://crates.io/crates/sdl2) or "Rust-SDL2" as the crate refers to itself provides access to [SDL](https://libsdl.org/).
SDL itself is a library that provides access to windows, input events and display in a cross-platform manner.
RuggRogue enables the `image` feature to load PNG files for tiles and ASCII symbols.

SDL is the only non-Rust external dependency of RuggRogue, which has interesting implications.
By choosing SDL instead of pure Rust alternatives, RuggRogue is able to avoid having to compile literally dozens of additional dependent crates, which drastically saves on initial compile times and final binary size.
On top of that, it means that unoptimized debug builds of RuggRogue run almost as fast as optimized release builds; for reference, the performance difference between debug and release builds of the pure Rust approach can be as high as 5x to 10x!

There is one big downside to using a non-Rust dependency in a Rust project, which is that it forces other developers who want to build the game to install SDL themselves; a task that requires some specialized platform-specific knowledge.
It's easiest on Linux, which is what I developed RuggRogue on: a package manager installs SDL2 and SDL2\_image in a standard location, Rust knows how to look in that standard location, and everything is flowers and sunshine.
It's hardest on Windows, which is used by almost 90% of people with a computer, since there's no standard location for development packages, so tools have no idea how to cooperate without messing with paths and deciphering cryptic error messages when you inevitably screw it up.

### serde, serde\_json

[serde](https://crates.io/crates/serde) provides plumbing and infrastructure to enable serialization and deserialization of data structures.
[serde\_json](https://crates.io/crates/serde_json) uses that plumbing to convert data to and from the JSON text-based data format.
RuggRogue uses these crates to convert its data structures into JSON when saving the game to a file, and convert them back out when loading a saved game from a file.

### shipyard

[shipyard](https://crates.io/crates/shipyard) is an Entity Component System (or "ECS") crate that provides:

1. data storage in the form of entities with data components attached,
2. systems that are functions that run on subsets of entities based on which components they have, and
3. workloads that are bundles of ordered systems that are to be executed repeatedly.

However, RuggRogue only uses the entity-and-component data storage of Shipyard, and mostly uses conventional functions, reaching for systems only when convenient and avoiding workloads entirely.
This avoids having lots of message queues to do cross-system communication, and thus a lot of red tape, since systems can't directly call other systems in the classic ECS arrangement.
On the other hand, I have to carefully handle every function call, every branch and every loop to make sure everything runs at exactly the right time, and the right number of times, which the flat and linear model of system-based workloads sidesteps entirely.
My "EC-only" approach isn't necessarily better than the full ECS approach, but it makes it very different to what it otherwise would have been.

### wyhash

[wyhash](https://crates.io/crates/wyhash) is a hashing crate; it ingests some data and calculates a hash value for that data.
Remember rand and rand\_xoshiro?
There's more to the random number story in RuggRogue.
RuggRogue uses wyhash to create seeds for temporary random number generators that it uses.

## The Web Support: Emscripten

The way that RuggRogue runs on the web is by telling Cargo (Rust's main build tool) to build for the `wasm32-unknown-emscripten` target.
If we ignore the `unknown`, `wasm32` is the target architecture (this would be something like `x86_64` for native), while `emscripten` is the target OS (that's `linux` if I build the game natively for myself).
`wasm32` is the 32-bit flavor of [WebAssembly](https://webassembly.org/), which is a machine-code-like binary format that web browsers can run in a sandbox as an alternative to JavaScript.
But WebAssembly can only muck about with memory and numbers; it has to call *host functions* to do interesting things, e.g. JavaScript functions in a web browser.

This is where [Emscripten](https://emscripten.org/) comes in.
Emscripten provides a whole bunch of host functions that make a WebAssembly blob believe it's running in a classic desktop-like environment.
For example, Emscripten provides POSIX-like file system APIs that enable the same file system code to compile and run unmodified in a web browser as it does natively.
Critically for RuggRogue, Emscripten implements the SDL API, so the windowing, input event handling and rendering all work in a web browser with minimal changes.
When Emscripten works, it's like magic.

But Emscripten's magic is imperfect.
A part of it is differences imposed by the browser environment that Emscripten operates in.
In a native application, processes automatically share access to the CPU due to pre-emptive multi-processing managed by the operating system.
In a browser, a tab has a single main thread, and if, say, a game runs its own main loop that never yields control back to the tab, that tab will just lock up.
A game that wants to run in a tab can't have a real main loop.
Instead, it has to be adapted to run just a single iteration of its main loop, and have Emscripten yield control to the browser.
Emscripten then runs this loop at around 60 FPS on the game's behalf.
So everything is good, right?

Unfortunately, RuggRogue has a special requirement for its own game loop.
When RuggRogue isn't handling an input event or animating something, it waits for an event, acting more like a GUI program than a game.
I pored over a lot of documentation, but for the life of me I could not find a good way to get Emscripten to support this kind of execution flow.
In order for RuggRogue to keep its own game loop while running in a browser tab without locking it up, I had to reach for a transformation known as [Asyncify](https://emscripten.org/docs/porting/asyncify.html).
The link explains what it does better than I can here.
Sadly, it's a pretty invasive transformation with a high CPU cost, but it allows CPU savings to occur when the player is idle, so it's still a net win.

Asyncify saves CPU by substituting `sleep` calls that RuggRogue makes during its main loop with the browser's [setTimeout](https://developer.mozilla.org/en-US/docs/Web/API/setTimeout) JavaScript function.
But there's a problem: RuggRogue relies on fine-grained `sleep` calls for smooth gameplay, but [setTimeout has delays](https://developer.mozilla.org/en-US/docs/Web/API/setTimeout#reasons_for_delays_longer_than_specified) when called repeatedly in a deep call stack.
It just so happens that the Asyncify transformation leads to very deep call stacks.
The result?
RuggRogue suffers unavoidable stutter in the web version.
There's no way around it without redoing its approach to web support entirely.

As well as the stutter, Emscripten is tricky to use with Rust in general.
In particular, it relies on the output format of [LLVM](https://llvm.org/) tools.
These formats are *not* stable across versions, so Emscripten relies on the most recent revision of LLVM at the time of development.
Meanwhile, Rust runs its own version of LLVM which is *not* the most recent revision of LLVM at any given time.
In order to correctly build a program with Rust and Emscripten, they usually have to use matching LLVM versions.
The LLVM version used by Rust can be found using `rustc --version --verbose`, but I couldn't find how to do the same for Emscripten anywhere I searched.
The use of version **1.39.20** is from [Therocode's blog](https://web.archive.org/web/20210618192808/https://blog.therocode.net/2020/10/a-guide-to-rust-sdl2-emscripten), who I can only assume did a deep dive into the release histories of Emscripten and LLVM to discover the version number.
Using the newest version of Emscripten with Rust will likely not work.

I would strongly consider taking the extra time to learn [Rust and WebAssembly](https://rustwasm.github.io/docs.html) without the Emscripten bit in the future.
I don't know if it would have gained results any quicker, but it seems like it would have dodged a lot of the headaches mentioned above.

## The Migrated Off Of: Piston

RuggRogue did not begin life as an SDL game; it began life as a Piston game.
[Piston](https://crates.io/crates/piston) is one of the earliest Rust game engines that existed, if not the earliest.
I initially chose it because it seemed like the only game engine that would let me write my own game loop, and because I didn't know any better.

RuggRogue no longer uses Piston.
Using Piston to draw a grid of characters onto the screen in the most obvious way led to extremely poor performance.
Trawling through documentation spread out across Piston's many sub-crates did not reveal any way to improve performance, so eventually it was just dropped entirely.
Switching from Piston to plain SDL both drastically dropped the compile time and boosted the performance of RuggRogue by *a lot*.
