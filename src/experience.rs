use rand::Rng;
use serde::{Deserialize, Serialize};
use shipyard::{EntityId, Get, IntoIter, Shiperator, UniqueView, UniqueViewMut, View, ViewMut};

use crate::{
    components::{CombatStats, Experience, GivesExperience, Monster, Name, Player},
    message::Messages,
    player::PlayerId,
};

/// Tracking state that counts total amount of experience points that could be gained at the time
/// of entering a new dungeon depth, used to determine the approximate level that monsters and
/// items on the current dungeon floor should be based around.
#[derive(Deserialize, Serialize)]
pub struct Difficulty {
    pub id: EntityId,
    exp_for_next_depth: u64,
}

impl Difficulty {
    /// Create a new instance of difficulty tracking state.
    ///
    /// `id` should be provisioned from [spawn::spawn_difficulty], which creates a component with
    /// an [components::Experience] component that is used to help calculate difficulty.
    pub fn new(id: EntityId) -> Self {
        Self {
            id,
            exp_for_next_depth: 0,
        }
    }

    /// Replace this instance of Difficulty with another instance.
    pub fn replace(&mut self, replacement: Self) {
        self.id = replacement.id;
        self.exp_for_next_depth = replacement.exp_for_next_depth;
    }

    /// Get the level tracked by difficulty with a fractional part based on experience.
    pub fn as_f32(&self, exps: &View<Experience>) -> f32 {
        let difficulty_exp = exps.get(self.id);
        difficulty_exp.level as f32 + difficulty_exp.exp as f32 / difficulty_exp.next as f32
    }

    /// Get the level tracked by difficulty, with a random chance of being the next level up based
    /// on experience progress.
    pub fn get_round_random<R: Rng>(&self, exps: &View<Experience>, rng: &mut R) -> i32 {
        let difficulty_exp = exps.get(self.id);

        if difficulty_exp.next > 0 && rng.gen_range(0u64..difficulty_exp.next) < difficulty_exp.exp
        {
            difficulty_exp.level + 1
        } else {
            difficulty_exp.level
        }
    }
}

/// Round with a random chance of rounding upwards based on the fractional part of the value.
pub fn f32_round_random<R: Rng>(value: f32, rng: &mut R) -> i32 {
    value.trunc() as i32
        + if rng.gen::<f32>() < value.fract() {
            1
        } else {
            0
        }
}

/// Count the experience provided by all monsters currently on the map, to be redeemed later.
///
/// This should run just after the map has been populated by monsters.
pub fn calc_exp_for_next_depth(
    mut difficulty: UniqueViewMut<Difficulty>,
    monsters: View<Monster>,
    gives_exps: View<GivesExperience>,
) {
    for (_, gives_exp) in (&monsters, &gives_exps).iter() {
        difficulty.exp_for_next_depth += gives_exp.0;
    }
}

/// Add the experience previously buffered by [calc_exp_for_next_depth] and reset the buffer.
///
/// Run [gain_levels] after this to properly calculate difficulty, then proceed with generating a
/// new dungeon level afterwards to take advantage of difficulty tracking using
/// [Difficulty::into_f32] and [Difficulty::get_round_random].
pub fn redeem_exp_for_next_depth(
    mut difficulty: UniqueViewMut<Difficulty>,
    mut exps: ViewMut<Experience>,
) {
    let difficulty_exp = (&mut exps).get(difficulty.id);

    difficulty_exp.exp += difficulty.exp_for_next_depth;
    difficulty.exp_for_next_depth = 0;
}

/// The factor around which all combat stats are scaled around.
fn level_factor(level: i32) -> f32 {
    (1.0 + (level - 1) as f32 * 0.1).max(0.1)
}

pub fn calc_player_max_hp(level: i32) -> i32 {
    (level_factor(level) * 30.0).round() as i32
}

pub fn calc_player_attack(level: i32) -> f32 {
    level_factor(level) * 4.8
}

pub fn calc_player_defense(level: i32) -> f32 {
    level_factor(level) * 2.4
}

pub fn calc_monster_max_hp(level: i32) -> i32 {
    (level_factor(level) * 14.0).round() as i32
}

pub fn calc_monster_attack(level: i32) -> f32 {
    level_factor(level) * 8.0
}

pub fn calc_monster_defense(level: i32) -> f32 {
    level_factor(level) * 4.0
}

pub fn calc_monster_exp(level: i32) -> u64 {
    (level_factor(level) * 10.0).ceil() as u64
}

pub fn calc_weapon_attack(level: i32) -> f32 {
    level_factor(level) * 3.2
}

pub fn calc_armor_defense(level: i32) -> f32 {
    level_factor(level) * 1.6
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
                exp.next = exp.next * 11 / 10;

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
                    stats.hp = stats.max_hp;
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
