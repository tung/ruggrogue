use shipyard::{Get, IntoIter, UniqueView, View, World};

use crate::{
    components::{Coord, FieldOfView, RenderOnFloor, RenderOnMap, Renderable},
    gamesym::GameSym,
    map::Map,
    player::PlayerId,
};
use ruggle::{
    util::{Color, Position},
    TileGrid,
};

#[allow(clippy::many_single_char_names)]
pub fn draw_map(world: &World, grid: &mut TileGrid<GameSym>) {
    let (map, player_id, coords, fovs) = world.borrow::<(
        UniqueView<Map>,
        UniqueView<PlayerId>,
        View<Coord>,
        View<FieldOfView>,
    )>();

    let player_pos = coords.get(player_id.0).0;
    let fov = fovs.get(player_id.0);
    let w = grid.width() as i32;
    let h = grid.height() as i32;
    let cx = w / 2;
    let cy = h / 2;

    grid.set_draw_offset(player_pos);

    for (tx, ty, tile) in map.iter_bounds(
        player_pos.x - cx,
        player_pos.y - cy,
        player_pos.x - cx + w - 1,
        player_pos.y - cy + h - 1,
    ) {
        if let Some((sym, color)) = tile {
            let color = if fov.get((tx, ty)) {
                color
            } else {
                let v =
                    ((color.r as i32 * 30 + color.g as i32 * 59 + color.b as i32 * 11) / 200) as u8;
                Color { r: v, g: v, b: v }
            };

            grid.put_sym_color_raw(
                (tx - player_pos.x + cx, ty - player_pos.y + cy),
                sym,
                color,
                None,
            );
        }
    }
}

pub fn draw_renderables(world: &World, grid: &mut TileGrid<GameSym>) {
    let (player_id, coords, fovs, render_on_floors, render_on_maps, renderables) = world.borrow::<(
        UniqueView<PlayerId>,
        View<Coord>,
        View<FieldOfView>,
        View<RenderOnFloor>,
        View<RenderOnMap>,
        View<Renderable>,
    )>();

    let Position { x, y } = coords.get(player_id.0).0;
    let fov = fovs.get(player_id.0);
    let w = grid.width() as i32;
    let h = grid.height() as i32;
    let cx = w / 2;
    let cy = h / 2;
    let mut render_entity = |coord: &Coord, render: &Renderable| {
        let gx = coord.0.x - x + cx;
        let gy = coord.0.y - y + cy;
        if gx >= 0 && gy >= 0 && gx < w && gy < h && fov.get(coord.0.into()) {
            grid.put_sym_color((gx, gy), render.sym, render.fg, render.bg);
        }
    };

    // Draw floor entities first.
    for (coord, render, _) in (&coords, &renderables, &render_on_floors).iter() {
        render_entity(coord, render);
    }

    // Draw normal map entities.
    for (coord, render, _) in (&coords, &renderables, &render_on_maps).iter() {
        render_entity(coord, render);
    }
}
