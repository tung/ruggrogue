# Word Wrapping

If you play RuggRogue, you'll notice the messages that appear in the sidebar.
There's enough room for messages up to 32 characters long to appear unbroken; anything longer than that must be *word wrapped*.
Word wrapping is the act of breaking a long line down into a sequence of shorter lines that fit inside a desired width while keeping words whole.

All of RuggRogue's word wrapping is done by the `ruggrogue::word_wrap` function in the `src/lib/word_wrap.rs` file.
If you're not used to Rust's iterator system, the logic might be hard to follow, so I'm going to do something a bit different here than in the rest of the book: we're going to build the whole thing from the ground up!
This chapter contains runnable code samples; press the "play" button in the top corner to build and run the examples (they get fed through the [Rust Playground](https://play.rust-lang.org/)).
If you get time-outs, there's a button to copy the code to the clipboard, and you can paste that into a file and throw it at `rustc` to run the demo if you have Rust installed.

Before we start, read the [iterators chapter of the Rust book](https://doc.rust-lang.org/stable/book/ch13-02-iterators.html).
No, really, I'll wait; you'll want to be familiar with Rust's iterators before continuing.

...

Okay, now that you've read that (or not), the key take-away is this idea of *iterator adaptors*: functions that take an existing iterator and produces a new one that adds their own processing to the end.
The job of the `ruggrogue::word_wrap` function is to create an iterator that takes a string slice (a reference to a sequence of characters in a string) and produces multiple string slices that all fit within a given width.
RuggRogue prints these word-wrapped strings onto tile grids, so the width here is measured as the number of characters per line.

Broadly speaking, the `ruggrogue::word_wrap` function creates an iterator that does the following:

1. Break the input string into lines based on any existing line break characters in the string itself.
2. Break each line down into characters and their byte offsets (all Rust strings are UTF-8 encoded).
3. Prepare the characters and byte offsets for word scanning.
4. Scan for words, emitting the byte offsets of whole words and individual whitespace characters.
5. Prepare the word and whitespace byte offset data for line building.
6. Build line data by fitting a sequence of whitespaces followed by a word onto the current line if it fits, or starting a new line with only the word if it doesn't.
7. Convert the line data back into strings.

## Step 1: Break on Existing Lines

This all starts really basic: we need a function that takes an input string and a maximum character width, and returns an iterator.
To see the results of all of this, we need some sample input and some output logic, maybe something like this:

```rust
fn main() {
    let max_length = 15;
    let msg = "  I'm nöbödy!  Whö are yöu?
Are yöu nöbödy, töö?
Then there's a pair of us - don't tell!
They'd banish us, you know.

  How dreary to be somebody!
How public, li-ke a ƒrog
To tell your name the live-long day
To an admiring bog!

  - nobody";

    for tmp in word_wrap(msg, max_length) {
        println!("{:?}", tmp);
    }
}

fn word_wrap(input: &str, max_length: usize) -> impl Iterator<Item = &str> {
    assert!(max_length > 0);

    input.lines()
}
```

I've taken the liberty of throwing in some random non-ASCII characters to ensure that we're handling the difference between bytes and characters, as you have to when handling UTF-8 encoded strings.
You should see the input string just being broken into the lines using Rust's [`str::lines`](https://doc.rust-lang.org/std/primitive.str.html#method.lines) function, which returns a simple iterator that does just that.
Rust's `for` loops automatically understand iterators, so that's what the main function does to produce its output.

From here on, we'll leave out the `main` function to focus on the `word_wrap` function that we're building up, but it will still be there when you run the code samples.
There's a toggle button in the top right of these samples to show the full demo code if you want to see it.

## Step 2: Characters and Byte Offsets

In order to perform word wrapping we'll need to know how each line breaks down into characters, so we know what's whitespace and what belongs to a word.
We'll also need the byte offsets of each of these characters, which can be larger than a single byte if they're not in the ASCII character range.
The byte offsets will be used to create the final string slices that refer back to the original input string data; we don't want to allocate string storage to replicate parts of strings that already exist!

```rust
# fn main() {
#     let max_length = 15;
#     let msg = "  I'm nöbödy!  Whö are yöu?
# Are yöu nöbödy, töö?
# Then there's a pair of us - don't tell!
# They'd banish us, you know.
#
#   How dreary to be somebody!
# How public, li-ke a ƒrog
# To tell your name the live-long day
# To an admiring bog!
#
#   - nobody";
#
#     for tmp in word_wrap(msg, max_length) {
#         println!("{:?}", tmp);
#     }
# }
#
# fn word_wrap(input: &str, max_length: usize) -> impl Iterator<Item = (usize, char)> + '_ {
#     assert!(max_length > 0);
#
    input.lines().flat_map(move |line| {
        line.char_indices()
    })
# }
```

Running the above sample should print a long list of characters and their byte offsets.

Rust's [`str::char_indices`](https://doc.rust-lang.org/std/primitive.str.html#method.char_indices) gives us the characters and byte offsets that you'll see if you run the code sample.
This by itself would give us a nested iterator: one iterator of chars-and-offsets per line.
We use Rust's [`Iterator::flat_map`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.flat_map) function to remove this nesting to get ourselves a single long list of chars-and-offsets.
Note that the byte offsets shown are relative to the start of each line, not the input string as a whole.

## Step 3: Prepare for Word Scanning

The word scanning that we're going to do next is done a character at a time, but we need to perform some finalization at the end.
But iterators in Rust only do things on a per-item basis.
How do we get an iterator to do something *after* the last item?

We're going to use Rust's [`Option` type](https://doc.rust-lang.org/std/option/enum.Option.html) to wrap each item inside a `Some` variant.
We'll then use [`Iterator::chain`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.chain) with a one-item iterator of the `None` variant using [`iter::once`](https://doc.rust-lang.org/std/iter/fn.once.html) to act as the final *sentinel* value.
Putting it together gives us something that looks like this:

```rust
# fn main() {
#     let max_length = 15;
#     let msg = "  I'm nöbödy!  Whö are yöu?
# Are yöu nöbödy, töö?
# Then there's a pair of us - don't tell!
# They'd banish us, you know.
#
#   How dreary to be somebody!
# How public, li-ke a ƒrog
# To tell your name the live-long day
# To an admiring bog!
#
#   - nobody";
#
#     for tmp in word_wrap(msg, max_length) {
#         println!("{:?}", tmp);
#     }
# }
#
# fn word_wrap(input: &str, max_length: usize) -> impl Iterator<Item = (usize, Option<char>)> + '_ {
#     assert!(max_length > 0);
#
    input.lines().flat_map(move |line| {
        line.char_indices()
            .map(|(pos, ch)| (pos, Some(ch)))
            .chain(std::iter::once((line.len(), None))) // character sentinel
    })
# }
```

Running the code sample should produce the same characters and offsets wrapped with `Some`, with a single `None` representing the end of each line.

## Step 4: Scan for Words

Now that we have characters, offsets and a sentinel to mark the end of each line, how do we detect words?
For that we'll use Rust's [`iter::scan`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.scan) function.
I recommend reading its official documentation.
The idea here is that we want to step through each character, building up memory of where each word begins, and the previous character to detect where each word should end.
We also treat hyphens as the end of a word, and break any words that exceed the maximum line width.
Every time we detect the end of a word, we need to emit data with `Some`, otherwise we'll emit a `None` to indicate that we're still processing characters.

```rust
# fn main() {
#     let max_length = 15;
#     let msg = "  I'm nöbödy!  Whö are yöu?
# Are yöu nöbödy, töö?
# Then there's a pair of us - don't tell!
# They'd banish us, you know.
#
#   How dreary to be somebody!
# How public, li-ke a ƒrog
# To tell your name the live-long day
# To an admiring bog!
#
#   - nobody";
#
#     for tmp in word_wrap(msg, max_length) {
#         println!("{:?}", tmp);
#     }
# }
#
# fn word_wrap(input: &str, max_length: usize) -> impl Iterator<Item = Option<(usize, usize, usize, bool)>> + '_ {
#     assert!(max_length > 0);
#
    input.lines().flat_map(move |line| {
        line.char_indices()
            .map(|(pos, ch)| (pos, Some(ch)))
            .chain(std::iter::once((line.len(), None))) // character sentinel
            .scan(None, move |state, (pos, ch)| {
                // Break into words and single spaces.
                if let Some(ch) = ch {
                    if let Some((in_word, start_pos, char_count, last_char)) = state {
                        if *char_count >= max_length || *last_char == '-' || ch.is_whitespace() {
                            // Line-length or hyphen-divided word, or mid-line whitespace.
                            let was_word = *in_word;
                            let last_start_pos = *start_pos;
                            let last_char_count = *char_count;
                            *in_word = !ch.is_whitespace();
                            *start_pos = pos;
                            *char_count = 1;
                            *last_char = ch;
                            Some(Some((last_start_pos, pos, last_char_count, was_word)))
                        } else if *in_word {
                            // Word continuation.
                            *char_count += 1;
                            *last_char = ch;
                            Some(None)
                        } else {
                            // Entering a word after whitespace.
                            let was_word = *in_word;
                            let last_start_pos = *start_pos;
                            let last_char_count = *char_count;
                            *in_word = true;
                            *start_pos = pos;
                            *char_count = 1;
                            *last_char = ch;
                            Some(Some((last_start_pos, pos, last_char_count, was_word)))
                        }
                    } else {
                        // Start of the line.
                        let in_word = !ch.is_whitespace();
                        let start_pos = pos;
                        let char_count = 1;
                        let last_char = ch;
                        *state = Some((in_word, start_pos, char_count, last_char));
                        Some(None)
                    }
                } else {
                    // End of the line.
                    if let Some((in_word, start_pos, char_count, _)) = state {
                        // Finish the final word or whitespace.
                        Some(Some((*start_pos, pos, *char_count, *in_word)))
                    } else {
                        // Empty line.
                        Some(Some((pos, pos, 0, false)))
                    }
                }
            })
    })
# }
```

If you run the above code, you'll see that the output is a mixture of `Some` and `None` variants.
The `None`s represent intermediate working steps when a character is scanned but a word or whitespace hasn't been fully processed yet.
The `Some` means a whole word or single whitespace character has been processed.
The `Some` holds a 4-tuple consisting of three numbers and boolean flag.
The first two numbers are the byte offset of the start (inclusive) and end (exclusive) of each word or whitespace character.
The third number is the length of the word, in characters.
The boolean is `true` when it represents a word, or `false` for a whitespace character.

Note the bottom of the code that picks up the `None` sentinel value at the end of the line to finish the last word or whitespace character.

## Step 5: Prepare for Line Scanning

In order to build characters up into words, we needed to wrap each value with `Some` and add a `None` sentinel value.
We need to perform scanning again, this time for words.
If you ran the previous code sample, you'll notice that our data is already wrapped up in `Some`, but there's a lot of `None` values mixed in there.
We're going to use [`iter::filter`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.filter) along with [`Option::is_some`](https://doc.rust-lang.org/std/option/enum.Option.html#method.is_some) to clear out those `None` values.
We'd also like to re-add that single `None` value to mark the end of the line again to handle the final word or whitespace after we perform line scanning.

```rust
# fn main() {
#     let max_length = 15;
#     let msg = "  I'm nöbödy!  Whö are yöu?
# Are yöu nöbödy, töö?
# Then there's a pair of us - don't tell!
# They'd banish us, you know.
#
#   How dreary to be somebody!
# How public, li-ke a ƒrog
# To tell your name the live-long day
# To an admiring bog!
#
#   - nobody";
#
#     for tmp in word_wrap(msg, max_length) {
#         println!("{:?}", tmp);
#     }
# }
#
# fn word_wrap(input: &str, max_length: usize) -> impl Iterator<Item = Option<(usize, usize, usize, bool)>> + '_ {
#     assert!(max_length > 0);
#
    input.lines().flat_map(move |line| {
        line.char_indices()
            .map(|(pos, ch)| (pos, Some(ch)))
            .chain(std::iter::once((line.len(), None))) // character sentinel
            .scan(None, move |state, (pos, ch)| {
                // Break into words and single spaces.
                // ...
#                 if let Some(ch) = ch {
#                     if let Some((in_word, start_pos, char_count, last_char)) = state {
#                         if *char_count >= max_length || *last_char == '-' || ch.is_whitespace() {
#                             // Line-length or hyphen-divided word, or mid-line whitespace.
#                             let was_word = *in_word;
#                             let last_start_pos = *start_pos;
#                             let last_char_count = *char_count;
#                             *in_word = !ch.is_whitespace();
#                             *start_pos = pos;
#                             *char_count = 1;
#                             *last_char = ch;
#                             Some(Some((last_start_pos, pos, last_char_count, was_word)))
#                         } else if *in_word {
#                             // Word continuation.
#                             *char_count += 1;
#                             *last_char = ch;
#                             Some(None)
#                         } else {
#                             // Entering a word after whitespace.
#                             let was_word = *in_word;
#                             let last_start_pos = *start_pos;
#                             let last_char_count = *char_count;
#                             *in_word = true;
#                             *start_pos = pos;
#                             *char_count = 1;
#                             *last_char = ch;
#                             Some(Some((last_start_pos, pos, last_char_count, was_word)))
#                         }
#                     } else {
#                         // Start of the line.
#                         let in_word = !ch.is_whitespace();
#                         let start_pos = pos;
#                         let char_count = 1;
#                         let last_char = ch;
#                         *state = Some((in_word, start_pos, char_count, last_char));
#                         Some(None)
#                     }
#                 } else {
#                     // End of the line.
#                     if let Some((in_word, start_pos, char_count, _)) = state {
#                         // Finish the final word or whitespace.
#                         Some(Some((*start_pos, pos, *char_count, *in_word)))
#                     } else {
#                         // Empty line.
#                         Some(Some((pos, pos, 0, false)))
#                     }
#                 }
            })
            .filter(Option::is_some)
            .chain(Some(None)) // word sentinel
    })
# }
```

The output should be the same as before, but with all of the `None` values gone, except for a final `None` to mark the end of each line.

## Step 6: Build Lines by Scanning Words and Whitespaces

We now have everything we need to create a scanner that builds up lines.
The idea of this scanner is to extend a line continuously with a sequence of whitespace characters followed by a single word.
If the sum of the character counts of that sequence and the existing line fits in the desired width, we append the whole thing to the line.
If not, we'll emit the line as-is, and start a new line *without* the preceeding whitespace characters.

```rust
# fn main() {
#     let max_length = 15;
#     let msg = "  I'm nöbödy!  Whö are yöu?
# Are yöu nöbödy, töö?
# Then there's a pair of us - don't tell!
# They'd banish us, you know.
#
#   How dreary to be somebody!
# How public, li-ke a ƒrog
# To tell your name the live-long day
# To an admiring bog!
#
#   - nobody";
#
#     for tmp in word_wrap(msg, max_length) {
#         println!("{:?}", tmp);
#     }
# }
#
# fn word_wrap(input: &str, max_length: usize) -> impl Iterator<Item = Option<(usize, usize)>> + '_ {
#     assert!(max_length > 0);
#
    input.lines().flat_map(move |line| {
        line.char_indices()
            .map(|(pos, ch)| (pos, Some(ch)))
            .chain(std::iter::once((line.len(), None))) // character sentinel
            .scan(None, move |state, (pos, ch)| {
                // Break into words and single spaces.
                // ...
#                 if let Some(ch) = ch {
#                     if let Some((in_word, start_pos, char_count, last_char)) = state {
#                         if *char_count >= max_length || *last_char == '-' || ch.is_whitespace() {
#                             // Line-length or hyphen-divided word, or mid-line whitespace.
#                             let was_word = *in_word;
#                             let last_start_pos = *start_pos;
#                             let last_char_count = *char_count;
#                             *in_word = !ch.is_whitespace();
#                             *start_pos = pos;
#                             *char_count = 1;
#                             *last_char = ch;
#                             Some(Some((last_start_pos, pos, last_char_count, was_word)))
#                         } else if *in_word {
#                             // Word continuation.
#                             *char_count += 1;
#                             *last_char = ch;
#                             Some(None)
#                         } else {
#                             // Entering a word after whitespace.
#                             let was_word = *in_word;
#                             let last_start_pos = *start_pos;
#                             let last_char_count = *char_count;
#                             *in_word = true;
#                             *start_pos = pos;
#                             *char_count = 1;
#                             *last_char = ch;
#                             Some(Some((last_start_pos, pos, last_char_count, was_word)))
#                         }
#                     } else {
#                         // Start of the line.
#                         let in_word = !ch.is_whitespace();
#                         let start_pos = pos;
#                         let char_count = 1;
#                         let last_char = ch;
#                         *state = Some((in_word, start_pos, char_count, last_char));
#                         Some(None)
#                     }
#                 } else {
#                     // End of the line.
#                     if let Some((in_word, start_pos, char_count, _)) = state {
#                         // Finish the final word or whitespace.
#                         Some(Some((*start_pos, pos, *char_count, *in_word)))
#                     } else {
#                         // Empty line.
#                         Some(Some((pos, pos, 0, false)))
#                     }
#                 }
            })
            .filter(Option::is_some)
            .chain(Some(None)) // word sentinel
            .scan(None, move |state, word_data| {
                // Build up lines up to max_length.
                if let Some((word_start, word_end, word_char_count, is_word)) = word_data {
                    if let Some((line_start, line_end, final_end, line_char_count)) = state {
                        if is_word {
                            if *line_char_count + word_char_count <= max_length {
                                // Word fits on line, so include it.
                                *line_end = word_end;
                                *final_end = word_end;
                                *line_char_count += word_char_count;
                                Some(None)
                            } else {
                                // Word exceeds line, so start a new line with it instead.
                                let last_line_start = *line_start;
                                let last_line_end = *line_end;
                                *line_start = word_start;
                                *line_end = word_end;
                                *final_end = word_end;
                                *line_char_count = word_char_count;
                                Some(Some((last_line_start, last_line_end)))
                            }
                        } else {
                            if *line_char_count + word_char_count <= max_length {
                                // Whitespace fits on line, so include it when finishing words.
                                *final_end = word_end;
                            }
                            *line_char_count += word_char_count;
                            Some(None)
                        }
                    } else {
                        // The first word.
                        let line_start = word_start;
                        let line_end = if is_word { word_end } else { word_start };
                        let final_end = word_end;
                        let line_char_count = word_char_count;
                        *state = Some((line_start, line_end, final_end, line_char_count));
                        Some(None)
                    }
                } else {
                    // End of words.
                    if let Some((line_start, _, final_end, _)) = state {
                        // Finish the line.
                        Some(Some((*line_start, *final_end)))
                    } else {
                        // Empty line.
                        Some(Some((0, 0)))
                    }
                }
            })
    })
# }
```

Running the above sample code will output just the byte offsets of the start (inclusive) and end (exclusive) of each wrapped line.
All those `Some` and `None` values are littered in there, and at this point we really only need the data inside each `Some`.
There's a trick we can do here: Rust can treat an `Option` like a container that has no items (`None`) or a single item (`Some(...)`).
What's more, Rust can convert an `Option` into an iterator, so if we squint hard enough, we sort of have a list of *iterators*.
We can therefore use the `iter::flatten` function to clean out the `None` values and extract the data from the `Some` variants in one fell swoop!

```rust
# fn main() {
#     let max_length = 15;
#     let msg = "  I'm nöbödy!  Whö are yöu?
# Are yöu nöbödy, töö?
# Then there's a pair of us - don't tell!
# They'd banish us, you know.
#
#   How dreary to be somebody!
# How public, li-ke a ƒrog
# To tell your name the live-long day
# To an admiring bog!
#
#   - nobody";
#
#     for tmp in word_wrap(msg, max_length) {
#         println!("{:?}", tmp);
#     }
# }
#
# fn word_wrap(input: &str, max_length: usize) -> impl Iterator<Item = (usize, usize)> + '_ {
#     assert!(max_length > 0);
#
    input.lines().flat_map(move |line| {
        line.char_indices()
            .map(|(pos, ch)| (pos, Some(ch)))
            .chain(std::iter::once((line.len(), None))) // character sentinel
            .scan(None, move |state, (pos, ch)| {
                // Break into words and single spaces.
                // ...
#                 if let Some(ch) = ch {
#                     if let Some((in_word, start_pos, char_count, last_char)) = state {
#                         if *char_count >= max_length || *last_char == '-' || ch.is_whitespace() {
#                             // Line-length or hyphen-divided word, or mid-line whitespace.
#                             let was_word = *in_word;
#                             let last_start_pos = *start_pos;
#                             let last_char_count = *char_count;
#                             *in_word = !ch.is_whitespace();
#                             *start_pos = pos;
#                             *char_count = 1;
#                             *last_char = ch;
#                             Some(Some((last_start_pos, pos, last_char_count, was_word)))
#                         } else if *in_word {
#                             // Word continuation.
#                             *char_count += 1;
#                             *last_char = ch;
#                             Some(None)
#                         } else {
#                             // Entering a word after whitespace.
#                             let was_word = *in_word;
#                             let last_start_pos = *start_pos;
#                             let last_char_count = *char_count;
#                             *in_word = true;
#                             *start_pos = pos;
#                             *char_count = 1;
#                             *last_char = ch;
#                             Some(Some((last_start_pos, pos, last_char_count, was_word)))
#                         }
#                     } else {
#                         // Start of the line.
#                         let in_word = !ch.is_whitespace();
#                         let start_pos = pos;
#                         let char_count = 1;
#                         let last_char = ch;
#                         *state = Some((in_word, start_pos, char_count, last_char));
#                         Some(None)
#                     }
#                 } else {
#                     // End of the line.
#                     if let Some((in_word, start_pos, char_count, _)) = state {
#                         // Finish the final word or whitespace.
#                         Some(Some((*start_pos, pos, *char_count, *in_word)))
#                     } else {
#                         // Empty line.
#                         Some(Some((pos, pos, 0, false)))
#                     }
#                 }
            })
            .filter(Option::is_some)
            .chain(Some(None)) // word sentinel
            .scan(None, move |state, word_data| {
                // Build up lines up to max_length.
                // ...
#                 if let Some((word_start, word_end, word_char_count, is_word)) = word_data {
#                     if let Some((line_start, line_end, final_end, line_char_count)) = state {
#                         if is_word {
#                             if *line_char_count + word_char_count <= max_length {
#                                 // Word fits on line, so include it.
#                                 *line_end = word_end;
#                                 *final_end = word_end;
#                                 *line_char_count += word_char_count;
#                                 Some(None)
#                             } else {
#                                 // Word exceeds line, so start a new line with it instead.
#                                 let last_line_start = *line_start;
#                                 let last_line_end = *line_end;
#                                 *line_start = word_start;
#                                 *line_end = word_end;
#                                 *final_end = word_end;
#                                 *line_char_count = word_char_count;
#                                 Some(Some((last_line_start, last_line_end)))
#                             }
#                         } else {
#                             if *line_char_count + word_char_count <= max_length {
#                                 // Whitespace fits on line, so include it when finishing words.
#                                 *final_end = word_end;
#                             }
#                             *line_char_count += word_char_count;
#                             Some(None)
#                         }
#                     } else {
#                         // The first word.
#                         let line_start = word_start;
#                         let line_end = if is_word { word_end } else { word_start };
#                         let final_end = word_end;
#                         let line_char_count = word_char_count;
#                         *state = Some((line_start, line_end, final_end, line_char_count));
#                         Some(None)
#                     }
#                 } else {
#                     // End of words.
#                     if let Some((line_start, _, final_end, _)) = state {
#                         // Finish the line.
#                         Some(Some((*line_start, *final_end)))
#                     } else {
#                         // Empty line.
#                         Some(Some((0, 0)))
#                     }
#                 }
            })
            .flatten()
    })
# }
```

Running the above code sample should produce a cleaned-up version of the output from the previous code sample.

## Step 7: Convert Data back into String Slices

We originally wanted string slices of word-wrapped lines, which trivially builds on the work that's been done up to this point.

```rust
# fn main() {
#     let max_length = 15;
#     let msg = "  I'm nöbödy!  Whö are yöu?
# Are yöu nöbödy, töö?
# Then there's a pair of us - don't tell!
# They'd banish us, you know.
#
#   How dreary to be somebody!
# How public, li-ke a ƒrog
# To tell your name the live-long day
# To an admiring bog!
#
#   - nobody";
#
#     for tmp in word_wrap(msg, max_length) {
#         println!("{:?}", tmp);
#     }
# }
#
# fn word_wrap(input: &str, max_length: usize) -> impl Iterator<Item = &str> {
#     assert!(max_length > 0);
#
    input.lines().flat_map(move |line| {
        line.char_indices()
            .map(|(pos, ch)| (pos, Some(ch)))
            .chain(std::iter::once((line.len(), None))) // character sentinel
            .scan(None, move |state, (pos, ch)| {
                // Break into words and single spaces.
                // ...
#                 if let Some(ch) = ch {
#                     if let Some((in_word, start_pos, char_count, last_char)) = state {
#                         if *char_count >= max_length || *last_char == '-' || ch.is_whitespace() {
#                             // Line-length or hyphen-divided word, or mid-line whitespace.
#                             let was_word = *in_word;
#                             let last_start_pos = *start_pos;
#                             let last_char_count = *char_count;
#                             *in_word = !ch.is_whitespace();
#                             *start_pos = pos;
#                             *char_count = 1;
#                             *last_char = ch;
#                             Some(Some((last_start_pos, pos, last_char_count, was_word)))
#                         } else if *in_word {
#                             // Word continuation.
#                             *char_count += 1;
#                             *last_char = ch;
#                             Some(None)
#                         } else {
#                             // Entering a word after whitespace.
#                             let was_word = *in_word;
#                             let last_start_pos = *start_pos;
#                             let last_char_count = *char_count;
#                             *in_word = true;
#                             *start_pos = pos;
#                             *char_count = 1;
#                             *last_char = ch;
#                             Some(Some((last_start_pos, pos, last_char_count, was_word)))
#                         }
#                     } else {
#                         // Start of the line.
#                         let in_word = !ch.is_whitespace();
#                         let start_pos = pos;
#                         let char_count = 1;
#                         let last_char = ch;
#                         *state = Some((in_word, start_pos, char_count, last_char));
#                         Some(None)
#                     }
#                 } else {
#                     // End of the line.
#                     if let Some((in_word, start_pos, char_count, _)) = state {
#                         // Finish the final word or whitespace.
#                         Some(Some((*start_pos, pos, *char_count, *in_word)))
#                     } else {
#                         // Empty line.
#                         Some(Some((pos, pos, 0, false)))
#                     }
#                 }
            })
            .filter(Option::is_some)
            .chain(Some(None)) // word sentinel
            .scan(None, move |state, word_data| {
                // Build up lines up to max_length.
                // ...
#                 if let Some((word_start, word_end, word_char_count, is_word)) = word_data {
#                     if let Some((line_start, line_end, final_end, line_char_count)) = state {
#                         if is_word {
#                             if *line_char_count + word_char_count <= max_length {
#                                 // Word fits on line, so include it.
#                                 *line_end = word_end;
#                                 *final_end = word_end;
#                                 *line_char_count += word_char_count;
#                                 Some(None)
#                             } else {
#                                 // Word exceeds line, so start a new line with it instead.
#                                 let last_line_start = *line_start;
#                                 let last_line_end = *line_end;
#                                 *line_start = word_start;
#                                 *line_end = word_end;
#                                 *final_end = word_end;
#                                 *line_char_count = word_char_count;
#                                 Some(Some((last_line_start, last_line_end)))
#                             }
#                         } else {
#                             if *line_char_count + word_char_count <= max_length {
#                                 // Whitespace fits on line, so include it when finishing words.
#                                 *final_end = word_end;
#                             }
#                             *line_char_count += word_char_count;
#                             Some(None)
#                         }
#                     } else {
#                         // The first word.
#                         let line_start = word_start;
#                         let line_end = if is_word { word_end } else { word_start };
#                         let final_end = word_end;
#                         let line_char_count = word_char_count;
#                         *state = Some((line_start, line_end, final_end, line_char_count));
#                         Some(None)
#                     }
#                 } else {
#                     // End of words.
#                     if let Some((line_start, _, final_end, _)) = state {
#                         // Finish the line.
#                         Some(Some((*line_start, *final_end)))
#                     } else {
#                         // Empty line.
#                         Some(Some((0, 0)))
#                     }
#                 }
            })
            .flatten()
            .map(move |(start, end)| &line[start..end])
    })
# }
```

And that's it!
If you run this code, you'll see the original input wrapped into lines no longer than 15 characters each.
Note that lines with non-ASCII multi-byte characters still count characters correctly, and hyphenated words are split across lines.

## Conclusion

Using Rust's iterator API to perform word wrapping has a couple of advantages over something like hand-rolled loops.
As part of the standard library, I have strong confidence that these iterator adaptor functions are always correct.
Writing lots of nested loops by hand means extra code and lots of extra small book-keeping variables, each of which is a chance for bugs to slip in and cause headaches.
Using iterators also encapsulates all of these tracking variables into a single bundle inside the iterator: calling the `ruggrogue::word_wrap` function returns an iterator *immediately* so a `for` loop can process it all at its own pace.

However, I still don't feel like this iterator approach is the easiest code to read.
But in order to write a simpler version, stable Rust would need a language feature known as *generators*; look them up if you're curious.
Still, this word wrapping code manages to perform its work on demand, avoid memory allocations and is fast enough to run every frame, so all-in-all it worked out pretty well for the game.
