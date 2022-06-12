use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Deserialize, Serialize)]
pub struct Messages {
    capacity: u16,
    msg_queue: VecDeque<String>,
    num_highlighted: usize,
    want_separator: bool,
}

impl Messages {
    pub fn new(capacity: u16) -> Self {
        assert!(capacity > 0);

        Self {
            capacity,
            msg_queue: VecDeque::with_capacity(capacity as usize),
            num_highlighted: 0,
            want_separator: false,
        }
    }

    pub fn replace(&mut self, replacement: Self) {
        self.msg_queue = replacement.msg_queue;
        self.num_highlighted = replacement.num_highlighted;
    }

    pub fn reset(&mut self) {
        self.msg_queue.clear();
        self.num_highlighted = 0;
    }

    pub fn add(&mut self, msg: String) {
        let space_needed = if self.want_separator { 2 } else { 1 };

        if self.msg_queue.len() + space_needed >= self.capacity as usize {
            for _ in 0..space_needed {
                self.msg_queue.pop_front();
            }
            self.num_highlighted = self.num_highlighted.min(self.msg_queue.len());
        }

        if self.want_separator {
            self.msg_queue.push_back("".to_string());
            self.want_separator = false;
        }

        self.msg_queue.push_back(msg);
        self.num_highlighted = self.num_highlighted.saturating_add(1);
    }

    pub fn separator(&mut self) {
        self.want_separator = true;
    }

    /// Returns an iterator over messages in reverse order, each with a highlight flag.
    pub fn rev_iter(&self) -> impl Iterator<Item = (&str, bool)> {
        self.msg_queue
            .iter()
            .rev()
            .enumerate()
            .map(move |(i, s)| (s.as_str(), i < self.num_highlighted))
    }

    pub fn reset_highlight(&mut self) {
        self.num_highlighted = 0;
    }
}
