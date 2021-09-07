use std::ops::{Index, IndexMut};

use ruggrogue::util::Position;

pub struct MenuMemory {
    menu: [i32; 7],
    pub pick_up_pos: Position,
}

impl MenuMemory {
    pub const INVENTORY: usize = 0;
    pub const INVENTORY_SHORTCUT_EQUIP: usize = 1;
    pub const INVENTORY_SHORTCUT_USE: usize = 2;
    pub const INVENTORY_SHORTCUT_DROP: usize = 3;
    pub const EQUIPMENT_SHORTCUT_REMOVE: usize = 4;
    pub const EQUIPMENT_SHORTCUT_DROP: usize = 5;
    pub const PICK_UP: usize = 6;

    pub fn new() -> Self {
        Self {
            menu: [0; 7],
            pick_up_pos: Position { x: 0, y: 0 },
        }
    }

    pub fn reset(&mut self) {
        for m in self.menu.iter_mut() {
            *m = 0;
        }
        self.pick_up_pos = Position { x: 0, y: 0 };
    }
}

impl Index<usize> for MenuMemory {
    type Output = i32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.menu[index]
    }
}

impl IndexMut<usize> for MenuMemory {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.menu[index]
    }
}
