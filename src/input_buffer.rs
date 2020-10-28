use piston::{Button, GenericEvent};

#[derive(Clone, Copy)]
pub enum InputEvent {
    Nothing,
    Press(Button),
}

/// An InputBuffer maintains a queue of input events that occur so that they can be handled later.
///
/// To use an InputBuffer, first create one with the desired maximum number of input events to
/// buffer.
///
/// Call handle_event on each event to fill the InputBuffer with any input events that are
/// detected.
///
/// To retrieve an input, call prepare_input to pull an input event out of the queue, then use
/// get_input to get the input itself.
///
/// At the end of each main loop iteration, call clear_input to make way for the next input.
/// Calling prepare_input multiple times without clear_input does nothing, ensuring that inputs
/// won't just vanish mid-iteration.
///
/// To detect if more inputs are queued up beyond any prepared inputs, call more_inputs.
///
/// As long as handle_event is called, inputs will be buffered.  If these buffered inputs aren't
/// needed, calling flush_all_inputs will clear them all.
pub struct InputBuffer {
    buffer: Vec<InputEvent>,
    capacity: usize,
    head: usize,
    len: usize,
    current_input: Option<InputEvent>,
}

impl InputBuffer {
    /// Create a new InputBuffer that can queue a number of input events up to capacity.
    pub fn new(capacity: usize) -> InputBuffer {
        assert!(capacity > 0);

        InputBuffer {
            buffer: vec![InputEvent::Nothing; capacity],
            capacity,
            head: 0,
            len: 0,
            current_input: None,
        }
    }

    /// Add an input event to the buffer.
    /// If the buffer is full, replace the most recent event instead.
    fn queue_input_event(&mut self, ie: InputEvent) {
        // Append to the buffer, or replace the last element if full.
        if self.len < self.capacity {
            self.len += 1;
        }

        let pos = (self.head + self.len - 1) % self.capacity;

        self.buffer[pos] = ie;
    }

    /// Check if an event is a relevant input event and buffer it if so.
    pub fn handle_event<E: GenericEvent>(&mut self, e: &E) {
        if let Some(args) = e.press_args() {
            self.queue_input_event(InputEvent::Press(args));
        }
    }

    /// If no event is prepared, set current input event to the next one in the buffer.
    /// If an event is already prepared, do nothing.
    pub fn prepare_input(&mut self) {
        if self.current_input.is_none() && self.len > 0 {
            // Pop an event out of the buffer.
            self.current_input = Some(self.buffer[self.head]);
            self.len -= 1;
            self.head += 1;
            if self.head >= self.capacity {
                self.head = 0;
            }
        }
    }

    /// Get the current input event.
    pub fn get_input(&self) -> Option<InputEvent> {
        self.current_input
    }

    /// Clear the current input event.
    pub fn clear_input(&mut self) {
        self.current_input = None;
    }

    /// Clear all buffered input events.
    pub fn flush_all_inputs(&mut self) {
        self.len = 0;
        self.current_input = None;
    }

    /// Returns true if there are more input events buffered beyond the current input.
    pub fn more_inputs(&self) -> bool {
        self.len > 0
    }
}
