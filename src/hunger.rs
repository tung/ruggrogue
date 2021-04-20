use shipyard::{
    EntityId, Get, IntoIter, Shiperator, UniqueView, UniqueViewMut, View, ViewMut, World,
};

use crate::{
    components::{CombatStats, Name, Player, Stomach},
    message::Messages,
    player::PlayerId,
};
use ruggle::util::Color;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum CanRegenResult {
    CanRegen,
    NoRegen,
    FullyRested,
    TooHungry,
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum HungerState {
    Starving,
    Famished,
    VeryHungry,
    Hungry,
    Normal,
    Full,
}

// "Very Hungry"
pub const MAX_HUNGER_WIDTH: usize = 11;

impl HungerState {
    fn reduced_to(&self) -> &'static str {
        match self {
            HungerState::Starving => "starving!",
            HungerState::Famished => "famished!",
            HungerState::VeryHungry => "getting very hungry.",
            HungerState::Hungry => "getting hungry.",
            HungerState::Normal => "no longer full.",
            HungerState::Full => "full.",
        }
    }

    fn turns_to_regen_to_max_hp(&self) -> Option<i32> {
        match self {
            HungerState::Starving => None,
            HungerState::Famished => None,
            HungerState::VeryHungry => None,
            HungerState::Hungry => Some(400),
            HungerState::Normal => Some(400),
            HungerState::Full => Some(200),
        }
    }

    fn turns_to_starve_from_max_hp(&self) -> Option<i32> {
        if matches!(self, HungerState::Starving) {
            Some(400)
        } else {
            None
        }
    }
}

impl From<i32> for HungerState {
    fn from(fullness: i32) -> Self {
        if fullness <= 0 {
            Self::Starving
        } else if fullness <= 100 {
            Self::Famished
        } else if fullness <= 200 {
            Self::VeryHungry
        } else if fullness <= 500 {
            Self::Hungry
        } else if fullness <= 800 {
            Self::Normal
        } else {
            Self::Full
        }
    }
}

/// Checks if the given entity can or cannot regenerate with a specific reason.
pub fn can_regen(world: &World, entity_id: EntityId) -> CanRegenResult {
    let (combat_stats, stomachs) = world.borrow::<(View<CombatStats>, View<Stomach>)>();
    let stats = if let Ok(stats) = combat_stats.try_get(entity_id) {
        stats
    } else {
        return CanRegenResult::NoRegen;
    };
    let stomach = if let Ok(stomach) = stomachs.try_get(entity_id) {
        stomach
    } else {
        return CanRegenResult::NoRegen;
    };

    if stats.hp >= stats.max_hp {
        return CanRegenResult::FullyRested;
    }

    if HungerState::from(stomach.fullness)
        .turns_to_regen_to_max_hp()
        .is_none()
    {
        return CanRegenResult::TooHungry;
    }

    CanRegenResult::CanRegen
}

/// Get a description for the player's hunger level to show in the UI, with foreground and
/// background colors.
pub fn player_hunger_label(
    player_id: UniqueView<PlayerId>,
    stomachs: View<Stomach>,
) -> (&'static str, Color, Color) {
    if let Ok(stomach) = stomachs.try_get(player_id.0) {
        match HungerState::from(stomach.fullness) {
            HungerState::Starving => ("Starving", Color::BLACK, Color::ORANGE),
            HungerState::Famished => ("Famished", Color::ORANGE, Color::BLACK),
            HungerState::VeryHungry => ("Very Hungry", Color::YELLOW, Color::BLACK),
            HungerState::Hungry => ("Hungry", Color::YELLOW, Color::BLACK),
            HungerState::Normal => ("Normal", Color::GRAY, Color::BLACK),
            HungerState::Full => ("Full", Color::GREEN, Color::BLACK),
        }
    } else {
        ("", Color::WHITE, Color::BLACK)
    }
}

/// Perform per-turn hunger effects like emptying stomachs, regeneration and starvation.
pub fn tick_hunger(
    mut msgs: UniqueViewMut<Messages>,
    player_id: UniqueView<PlayerId>,
    mut combat_stats: ViewMut<CombatStats>,
    names: View<Name>,
    mut players: ViewMut<Player>,
    mut stomachs: ViewMut<Stomach>,
) {
    for (id, stomach) in (&mut stomachs).iter().with_id() {
        let name = names.get(id);

        if stomach.fullness > 0 {
            let old_hunger = HungerState::from(stomach.fullness);
            stomach.fullness -= 1;

            if let Ok(stats) = (&mut combat_stats).try_get(id) {
                if stats.hp > 0 {
                    // Regenerate hit points if below max and stomach allows it.
                    if stats.hp < stats.max_hp && stomach.fullness > 0 {
                        if let Some(regen_turns) =
                            HungerState::from(stomach.fullness).turns_to_regen_to_max_hp()
                        {
                            // Regeneration costs extra hunger.
                            stomach.fullness -= 1;

                            // Track a partial hit point in stomach and grant it to stats when it
                            // exceeds regen_turns.
                            stomach.sub_hp += stats.max_hp;
                            if stomach.sub_hp >= regen_turns && regen_turns > 0 {
                                let amount = stomach.sub_hp / regen_turns;
                                stats.hp = (stats.hp + amount).min(stats.max_hp);
                                stomach.sub_hp -= regen_turns * amount;
                            }
                        }
                    }
                }
            }

            let new_hunger = HungerState::from(stomach.fullness);
            if new_hunger != old_hunger {
                // Stop auto-run when hunger state changes.
                if let Ok(player) = (&mut players).try_get(id) {
                    player.auto_run = None;
                }

                // Tell the player when their hunger state changes.
                if id == player_id.0 {
                    msgs.add(format!("{} is {}", &name.0, new_hunger.reduced_to()));
                }
            }
        }

        if let Ok(stats) = (&mut combat_stats).try_get(id) {
            if stats.hp > 0 {
                // Inflict damage if stomach is starving.
                if let Some(starve_turns) =
                    HungerState::from(stomach.fullness).turns_to_starve_from_max_hp()
                {
                    // Track partial hit point reduction in stomach and deduct it from stats when
                    // it exceeds starve_turns.
                    stomach.sub_hp -= stats.max_hp;
                    if -stomach.sub_hp >= starve_turns && starve_turns > 0 {
                        let amount = -stomach.sub_hp / starve_turns;
                        stats.hp = (stats.hp - amount).max(0);
                        stomach.sub_hp += starve_turns * amount;

                        // Stop auto-run when taking damage from starvation.
                        if let Ok(player) = (&mut players).try_get(id) {
                            player.auto_run = None;
                        }

                        // Tell the player when they take damage from starvation.
                        if id == player_id.0 {
                            msgs.add(format!("{} aches with hunger!", &name.0));
                        }
                    }
                }
            }
        }
    }
}