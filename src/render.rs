use shipyard::{Get, IntoIter, UniqueView, View, World};

use crate::{
    components::{Coord, FieldOfView, RenderOnFloor, RenderOnMap, Renderable},
    map::Map,
    player::PlayerId,
    ui,
};
use ruggle::{
    util::{Color, Position},
    CharGrid,
};

#[allow(clippy::many_single_char_names)]
pub fn draw_map(world: &World, grid: &mut CharGrid, active: bool) {
    let (map, player_id, coords, fovs) = world.borrow::<(
        UniqueView<Map>,
        UniqueView<PlayerId>,
        View<Coord>,
        View<FieldOfView>,
    )>();

    let Position { x, y } = coords.get(player_id.0).0;
    let fov = fovs.get(player_id.0);
    let w = grid.width() as i32;
    let h = grid.height() as i32;
    let cx = w / 2;
    let cy = h / 2;

    for (tx, ty, tile) in map.iter_bounds(x - cx, y - cy, x - cx + w - 1, y - cy + h - 1) {
        if let Some((ch, color)) = tile {
            let color = if fov.get((tx, ty)) {
                ui::recolor(color, active)
            } else {
                let v =
                    ((color.r as i32 * 30 + color.g as i32 * 59 + color.b as i32 * 11) / 200) as u8;
                ui::recolor(Color { r: v, g: v, b: v }, active)
            };

            grid.put_color_raw((tx - x + cx, ty - y + cy), color, None, ch);
        }
    }
}

pub fn draw_renderables(world: &World, grid: &mut CharGrid, active: bool) {
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
            grid.put_color(
                (gx, gy),
                ui::recolor(render.fg, active),
                ui::recolor(render.bg, active),
                render.ch,
            );
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
