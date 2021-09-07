use shipyard::{Get, IntoIter, UniqueView, View, World};

use crate::{
    chunked::ChunkedMapGrid,
    components::{Coord, FieldOfView, RenderOnFloor, RenderOnMap, Renderable},
    gamesym::GameSym,
    player::PlayerId,
};
use ruggrogue::TileGrid;

pub fn draw_renderables(
    chunked_map_grid: &ChunkedMapGrid,
    world: &World,
    grid: &mut TileGrid<GameSym>,
) {
    let (player_id, coords, fovs, render_on_floors, render_on_maps, renderables) = world.borrow::<(
        UniqueView<PlayerId>,
        View<Coord>,
        View<FieldOfView>,
        View<RenderOnFloor>,
        View<RenderOnMap>,
        View<Renderable>,
    )>();

    let fov = fovs.get(player_id.0);

    // Draw floor entities first.
    for (coord, render, _) in (&coords, &renderables, &render_on_floors).iter() {
        if fov.get(coord.0.into()) {
            if let Some(pos) = chunked_map_grid.map_to_grid_pos(world, coord.0) {
                grid.put_sym_color(pos, render.sym, render.fg, render.bg);
            }
        }
    }

    // Draw normal map entities.
    for (coord, render, _) in (&coords, &renderables, &render_on_maps).iter() {
        if fov.get(coord.0.into()) {
            if let Some(pos) = chunked_map_grid.map_to_grid_pos(world, coord.0) {
                grid.put_sym_color(pos, render.sym, render.fg, render.bg);
            }
        }
    }
}
