use sdl2::keyboard::Keycode;

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

impl From<Keycode> for GameKey {
    fn from(key: Keycode) -> GameKey {
        match key {
            Keycode::Up | Keycode::K | Keycode::Kp8 => GameKey::Up,
            Keycode::Down | Keycode::J | Keycode::Kp2 => GameKey::Down,
            Keycode::Left | Keycode::H | Keycode::Kp4 => GameKey::Left,
            Keycode::Right | Keycode::L | Keycode::Kp6 => GameKey::Right,
            Keycode::Y | Keycode::Kp7 => GameKey::UpLeft,
            Keycode::U | Keycode::Kp9 => GameKey::UpRight,
            Keycode::B | Keycode::Kp1 => GameKey::DownLeft,
            Keycode::N | Keycode::Kp3 => GameKey::DownRight,
            Keycode::Period | Keycode::Kp5 | Keycode::Space => GameKey::Wait,
            Keycode::Escape => GameKey::Cancel,
            Keycode::Return => GameKey::Confirm,
            Keycode::Comma | Keycode::G => GameKey::PickUp,
            Keycode::I => GameKey::Inventory,
            _ => GameKey::Unmapped,
        }
    }
}