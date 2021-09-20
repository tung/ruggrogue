/// Word wrap input string into lines with at most `max_length` characters.
///
/// Leading and trailing whitespace is preserved.  Accounts for multi-byte UTF-8 encoded
/// characters, but not combining characters or characters whose width != 1.
pub fn word_wrap(input: &str, max_length: usize) -> impl Iterator<Item = &str> {
    assert!(max_length > 0);

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
            .filter(Option::is_some)
            .chain(Some(None)) // word sentinel
            .scan(None, move |state, word_data| {
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
            .flatten()
            .map(move |(start, end)| &line[start..end])
    })
}
