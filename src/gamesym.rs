use std::{collections::HashMap, path::PathBuf};

use ruggle::{Symbol, TileColor, TileIndex, TilesetInfo};

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum GameSym {
    Floor,
    WallPillar,
    WallN,
    WallE,
    WallS,
    WallW,
    WallNE,
    WallNS,
    WallNW,
    WallES,
    WallEW,
    WallSW,
    WallNES,
    WallNEW,
    WallNSW,
    WallESW,
    WallNESW,
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
    fn text_fallback(self) -> (char, (u8, u8, u8)) {
        use GameSym::*;

        match self {
            Floor => ('·', (102, 102, 102)),
            WallPillar => ('■', (179, 102, 26)),
            WallN => ('║', (179, 102, 26)),
            WallE => ('═', (179, 102, 26)),
            WallS => ('║', (179, 102, 26)),
            WallW => ('═', (179, 102, 26)),
            WallNE => ('╚', (179, 102, 26)),
            WallNS => ('║', (179, 102, 26)),
            WallNW => ('╝', (179, 102, 26)),
            WallES => ('╔', (179, 102, 26)),
            WallEW => ('═', (179, 102, 26)),
            WallSW => ('╗', (179, 102, 26)),
            WallNES => ('╠', (179, 102, 26)),
            WallNEW => ('╩', (179, 102, 26)),
            WallNSW => ('╣', (179, 102, 26)),
            WallESW => ('╦', (179, 102, 26)),
            WallNESW => ('╦', (179, 102, 26)),
            WallOther => ('#', (179, 102, 26)),
            DownStairs => ('>', (255, 255, 0)),
            Player => ('@', (255, 255, 0)),
            HealthPotion => ('!', (255, 0, 255)),
            MagicMissileScroll => ('?', (0, 255, 255)),
            FireballScroll => ('?', (255, 166, 0)),
            SleepScroll => ('?', (255, 191, 204)),
            Goblin => ('g', (128, 230, 51)),
            Orc => ('o', (230, 77, 51)),
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

    let mut symbol_map: HashMap<GameSym, (TileIndex, Option<TileColor>)> = HashMap::new();
    {
        use GameSym::*;

        symbol_map.insert(Floor, ((0, 2), Some((102, 102, 102))));
        symbol_map.insert(WallPillar, ((0, 0), Some((120, 68, 17))));
        symbol_map.insert(WallN, ((1, 0), Some((120, 68, 17))));
        symbol_map.insert(WallE, ((1, 0), Some((120, 68, 17))));
        symbol_map.insert(WallS, ((1, 0), Some((120, 68, 17))));
        symbol_map.insert(WallW, ((1, 0), Some((120, 68, 17))));
        symbol_map.insert(WallNE, ((1, 0), Some((120, 68, 17))));
        symbol_map.insert(WallNS, ((0, 0), Some((120, 68, 17))));
        symbol_map.insert(WallNW, ((1, 0), Some((120, 68, 17))));
        symbol_map.insert(WallES, ((1, 0), Some((120, 68, 17))));
        symbol_map.insert(WallEW, ((0, 0), Some((120, 68, 17))));
        symbol_map.insert(WallSW, ((1, 0), Some((120, 68, 17))));
        symbol_map.insert(WallNES, ((1, 0), Some((120, 68, 17))));
        symbol_map.insert(WallNEW, ((1, 0), Some((120, 68, 17))));
        symbol_map.insert(WallNSW, ((1, 0), Some((120, 68, 17))));
        symbol_map.insert(WallESW, ((1, 0), Some((120, 68, 17))));
        symbol_map.insert(WallNESW, ((1, 0), Some((120, 68, 17))));
        symbol_map.insert(WallOther, ((1, 0), Some((120, 68, 17))));
        symbol_map.insert(DownStairs, ((10, 0), Some((128, 128, 0))));
        symbol_map.insert(Player, ((29, 0), Some((255, 255, 0))));
        symbol_map.insert(HealthPotion, ((29, 19), Some((255, 0, 255))));
        symbol_map.insert(MagicMissileScroll, ((28, 25), Some((0, 255, 255))));
        symbol_map.insert(FireballScroll, ((28, 25), Some((255, 166, 0))));
        symbol_map.insert(SleepScroll, ((28, 25), Some((255, 191, 204))));
        symbol_map.insert(Goblin, ((41, 2), None));
        symbol_map.insert(Orc, ((26, 4), None));
    }

    TilesetInfo::<GameSym> {
        image_path: PathBuf::from("assets/urizen/urizen-onebit-tileset.png"),
        tile_size: (12, 12).into(),
        tile_start: (1, 1).into(),
        tile_gap: (1, 1).into(),
        font_map,
        symbol_map,
    }
}
