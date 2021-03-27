use std::{collections::HashMap, path::PathBuf};

use ruggle::{Symbol, TilesetInfo};

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum GameSym {
    Floor,
    WallPillar,
    WallN,
    WallE,
    WallS,
    WallW,
    WallNe,
    WallNs,
    WallNw,
    WallEs,
    WallEw,
    WallSw,
    WallNes,
    WallNew,
    WallNsw,
    WallEsw,
    WallNesw,
    WallOther,
    DownStairs,
    Player,
    HealthPotion,
    MagicMissileScroll,
    FireballScroll,
    SleepScroll,
    Goblin,
    Orc,
}

impl Symbol for GameSym {
    fn text_fallback(self) -> char {
        use GameSym::*;

        match self {
            Floor => '·',
            WallPillar => '■',
            WallN => '║',
            WallE => '═',
            WallS => '║',
            WallW => '═',
            WallNe => '╚',
            WallNs => '║',
            WallNw => '╝',
            WallEs => '╔',
            WallEw => '═',
            WallSw => '╗',
            WallNes => '╠',
            WallNew => '╩',
            WallNsw => '╣',
            WallEsw => '╦',
            WallNesw => '╦',
            WallOther => '#',
            DownStairs => '>',
            Player => '@',
            HealthPotion => '!',
            MagicMissileScroll => '?',
            FireballScroll => '?',
            SleepScroll => '?',
            Goblin => 'g',
            Orc => 'o',
        }
    }
}

pub fn urizen_tileset_info() -> TilesetInfo<GameSym> {
    let mut font_map: HashMap<char, (i32, i32)> = HashMap::new();
    {
        for (i, ch) in ('A'..='T').enumerate() {
            font_map.insert(ch, (i as i32, 44));
        }
        for (i, ch) in ('1'..='5').enumerate() {
            font_map.insert(ch, (21 + i as i32, 44));
        }

        for (i, ch) in ('U'..='Z').enumerate() {
            font_map.insert(ch, (i as i32, 45));
        }
        for (i, ch) in ('a'..='n').enumerate() {
            font_map.insert(ch, (6 + i as i32, 45));
        }
        for (i, ch) in ('6'..='9').enumerate() {
            font_map.insert(ch, (21 + i as i32, 45));
        }
        font_map.insert('0', (25, 45));

        for (i, ch) in ('o'..='z').enumerate() {
            font_map.insert(ch, (i as i32, 46));
        }
        font_map.insert('(', (12, 46));
        font_map.insert(')', (13, 46));
        font_map.insert('[', (14, 46));
        font_map.insert(']', (15, 46));
        font_map.insert('{', (16, 46));
        font_map.insert('}', (17, 46));
        font_map.insert('<', (18, 46));
        font_map.insert('>', (19, 46));
        font_map.insert('+', (20, 46));
        font_map.insert('-', (21, 46));
        font_map.insert('?', (22, 46));
        font_map.insert('!', (23, 46));
        font_map.insert('^', (24, 46));

        font_map.insert(':', (0, 47));
        font_map.insert('#', (1, 47));
        font_map.insert('_', (2, 47));
        font_map.insert('@', (3, 47));
        font_map.insert('%', (4, 47));
        font_map.insert('~', (5, 47));
        font_map.insert('$', (6, 47));
        font_map.insert('"', (7, 47));
        font_map.insert('\'', (8, 47));
        font_map.insert('&', (9, 47));
        font_map.insert('*', (10, 47));
        font_map.insert('=', (11, 47));
        font_map.insert('`', (12, 47));
        font_map.insert('|', (13, 47));
        font_map.insert('/', (14, 47));
        font_map.insert('\\', (15, 47));
        font_map.insert('.', (16, 47));
        font_map.insert(',', (17, 47));
        font_map.insert(';', (18, 47));
    }

    let mut symbol_map: HashMap<GameSym, (i32, i32)> = HashMap::new();
    {
        use GameSym::*;

        symbol_map.insert(Floor, (0, 2));
        symbol_map.insert(WallPillar, (0, 0));
        symbol_map.insert(WallN, (1, 0));
        symbol_map.insert(WallE, (1, 0));
        symbol_map.insert(WallS, (1, 0));
        symbol_map.insert(WallW, (1, 0));
        symbol_map.insert(WallNe, (1, 0));
        symbol_map.insert(WallNs, (0, 0));
        symbol_map.insert(WallNw, (1, 0));
        symbol_map.insert(WallEs, (1, 0));
        symbol_map.insert(WallEw, (0, 0));
        symbol_map.insert(WallSw, (1, 0));
        symbol_map.insert(WallNes, (1, 0));
        symbol_map.insert(WallNew, (1, 0));
        symbol_map.insert(WallNsw, (1, 0));
        symbol_map.insert(WallEsw, (1, 0));
        symbol_map.insert(WallNesw, (1, 0));
        symbol_map.insert(WallOther, (1, 0));
        symbol_map.insert(DownStairs, (10, 0));
        symbol_map.insert(Player, (29, 0));
        symbol_map.insert(HealthPotion, (29, 19));
        symbol_map.insert(MagicMissileScroll, (28, 25));
        symbol_map.insert(FireballScroll, (28, 25));
        symbol_map.insert(SleepScroll, (28, 25));
        symbol_map.insert(Goblin, (41, 2));
        symbol_map.insert(Orc, (26, 4));
    }

    TilesetInfo::<GameSym> {
        image_path: PathBuf::from("assets/urizen/urizen-onebit-tileset-mono.png"),
        tile_size: (12, 12).into(),
        tile_start: (1, 1).into(),
        tile_gap: (1, 1).into(),
        font_map,
        symbol_map,
    }
}
