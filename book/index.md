# Introduction

Welcome to the *RuggRogue Source Code Guide*!
This is a web book describing the internal workings of [RuggRogue](https://tung.github.io/ruggrogue/): a simple, complete roguelike inspired by the first part of the [Rust Roguelike Tutorial](https://bfnightly.bracketproductions.com).
Unlike that tutorial, however, it's made without the help of any game engine or roguelike helper libraries, instead relying on [SDL2](https://libsdl.org), with [Emscripten](https://emscripten.org) for the web port.

RuggRogue itself plays out like many other roguelikes: You fight your way down a procedurally generated dungeon through ever-stronger monsters.
Along the way you'll find weapons, armor and magical items that will aid you in your quest.

The source code of RuggRogue can be found at [its GitHub repository](https://github.com/tung/ruggrogue).
It consists of thirteen thousand lines of Rust code across forty four files.
How does it all fit together?
Read on and find out!

## About this Book

RuggRogue is a relatively small open source game, so in theory everything about how it works could be learned by simply reading its source code.
However, the code is arranged for the computer to run first and foremost, so broad ideas are obscured by a vast sea of details.
The aim of this book is to highlight these ideas, from a high-level perspective all the way down to how they translate into functions and variables.

Studying the code architecture of RuggRogue is interesting for a few reasons.
The game is directly inspired by the numerous roguelike tutorials that can found across the Internet, but it goes beyond them in a number of ways.
It directly implements algorithms that tutorials typically provide canned solutions to, such as field of view and pathfinding.
It also answers questions that such tutorials typically leave as an exercise to the reader, such as game balance, word wrapping and auto-run.

At the same time, RuggRogue is a complete game with a limited scope that doesn't go too much further than those roguelike tutorials.
A person who has followed one of them has a realistic chance of learning from the RuggRogue source code without being overwhelmed.

Finally, RuggRogue's source code architecture differs quite a bit from most roguelike tutorials.
RuggRogue arranges a lot of its logic into *game states* with an explicit *game state stack* (internally referred to as *"modes"* and *"the mode stack"*).
This allows different screens and menus to keep their data and the code very close together.
This technique is hard to find in roguelike tutorials, but it's described in this book.

## Who is this Book for?

This book is written for programmers, so prior knowledge is assumed for things like variables, branches, loops, functions and basic data structures such as stacks, vectors and hash maps.

The game is written in the Rust programming language, but I try to keep it simple, so Rust knowledge is helpful to follow along but not mandatory.
Readers coming from other programming languages may want to look up Rust topics such as [*traits*][rust-traits] (like interfaces in other languages), [*modules*][rust-modules], [*pattern matching*][rust-pattern-matching] and [*iterators*][rust-iterators].

[rust-traits]: https://doc.rust-lang.org/book/ch10-02-traits.html
[rust-modules]: https://doc.rust-lang.org/book/ch07-05-separating-modules-into-different-files.html
[rust-pattern-matching]: https://doc.rust-lang.org/book/ch06-00-enums.html
[rust-iterators]: https://doc.rust-lang.org/book/ch13-02-iterators.html

If you're an aspiring roguelike developer, this book will give you broad idea of the scope of a roguelike.
Reading a chapter in detail should serve as useful guidance as to how to implement features yourself.

If you're developing a roguelike yourself already, this book should serve as an interesting case study to compare and contrast your existing approaches to various features.
You may stumble across ideas you hadn't thought of to enhance the game you're working on.

If you're a programmer that's curious about game development in general, this book will shed some light on how a game functions under the hood.
Everything game-related must be handled by the source code, since there's no game engine for anything to hide in.

## How to Read this Book

Each chapter of the book is more or less standalone, so they can mostly be read in any order.
There are a few cross-references, most of which point backwards.

Chapters vary in balance between describing high-level ideas and fine-grained technical details.
Unfortunately, the early chapters are fairly detail-heavy due to establishing the technical base upon which all of the (hopefully) fun gameplay is built upon.
If it becomes too much to bear, feel free to skip the chapter and come back later.

In all of the chapters, there are many references to the names of files, functions, variables and other code-specific things.
You'll get the most out of this book with the source code handy in another window.

On the other hand, if you're not interested in juggling the game's source code while reading the book, you can still skim the chapters just for the ideas and skip over the source code references.

## Chapter Overview

[**Dependencies**](dependencies.md): The technology, libraries and tools used to create the game.

[**Source Code Layout**](source-code-layout.md): The location and purpose of each file and directory of the source code.

[**Overall Game Flow**](overall-game-flow.md): Game initialization, game loop and mode (game state) stack.

[**Event Handling**](event-handling.md): Handling of external events such as player input, window resizing and closing.

[**Rendering**](rendering.md): Drawing grids of tiles onto the screen and performance-improving techniques.

[**User Interface**](user-interface.md): How menus work and screens are laid out.

[**Options**](options.md): The options menu and how option changes are reflected in real-time.

[**Word Wrapping**](word-wrapping.md): How long lines of text are broken up to fit inside a limited space.

[**Entity Component System**](entity-component-system.md): How data is stored, retrieved and modified.

[**Game Data**](game-data.md): The different types of data components and how entities are created and destroyed.

[**Saving and Loading**](saving-and-loading.md): What save files look like and how they work.

[**Field of View**](field-of-view.md): Determining which tiles the player and monsters can see using shadow casting.

[**Pathfinding**](pathfinding.md): A\* search algorithm for finding paths between points, its uses and subtleties.

[**Randomness**](randomness.md): Pseudo-random number generation, seeds and reproducibility of results.

[**Map Generation**](map-generation.md): Data structures and logic for randomly laying out rooms, corridors and stairs.

[**Map Population**](map-population.md): Placement of the player, monsters and items in freshly-generated maps.

[**Auto-Run**](auto-run.md): Implementing the smart directional "auto-run" movement command that follows corridors and crosses open space.

[**Turn Order and Combat**](turn-order-and-combat.md): Monster turns, melee combat, damage formula and death handling.

[**Items**](items.md): List of items, spawn rates, item-related data structures, menus and usage logic.

[**Hunger and Regeneration**](hunger-and-regeneration.md): How hunger fits into the rest of the game and its link to regeneration.

[**Experience and Difficulty**](experience-and-difficulty.md): Game balance, numeric progression and pacing the flow of new monsters, weapons and armor.

[**Monsters**](monsters.md): Mini-chapter with a list of monsters and cross-references to other chapters about how they work.

[**New Game Plus**](new-game-plus.md): Gameplay and implementation details of how successive wins change play-throughs.
