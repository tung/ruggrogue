use shipyard::{IntoIter, UniqueView, View, ViewMut};

use crate::{
    components::{FieldOfView, Position},
    map::{Map, Tile},
};
use ruggle::FovShape;

pub fn recalculate_fields_of_view(
    map: UniqueView<Map>,
    positions: View<Position>,
    mut fovs: ViewMut<FieldOfView>,
) {
    for (pos, mut fov) in (&positions, &mut fovs).iter() {
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
        }
    }
}
