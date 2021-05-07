use shipyard::{Get, IntoIter, Shiperator, UniqueView, UniqueViewMut, View, ViewMut};

use crate::{
    components::{CombatStats, Experience, Name, Player},
    message::Messages,
    player::PlayerId,
};

/// The factor around which all combat stats are scaled around.
fn level_factor(level: i32) -> f32 {
    (1.0 + (level - 1) as f32 * 0.1).max(0.1)
}

pub fn calc_player_max_hp(level: i32) -> i32 {
    (level_factor(level) * 30.0).round() as i32
}

pub fn calc_player_attack(level: i32) -> f32 {
    level_factor(level) * 3.0
}

pub fn calc_player_defense(level: i32) -> f32 {
    level_factor(level) * 1.2
}

pub fn calc_monster_max_hp(level: i32) -> i32 {
    (level_factor(level) * 15.0).round() as i32
}

pub fn calc_monster_attack(level: i32) -> f32 {
    level_factor(level) * 4.0
}

pub fn calc_monster_defense(level: i32) -> f32 {
    level_factor(level)
}

pub fn calc_weapon_attack(level: i32) -> f32 {
    level_factor(level) * 2.0
}

pub fn calc_armor_defense(level: i32) -> f32 {
    level_factor(level) * 0.8
}

pub fn gain_levels(
    mut msgs: UniqueViewMut<Messages>,
    player_id: UniqueView<PlayerId>,
    mut combat_stats: ViewMut<CombatStats>,
    mut exps: ViewMut<Experience>,
    names: View<Name>,
    players: View<Player>,
) {
    for (id, exp) in (&mut exps).iter().with_id() {
        if exp.next > 0 {
            while exp.exp >= exp.next {
                exp.level += 1;
                exp.exp -= exp.next;
                exp.base += exp.next;
                exp.next = exp.next * 6 / 5;

                if let Ok(stats) = (&mut combat_stats).try_get(id) {
                    let hp_gain;
                    let new_attack;
                    let new_defense;

                    if players.contains(id) {
                        hp_gain = calc_player_max_hp(exp.level) - calc_player_max_hp(exp.level - 1);
                        new_attack = calc_player_attack(exp.level);
                        new_defense = calc_player_defense(exp.level);
                    } else {
                        hp_gain =
                            calc_monster_max_hp(exp.level) - calc_monster_max_hp(exp.level - 1);
                        new_attack = calc_monster_attack(exp.level);
                        new_defense = calc_monster_defense(exp.level);
                    }

                    stats.max_hp += hp_gain;
                    stats.hp += hp_gain;
                    stats.attack = new_attack;
                    stats.defense = new_defense;

                    if id == player_id.0 {
                        msgs.add(format!("{} is now level {}!", &names.get(id).0, exp.level));
                    }
                }
            }
        }
    }
}
