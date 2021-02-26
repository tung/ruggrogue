use piston::input::Key;

pub enum GameKey {
    Unmapped,
    Up,
    Down,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
    Wait,
    Cancel,
    Confirm,
    PickUp,
    Inventory,
}

impl From<Key> for GameKey {
    fn from(key: Key) -> GameKey {
        match key {
            Key::Up | Key::K | Key::NumPad8 => GameKey::Up,
            Key::Down | Key::J | Key::NumPad2 => GameKey::Down,
            Key::Left | Key::H | Key::NumPad4 => GameKey::Left,
            Key::Right | Key::L | Key::NumPad6 => GameKey::Right,
            Key::Y | Key::NumPad7 => GameKey::UpLeft,
            Key::U | Key::NumPad9 => GameKey::UpRight,
            Key::B | Key::NumPad1 => GameKey::DownLeft,
            Key::N | Key::NumPad3 => GameKey::DownRight,
            Key::Period | Key::NumPad5 | Key::Space => GameKey::Wait,
            Key::Escape => GameKey::Cancel,
            Key::Return => GameKey::Confirm,
            Key::Comma | Key::G => GameKey::PickUp,
            Key::I => GameKey::Inventory,
            _ => GameKey::Unmapped,
        }
    }
}
