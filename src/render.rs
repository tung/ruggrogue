use shipyard::{Get, IntoIter, UniqueView, View, World};

use crate::{
    components::{FieldOfView, Position, RenderOnFloor, RenderOnMap, Renderable},
    map::Map,
    player::PlayerId,
    ui,
};
use ruggle::{util::Color, CharGrid};

#[allow(clippy::many_single_char_names)]
pub fn draw_map(world: &World, grid: &mut CharGrid, active: bool) {
    let (map, player_id, fovs, positions) = world.borrow::<(
        UniqueView<Map>,
        UniqueView<PlayerId>,
        View<FieldOfView>,
        View<Position>,
    )>();

    let (x, y) = positions.get(player_id.0).into();
    let fov = fovs.get(player_id.0);
    let w = grid.size_cells().w as i32;
    let h = grid.size_cells().h as i32 - ui::HUD_LINES;
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
    let (player_id, fovs, positions, render_on_floors, render_on_maps, renderables) = world
        .borrow::<(
            UniqueView<PlayerId>,
            View<FieldOfView>,
            View<Position>,
            View<RenderOnFloor>,
            View<RenderOnMap>,
            View<Renderable>,
        )>();

    let (x, y) = positions.get(player_id.0).into();
    let fov = fovs.get(player_id.0);
    let w = grid.size_cells().w as i32;
    let h = grid.size_cells().h as i32 - ui::HUD_LINES;
    let cx = w / 2;
    let cy = h / 2;
    let mut render_entity = |pos: &Position, render: &Renderable| {
        let gx = pos.x - x + cx;
        let gy = pos.y - y + cy;
        if gx >= 0 && gy >= 0 && gx < w && gy < h && fov.get(pos.into()) {
            grid.put_color(
                (gx, gy),
                ui::recolor(render.fg, active),
                ui::recolor(render.bg, active),
                render.ch,
            );
        }
    };

    // Draw floor entities first.
    for (pos, render, _) in (&positions, &renderables, &render_on_floors).iter() {
        render_entity(pos, render);
    }

    // Draw normal map entities.
    for (pos, render, _) in (&positions, &renderables, &render_on_maps).iter() {
        render_entity(pos, render);
    }
}
