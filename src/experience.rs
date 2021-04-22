use shipyard::{Get, IntoIter, Shiperator, UniqueView, UniqueViewMut, View, ViewMut};

use crate::{
    components::{CombatStats, Experience, Name},
    message::Messages,
    player::PlayerId,
};

pub fn gain_levels(
    mut msgs: UniqueViewMut<Messages>,
    player_id: UniqueView<PlayerId>,
    mut combat_stats: ViewMut<CombatStats>,
    mut exps: ViewMut<Experience>,
    names: View<Name>,
) {
    for (id, exp) in (&mut exps).iter().with_id() {
        if exp.next > 0 {
            while exp.exp >= exp.next {
                exp.level += 1;
                exp.exp -= exp.next;
                exp.base += exp.next;
                exp.next = exp.next * 6 / 5;

                if let Ok(stats) = (&mut combat_stats).try_get(id) {
                    let hp_gain = (stats.max_hp / 10).max(1);

                    stats.max_hp += hp_gain;
                    stats.hp += hp_gain;

                    if id == player_id.0 {
                        msgs.add(format!("{} is now level {}!", &names.get(id).0, exp.level));
                    }
                }
            }
        }
    }
}
