use std::collections::VecDeque;

pub struct Messages(VecDeque<String>);

impl Messages {
    pub fn new(capacity: usize) -> Self {
        Self(VecDeque::with_capacity(capacity))
    }

    pub fn add(&mut self, msg: String) {
        if self.0.len() == self.0.capacity() {
            self.0.pop_front();
        }
        self.0.push_back(msg);
    }

    pub fn rev_iter(&self) -> impl Iterator<Item = &String> {
        self.0.iter().rev()
    }
}
