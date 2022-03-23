# Introduction

RuggRogue is a simple classic roguelike made with little more than [Rust](https://www.rust-lang.org) and [SDL2](https://libsdl.org).
RuggRogue can be played in a web browser or be built to be played natively; for more information, check out the [git repo](https://github.com/tung/ruggrogue).

This guide is companion to the source code of the game.
The reason that this exists was that I released the game as open source under the MIT License, hoping that it would serve as a useful case study of how to make a simple roguelike without leaning on roguelike helper libraries that are commonly used by beginner roguelike tutorials.
However, most people interested in making their own roguelike are already too busy with their own code to make sense of somebody else's, so the raw release of the source code of RuggRogue by itself wouldn't help them that much.
For those want to learn from the RuggRogue source code, this guide should make it easier to approach.
For those who are just curious about how a simple roguelike works, this guide also covers how the game solves various problems such as rendering, word wrapping, map generation and auto-run.

Before getting into this, there's one last thing I'd like to mention.
RuggRogue is my first non-trivial Rust project, and I was fairly new to Rust when I started, with little more than the [Rust Book](https://doc.rust-lang.org/book/) and part of the [Rust Roguelike Tutorial](bfnightly.bracketproductions.com/) under my belt.
Despite going to the effort of writing this source code guide, RuggRogue is not a model of what a perfect Rust project should look like.
It is the product of my desire to create a complete game with a scope larger than a tutorial or a tech demo.
As such, there are places where the code and structure are technically functional but awkward, which had to be left as-is in the name of finishing the project without burning extra months on it.
The code could be better, but then it also wouldn't be complete or released either.

## Overview

Here's a sneak peak at what this guide covers:

- **Dependencies** talks about the stuff that RuggRogue leans on to do things outside of its development scope, but still need to be handled.
- **Source Code Layout** gives a brief overview of why each file exists.
- **Overall Game Flow** provides a bird's eye view of how the game goes from launch to its game loop, managing control flow through what I refer to as a "mode stack".
- **Input Handling** describes how and why the game handles input the way that it does.
- **Event Handling** describes how the game reacts to events such as keyboard input and the game window being resized or closed.
- **Rendering** covers how the game displays graphical output using a home-grown system of tile grids, as well as making it all run fast enough to not feel terrible to play.
- **User interface** explains how menus and dialogs work, how the main game screen is drawn and how a request to close the application is handled.
- **Word Wrapping** is about how to break any line of more than a couple of words into multiple lines that fit within a given width.
- **Entity Component System** describes Shipyard and how RuggRogue uses it to store and access its game data.
- **Game Data** provides a run-down of the data that constitutes the game and how it's managed.
- **Saving and Loading** talks about the save file format, the game's approach to loading and save persistence for the web version.
- **Field of View** goes over the game's approach to high performance field of view calculation.
- **Pathfinding** is exactly as it says on the tin, but also covers mitigations and fallback pathfinding if a direct path can't be found in a reasonable number of steps.
- **Randomness** is about how the game uses seeds, magic numbers and hashing to leverage RNGs that seem random but aren't.
- **Map Generation** discusses the placement of rooms and corridors in maps.
- **Map Population** follows on from map generation with the topic of filling empty maps with things like monsters and items.
- **Auto-Run** goes over how the game figures out how to run along corridors and hook into the same places as player input to automatically issue commands to follow them.
- **Turn Order and Combat** covers how the player and monsters take turns, as well as damage and in particular avoiding the zero damage problem.
- **Items** provides an overview of how items like consumables and equipment work.
- **Hunger and Regeneration** discusses how the game tracks hunger and the book-keeping it does to gradually restore health over time.
- **Experience and Difficulty** goes into the game's numeric approach to balance: something that would otherwise be done by hand and is often omitted entirely by roguelike tutorials.
- **New Game Plus** talks about what happens when the player wins, both in terms of gameplay and technical details.
