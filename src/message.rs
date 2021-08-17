use std::collections::VecDeque;

pub struct Messages {
    msg_queue: VecDeque<String>,
    num_highlighted: usize,
}

impl Messages {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0);

        Self {
            msg_queue: VecDeque::with_capacity(capacity),
            num_highlighted: 0,
        }
    }

    pub fn reset(&mut self) {
        self.msg_queue.clear();
        self.num_highlighted = 0;
    }

    pub fn add(&mut self, msg: String) {
        if self.msg_queue.len() >= self.msg_queue.capacity() {
            self.msg_queue.pop_front();
            self.num_highlighted = self.num_highlighted.min(self.msg_queue.len());
        }

        self.msg_queue.push_back(msg);
        self.num_highlighted = self.num_highlighted.saturating_add(1);
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
