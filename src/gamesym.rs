use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

use ruggrogue::{Symbol, TilesetInfo};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
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
    Ration,
    HealthPotion,
    MagicMissileScroll,
    FireballScroll,
    SleepScroll,
    Knife,
    Club,
    Hatchet,
    Spear,
    Rapier,
    Saber,
    Longsword,
    Crowbar,
    Tonfa,
    BeamSword,
    Jerkin,
    Coat,
    WoodenShield,
    TowerShield,
    KiteShield,
    StuddedArmor,
    Hauberk,
    Platemail,
    ArmyHelmet,
    FlakJacket,
    Present,
    Blob,
    Bat,
    Crab,
    Snake,
    Goblin,
    Kobold,
    Gnome,
    Orc,
    Unicorn,
    Pirate,
    Lizardman,
    Ghost,
    Skeleton,
    Ogre,
    Naga,
    Warlock,
    Demon,
    Sentinel,
    Robber,
    SkateboardKid,
    Jellybean,
    Alien,
    Dweller,
    LittleHelper,
    BigHelper,
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
            WallNesw => '╬',
            WallOther => '#',
            DownStairs => '>',
            Player => '@',
            Ration => '%',
            HealthPotion => '!',
            MagicMissileScroll => '?',
            FireballScroll => '?',
            SleepScroll => '?',
            Knife => ')',
            Club => ')',
            Hatchet => ')',
            Spear => ')',
            Rapier => ')',
            Saber => ')',
            Longsword => ')',
            Crowbar => ')',
            Tonfa => ')',
            BeamSword => ')',
            Jerkin => '[',
            Coat => '[',
            WoodenShield => '[',
            TowerShield => '[',
            KiteShield => '[',
            StuddedArmor => '[',
            Hauberk => '[',
            Platemail => '[',
            ArmyHelmet => '[',
            FlakJacket => '[',
            Present => '$',
            Blob => 'b',
            Bat => 'B',
            Crab => 'c',
            Snake => 'S',
            Goblin => 'g',
            Kobold => 'k',
            Gnome => 'G',
            Orc => 'o',
            Unicorn => 'u',
            Pirate => 'P',
            Lizardman => 'L',
            Ghost => 'G',
            Skeleton => 'Z',
            Ogre => 'O',
            Naga => 'N',
            Warlock => 'W',
            Demon => '&',
            Sentinel => 'E',
            Robber => 'R',
            SkateboardKid => 'K',
            Jellybean => 'J',
            Alien => 'A',
            Dweller => 'D',
            LittleHelper => 'h',
            BigHelper => 'H',
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
        symbol_map.insert(Ration, (5, 16));
        symbol_map.insert(HealthPotion, (29, 19));
        symbol_map.insert(MagicMissileScroll, (28, 25));
        symbol_map.insert(FireballScroll, (28, 25));
        symbol_map.insert(SleepScroll, (28, 25));
        symbol_map.insert(Knife, (14, 23));
        symbol_map.insert(Club, (37, 21));
        symbol_map.insert(Hatchet, (42, 21));
        symbol_map.insert(Spear, (38, 21));
        symbol_map.insert(Rapier, (30, 21));
        symbol_map.insert(Saber, (34, 21));
        symbol_map.insert(Longsword, (26, 21));
        symbol_map.insert(Crowbar, (33, 45));
        symbol_map.insert(Tonfa, (43, 42));
        symbol_map.insert(BeamSword, (6, 38));
        symbol_map.insert(Jerkin, (12, 22));
        symbol_map.insert(Coat, (0, 22));
        symbol_map.insert(WoodenShield, (27, 23));
        symbol_map.insert(TowerShield, (26, 23));
        symbol_map.insert(KiteShield, (25, 23));
        symbol_map.insert(StuddedArmor, (33, 23));
        symbol_map.insert(Hauberk, (32, 23));
        symbol_map.insert(Platemail, (31, 23));
        symbol_map.insert(ArmyHelmet, (33, 43));
        symbol_map.insert(FlakJacket, (34, 43));
        symbol_map.insert(Present, (27, 30));
        symbol_map.insert(Blob, (39, 10));
        symbol_map.insert(Bat, (8, 13));
        symbol_map.insert(Crab, (7, 13));
        symbol_map.insert(Snake, (2, 13));
        symbol_map.insert(Goblin, (41, 2));
        symbol_map.insert(Kobold, (42, 4));
        symbol_map.insert(Gnome, (28, 8));
        symbol_map.insert(Orc, (26, 4));
        symbol_map.insert(Unicorn, (38, 10));
        symbol_map.insert(Pirate, (25, 10));
        symbol_map.insert(Lizardman, (33, 6));
        symbol_map.insert(Ghost, (43, 10));
        symbol_map.insert(Skeleton, (41, 5));
        symbol_map.insert(Ogre, (33, 10));
        symbol_map.insert(Naga, (41, 6));
        symbol_map.insert(Warlock, (39, 6));
        symbol_map.insert(Demon, (38, 2));
        symbol_map.insert(Sentinel, (42, 10));
        symbol_map.insert(Robber, (40, 39));
        symbol_map.insert(SkateboardKid, (42, 39));
        symbol_map.insert(Jellybean, (15, 37));
        symbol_map.insert(Alien, (0, 37));
        symbol_map.insert(Dweller, (1, 41));
        symbol_map.insert(LittleHelper, (26, 30));
        symbol_map.insert(BigHelper, (25, 30));
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
