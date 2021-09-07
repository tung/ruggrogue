use shipyard::{IntoIter, Shiperator, UniqueViewMut, View, ViewMut};

use crate::{
    components::{Coord, FieldOfView, Player},
    map::{Map, Tile},
};
use ruggrogue::FovShape;

pub fn recalculate_fields_of_view(
    mut map: UniqueViewMut<Map>,
    coords: View<Coord>,
    mut fovs: ViewMut<FieldOfView>,
    players: View<Player>,
) {
    for (id, (coord, mut fov)) in (&coords, &mut fovs).iter().with_id() {
        if fov.dirty {
            fov.center = coord.0.into();
            fov.tiles.zero_out_bits();

            // Update field of view.
            for (x, y, symmetric) in
                ruggrogue::field_of_view(&*map, coord.0.into(), fov.range, FovShape::CirclePlus)
            {
                if symmetric || matches!(map.get_tile(x, y), &Tile::Wall) {
                    fov.set((x, y), true);
                }
            }

            fov.dirty = false;

            // Update map seen tiles if this field of view belongs to a player.
            if players.contains(id) {
                fov.mark_seen(&mut map.seen);
            }
        }
    }
}
