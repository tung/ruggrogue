# Input Handling

When I sat down to write this chapter, I thought it was going to be really straightforward.
I mean, the input handling is just a buffer that accepts and distributes input events, right?
It should be really simple, and indeed it is.
However, as I look back at in the context of the finished game, I realize how odd it is that it exists, and I'll talk about that a little further on.
First, I'd like to talk about the problem it's supposed to solve.
If you'd like to follow along at home, the file that has all this is `src/lib/input_buffer.rs`.

## The Idea Behind the Input Buffer

Imagine you're playing the game.
It's the monster's turn and not the player's, but the player nonetheless presses a key.
What should happen?
We don't want to player to move during the monster's turn.
But since we've retrieved the input event of the key, if we don't handle it somehow, we won't get it again for the next update, even if that update would have been the player's turn.
An unhandled input is a dropped input, and dropped inputs feel *really* bad to play with.
What should we do?

The logical thing to do to avoid dropped inputs is to just save those input events for later, when they can be properly handled during the player's turn.
This is exact purpose of the `InputBuffer` struct: to save inputs so that they can be handled later.
Inputs are loaded into the buffer by the main game loop feeding `InputBuffer::handle_event` with SDL events.
Only keyboard inputs and the application close event are relevant to the game, so there's an `InputEvent` enum at the top of `src/lib/input_buffer.rs` that SDL events get coverted into.
The main game loop handles a few event types specially, but for the most part, any SDL event that isn't a keyboard input or application close is dropped at this point.
This is especially important to cull mass floods of events like mouse move events, of which dozens can be emitted per second.
Having the game run its full update and draw logic for every single mouse move event would be a pretty bad experience that the `InputBuffer` prevents entirely.

The conversion from SDL event to `InputEvent` is not complete, that is, a little bit of SDL sneaks through.
The `Keycode` inside `InputEvent::Press` and `InputEvent::Release` is still an SDL value, so the game still has to deal with SDL key values.
However, the game converts SDL keys into logical `GameKey` values in the `from_keycode` function that lives in `src/gamekey.rs`.
This conversion allows multiple SDL keys to map to a single key, which helps to provide both numpad and vi-keys movement keys by mapping them to one set of logical direction keys instead.

There's a more technical scenario to worry about with input events, where some update logic gets an input to check for a key, and instead there's a mouse move event.
That logic just quietly ignores the mouse move event.
Later on in the same update frame, some other update logic gets an input to check for a mouse move and instead sees a key.
That logic just quietly ignores the key event.
We just got two input events but handled neither of them, even though we should have been perfectly able to do so.
The `InputBuffer` struct solves this by operating a rate-limiting latch that forces a maximum of a single input event per update frame.
What happens is that if a mode's `update` function wants to check for input, it calls `InputBuffer::prepare_input` to prepare a single input for the whole update.
The input event itself can then be checked with `InputBuffer::get_input`, which *leaves* the input in the buffer.
After the update logic has completed, the main loop calls `InputBuffer::clear_input` to clear out that one input event so the next input can be prepared in a later frame.

## The Meaning of the Existence of the Input Buffer

If you've been following along, you'll know that RuggRogue uses SDL for input handling.
If you know SDL, you'll know that it maintains its own input queue, which is effectively a buffer.
If SDL already has an input buffer, why the heck do we need our own?

This is what I was talking about earlier about how odd it was that our own input buffer even exists.
Technically, I could have just pulled SDL events throughout the game update logic, and only kept the logical game key translation.

There's a bigger problem with the input buffer than mere redundancy, though.
Those hypothetical scenarios I mentioned earlier?
They never happen in the completed game as far as I can tell.
That is, the game never animates monster turns, so it's impossible for a monster turn to retrieve an input and not handle it.
Similarly, the game never asks for more than one input event per update anyway.
I feel conflicted over the existence of the input buffer.

The custom input buffer is not completely purposeless, though.
I believe a lot of desktop APIs get pretty upset if they give your application events and they aren't taken immediately; they act as if your application has frozen.
Having an input buffer that takes those events straight away at least avoids that particular problem.

Still, why have our own input buffer if SDL already has one?
To answer that, we'll need to take a trip down memory lane.

## The Origin Story of the Input Buffer

Recall that RuggRogue used to be based on Piston, and only later switched to SDL.
I do remember writing input handling code, but I don't recall the exact order of events.
One of the following things happened:

1. I wrote input logic with Piston's API, realized it was getting awkward to deal with everywhere and wrote the input buffer to confine most of it in a single file.
2. The same as above, except I realized the awkwardness in advance and wrote it to protect the game code from having Piston references everywhere.
3. Piston was everywhere in the game code, and I wrote the input buffer to make it easier to port the game off of Piston and onto anything else.

The more I think about it, the more likely the third scenario is what ended up happening.
The input buffer is a vestige of RuggRogue's Piston past that, to this day, sometimes haunts it in its sleep.
Rest easy, RuggRogue, you did good.
