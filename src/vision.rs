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
            fov.center = pos.into();
            fov.tiles.set_elements(0);

            // Update field of view.
            for (x, y, symmetric) in
                ruggle::field_of_view(&*map, pos.into(), fov.range, FovShape::CirclePlus)
            {
                if symmetric || matches!(map.get_tile(x, y), &Tile::Wall) {
                    fov.set((x, y), true);
                }
            }

            fov.dirty = false;

            // Update map seen tiles if this field of view belongs to a player.
            if players.contains(id) {
                for y in fov.center.1 - fov.range..=fov.center.1 + fov.range {
                    for x in fov.center.0 - fov.range..=fov.center.0 + fov.range {
                        if fov.get((x, y)) {
                            let idx = (y * map.width + x) as usize;
                            map.seen.set(idx, true);
                        }
                    }
                }
            }
        }
    }
}
