use shipyard::{IntoIter, Shiperator, UniqueViewMut, View, ViewMut};

use crate::{
    components::{FieldOfView, Player, Position},
    map::{Map, Tile},
};
use ruggle::FovShape;

pub fn recalculate_fields_of_view(
    mut map: UniqueViewMut<Map>,
    positions: View<Position>,
    mut fovs: ViewMut<FieldOfView>,
    players: View<Player>,
) {
    for (id, (pos, mut fov)) in (&positions, &mut fovs).iter().with_id() {
        if fov.dirty {
            // Mark visible tiles with this boolean.
            let new_mark = !fov.mark;

            // Update field of view.
            for (x, y, symmetric) in
                ruggle::field_of_view(&*map, pos.into(), fov.range, FovShape::CirclePlus)
            {
                if symmetric || matches!(map.get_tile(x, y), &Tile::Wall) {
                    fov.tiles
                        .entry((x, y))
                        .and_modify(|e| *e = new_mark)
                        .or_insert(new_mark);
                }
            }

            // Sweep out stale tiles.
            fov.tiles.retain(|_, v| *v == new_mark);

            fov.mark = new_mark;
            fov.dirty = false;

            // Update map seen tiles if this field of view belongs to a player.
            if players.contains(id) {
                for (x, y) in fov.tiles.keys() {
                    let idx = (y * map.width + x) as usize;
                    map.seen.set(idx, true);
                }
            }
        }
    }
}
