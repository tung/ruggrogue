use sdl2::{event::Event, keyboard::Keycode};
use std::collections::VecDeque;

/// Input events buffered by and emitted from an [InputBuffer].
#[derive(Clone, Copy)]
pub enum InputEvent {
    AppQuit,
    Press(Keycode),
    Release(Keycode),
}

bitflags! {
    /// Modifier key flags, tracked by an [InputBuffer] and checked via [InputBuffer::get_mods].
    pub struct KeyMods: u8 {
        /// Left Shift.
        const LSHIFT = 0b00000001;
        /// Right Shift.
        const RSHIFT = 0b00000010;
        /// Left Ctrl.
        const LCTRL = 0b00000100;
        /// Right Ctrl.
        const RCTRL = 0b00001000;
        /// Left Alt.
        const LALT = 0b00010000;
        /// Right Alt.
        const RALT = 0b00100000;

        /// Left and right Shift.
        const SHIFT = Self::LSHIFT.bits | Self::RSHIFT.bits;
        /// Left and right Ctrl.
        const CTRL = Self::LCTRL.bits | Self::RCTRL.bits;
        /// Left and right Alt.
        const ALT = Self::LALT.bits | Self::RALT.bits;
    }
}

/// An InputBuffer maintains a queue of input events that occur so that they can be handled later.
///
/// To use an InputBuffer, first create one, then call [InputBuffer::handle_event] on each event to
/// fill the InputBuffer with any input events that are detected.
///
/// To retrieve an input, call [InputBuffer::prepare_input] to pull an input event out of the
/// queue, then use [InputBuffer::get_input] to get the input itself.
///
/// At the end of each main loop iteration, call [InputBuffer::clear_input] to make way for the
/// next input.  Calling prepare_input multiple times without clear_input does nothing, ensuring
/// that inputs won't just vanish mid-iteration.
///
/// To detect if more inputs are queued up beyond any prepared inputs, call
/// [InputBuffer::more_inputs].
///
/// As long as handle_event is called, inputs will be buffered.  If these buffered inputs aren't
/// needed, calling [InputBuffer::flush_all_inputs] will clear them all.
pub struct InputBuffer {
    buffer: VecDeque<InputEvent>,
    current_input: Option<InputEvent>,
    keymods: KeyMods,
}

impl Default for InputBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl InputBuffer {
    /// Create a new InputBuffer that can queue input events.
    pub fn new() -> InputBuffer {
        InputBuffer {
            buffer: VecDeque::new(),
            current_input: None,
            keymods: KeyMods::empty(),
        }
    }

    /// Check if an event is a relevant input event and buffer it if so.
    pub fn handle_event(&mut self, event: &sdl2::event::Event) {
        match event {
            Event::KeyDown {
                keycode: Some(key), ..
            } => self.buffer.push_back(InputEvent::Press(*key)),
            Event::KeyUp {
                keycode: Some(key), ..
            } => self.buffer.push_back(InputEvent::Release(*key)),
            Event::Quit { .. } => self.buffer.push_back(InputEvent::AppQuit),
            _ => {}
        }
    }

    /// If no event is prepared, set current input event to the next one in the buffer.
    /// If an event is already prepared, do nothing.
    pub fn prepare_input(&mut self) {
        if self.current_input.is_none() && !self.buffer.is_empty() {
            self.current_input = self.buffer.pop_front();

            // Track modifier keys.
            if let Some(input) = self.current_input {
                match input {
                    InputEvent::Press(keycode) => match keycode {
                        Keycode::LShift => self.keymods |= KeyMods::LSHIFT,
                        Keycode::RShift => self.keymods |= KeyMods::RSHIFT,
                        Keycode::LCtrl => self.keymods |= KeyMods::LCTRL,
                        Keycode::RCtrl => self.keymods |= KeyMods::RCTRL,
                        Keycode::LAlt => self.keymods |= KeyMods::LALT,
                        Keycode::RAlt => self.keymods |= KeyMods::RALT,
                        _ => {}
                    },
                    InputEvent::Release(keycode) => match keycode {
                        Keycode::LShift => self.keymods &= !KeyMods::LSHIFT,
                        Keycode::RShift => self.keymods &= !KeyMods::RSHIFT,
                        Keycode::LCtrl => self.keymods &= !KeyMods::LCTRL,
                        Keycode::RCtrl => self.keymods &= !KeyMods::RCTRL,
                        Keycode::LAlt => self.keymods &= !KeyMods::LALT,
                        Keycode::RAlt => self.keymods &= !KeyMods::RALT,
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
    }

    /// Get the current input event.
    pub fn get_input(&self) -> Option<InputEvent> {
        self.current_input
    }

    /// Get modifier keys that were active when the current input event was received.
    pub fn get_mods(&self, mods: KeyMods) -> bool {
        !(self.keymods & mods).is_empty()
    }

    /// Clear the current input event.
    pub fn clear_input(&mut self) {
        self.current_input = None;
    }

    /// Clear all buffered input events.
    pub fn flush_all_inputs(&mut self) {
        self.buffer.clear();
        self.current_input = None;
    }

    /// Returns true if there are more input events buffered beyond the current input.
    pub fn more_inputs(&self) -> bool {
        !self.buffer.is_empty()
    }
}
