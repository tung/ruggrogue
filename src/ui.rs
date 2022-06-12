use shipyard::{Get, UniqueView, View, World};

use crate::{
    chunked::ChunkedMapGrid,
    components::{CombatStats, Equipment, Experience, Inventory, Name, Renderable},
    gamesym::GameSym,
    hunger,
    map::Map,
    message::Messages,
    player::PlayerId,
    TurnCount,
};
use ruggrogue::{
    util::{Color, Position, Size},
    Symbol, TileGrid, Tileset,
};

pub const SELECTED_BG: Color = Color {
    r: 0,
    g: 128,
    b: 255,
};

pub struct Options {
    pub tileset: u32,
    pub font: u32,
    pub map_zoom: u32,
    pub text_zoom: u32,
}

pub const MAP_GRID: usize = 0;
pub const STATUS_GRID: usize = 1;
pub const ITEM_GRID: usize = 2;
pub const MSG_FRAME_GRID: usize = 3;
pub const MSG_GRID: usize = 4;

fn draw_status<Y: Symbol>(world: &World, grid: &mut TileGrid<Y>) {
    let player_id = world.borrow::<UniqueView<PlayerId>>();

    // Draw the box one tile higher than the grid so it runs off the bottom.
    grid.draw_box(
        (0, 0),
        (grid.width(), grid.height() + 1),
        Color::GRAY,
        Color::BLACK,
    );

    // Show the player name in the top of the frame border.
    {
        let names = world.borrow::<View<Name>>();
        let player_name = &names.get(player_id.0).0;

        grid.put_char((2, 0), ' ');
        grid.print((3, 0), player_name);
        grid.put_char((3 + player_name.len() as i32, 0), ' ');
    }

    grid.print_color(
        (grid.width() as i32 - 17, 0),
        " [esc] Options ",
        true,
        None,
        None,
    );
    for x in 0..3 {
        grid.recolor_pos((grid.width() as i32 - 15 + x, 0), Color::YELLOW, None);
    }

    // Level and experience.
    {
        let exps = world.borrow::<View<Experience>>();
        let player_exp = exps.get(player_id.0);

        grid.print_color((2, 1), "Level:", true, Color::LIGHT_GRAY, None);
        grid.print((14, 1), &format!("{}", player_exp.level));

        grid.print_color((2, 2), "Experience:", true, Color::LIGHT_GRAY, None);
        grid.draw_bar(
            false,
            (14, 2),
            20,
            0,
            player_exp.exp.min(i32::MAX as u64) as i32,
            player_exp.next.min(i32::MAX as u64) as i32,
            Color::PURPLE,
            None,
        );
        grid.print_color(
            (14, 2),
            &format!("{}", player_exp.base + player_exp.exp),
            false,
            Color::WHITE,
            None,
        );
    }

    // Combat stats.
    {
        let combat_stats = world.borrow::<View<CombatStats>>();
        let player_stats = combat_stats.get(player_id.0);

        grid.print_color((2, 3), "Health:", true, Color::LIGHT_GRAY, None);
        grid.draw_bar(
            false,
            (14, 3),
            20,
            0,
            player_stats.hp,
            player_stats.max_hp,
            Color { r: 192, g: 0, b: 0 },
            None,
        );
        grid.print_color(
            (14, 3),
            &format!("{} / {}", player_stats.hp, player_stats.max_hp),
            false,
            Color::YELLOW,
            None,
        );

        grid.print_color((2, 4), "Attack:", true, Color::LIGHT_GRAY, None);
        grid.print((14, 4), &format!("{:+.0}", player_stats.attack.round()));

        grid.print_color((2, 5), "Defense:", true, Color::LIGHT_GRAY, None);
        grid.print((14, 5), &format!("{:+.0}", player_stats.defense.round()));
    }

    // Hunger
    {
        let (hunger_label, hunger_fg, hunger_bg) = world.run(hunger::player_hunger_label);

        grid.print_color((2, 6), "Hunger:", true, Color::LIGHT_GRAY, None);
        grid.print_color((14, 6), hunger_label, true, hunger_fg, hunger_bg);
    }

    // Depth
    grid.print_color((2, 7), "Depth:", true, Color::LIGHT_GRAY, None);
    grid.print(
        (14, 7),
        &format!("{}", world.borrow::<UniqueView<Map>>().depth),
    );

    // Turn
    grid.print_color((2, 8), "Turn:", true, Color::LIGHT_GRAY, None);
    grid.print(
        (14, 8),
        &format!("{}", world.borrow::<UniqueView<TurnCount>>().0),
    );
}

fn draw_item_info(world: &World, grid: &mut TileGrid<GameSym>) {
    let player_id = world.borrow::<UniqueView<PlayerId>>();

    // Draw the box one tile higher than the grid so it runs off the bottom.
    grid.draw_box(
        (0, 0),
        (grid.width(), grid.height() + 1),
        Color::GRAY,
        Color::BLACK,
    );
    grid.put_char_color((0, 0), '├', None, None);
    grid.put_char_color((grid.width() as i32 - 1, 0), '┤', None, None);

    // Show box title and item count in the top of the frame border.
    {
        let inventories = world.borrow::<View<Inventory>>();
        let item_count = inventories.get(player_id.0).items.len();

        grid.print(
            (2, 0),
            &format!(
                " [i] Inventory ({} {}) ",
                item_count,
                if item_count == 1 { "item" } else { "items" },
            ),
        );
        grid.recolor_pos((4, 0), Color::YELLOW, None);
    }

    // Weapon and armor
    {
        let equipments = world.borrow::<View<Equipment>>();
        let names = world.borrow::<View<Name>>();
        let renderables = world.borrow::<View<Renderable>>();
        let player_equipment = equipments.get(player_id.0);

        grid.print_color((2, 1), "Weapon:", true, Color::LIGHT_GRAY, None);
        if let Some(weapon) = player_equipment.weapon {
            let x = if let Ok(render) = renderables.try_get(weapon) {
                grid.put_sym_color((10, 1), render.sym, render.fg, render.bg);
                12
            } else {
                10
            };
            grid.print((x, 1), &names.get(weapon).0);
        } else {
            grid.print_color((10, 1), "-- nothing --", true, Color::GRAY, None);
        }

        grid.print_color((2, 2), "Armor:", true, Color::LIGHT_GRAY, None);
        if let Some(armor) = player_equipment.armor {
            let x = if let Ok(render) = renderables.try_get(armor) {
                grid.put_sym_color((10, 2), render.sym, render.fg, render.bg);
                12
            } else {
                10
            };
            grid.print((x, 2), &names.get(armor).0);
        } else {
            grid.print_color((10, 2), "-- nothing --", true, Color::GRAY, None);
        }
    }
}

pub fn draw_msg_frame<Y: Symbol>(msg_frame_grid: &mut TileGrid<Y>, view_mode: bool) {
    msg_frame_grid.draw_box(
        (0, 0),
        (msg_frame_grid.width(), msg_frame_grid.height()),
        Color::GRAY,
        Color::BLACK,
    );
    msg_frame_grid.put_char_color((0, 0), '├', None, None);
    msg_frame_grid.put_char_color((msg_frame_grid.width() as i32 - 1, 0), '┤', None, None);

    msg_frame_grid.print_color(
        (2, 0),
        " Messages ",
        true,
        if view_mode { Color::GRAY } else { Color::WHITE },
        Color::BLACK,
    );

    msg_frame_grid.print_color(
        (msg_frame_grid.width() as i32 - 17, 0),
        " [v] View Mode ",
        true,
        if view_mode { Color::WHITE } else { Color::GRAY },
        Color::BLACK,
    );
    msg_frame_grid.recolor_pos((msg_frame_grid.width() as i32 - 15, 0), Color::YELLOW, None);
}

fn draw_messages<Y>(world: &World, grid: &mut TileGrid<Y>, active: bool, min_y: i32, max_y: i32)
where
    Y: Symbol,
{
    let messages = world.borrow::<UniqueView<Messages>>();
    let width = grid.width().saturating_sub(2).max(1) as usize;
    let mut y = min_y;
    let mut skip_y = min_y;
    let (fg, highlight_fg) = if active {
        (Color::GRAY, Color::WHITE)
    } else {
        (Color::DARK_GRAY, Color::GRAY)
    };

    for (message, highlighted) in messages.rev_iter() {
        if y > max_y {
            break;
        }

        if message.is_empty() {
            y += 1;
            continue;
        }

        let msg_fg = if highlighted { highlight_fg } else { fg };

        grid.put_char_color((0, y), '>', msg_fg, None);
        for line in ruggrogue::word_wrap(message, width) {
            if skip_y > 0 {
                skip_y -= 1;
                continue;
            }
            grid.print_color((2, y), line, true, msg_fg, None);
            y += 1;
            if y > max_y {
                break;
            }
        }
    }
}

pub fn draw_ui<Y: Symbol>(
    world: &World,
    status_grid: &mut TileGrid<Y>,
    item_grid: &mut TileGrid<GameSym>,
    msg_grid: &mut TileGrid<Y>,
    prompt: Option<&str>,
) {
    draw_status(world, status_grid);
    draw_item_info(world, item_grid);

    if let Some(prompt) = prompt {
        for (y, prompt_line) in ruggrogue::word_wrap(prompt, 32).enumerate() {
            msg_grid.print_color((2, y as i32), prompt_line, true, Color::WHITE, None);
        }
        draw_messages(world, msg_grid, false, 3, msg_grid.height() as i32 - 1);
    } else {
        draw_messages(world, msg_grid, true, 0, msg_grid.height() as i32 - 1);
    }
}

/// Prepares grids to display the dungeon map and user interface.
pub fn prepare_main_grids<Y: Symbol>(
    chunked_map_grid: &mut ChunkedMapGrid,
    world: &World,
    grids: &mut Vec<TileGrid<Y>>,
    tilesets: &[Tileset<Y>],
    window_size: Size,
) {
    let Options {
        tileset: map_tileset_index,
        font: ui_tileset_index,
        text_zoom,
        ..
    } = *world.borrow::<UniqueView<Options>>();
    let ui_tileset = &tilesets
        .get(ui_tileset_index as usize)
        .unwrap_or(&tilesets[0]);

    let sidebar_w = 36;
    let sidebar_px_w = sidebar_w * ui_tileset.tile_width() * text_zoom;

    let new_status_size = Size {
        w: sidebar_w,
        h: 10,
    };
    let new_item_size = Size { w: sidebar_w, h: 4 };
    let new_msg_frame_size = Size {
        w: sidebar_w,
        h: (window_size.h / (ui_tileset.tile_height() * text_zoom))
            .saturating_sub(new_status_size.h + new_item_size.h)
            .max(4),
    };
    let new_msg_size = Size {
        w: new_msg_frame_size.w.saturating_sub(2).max(1),
        h: new_msg_frame_size.h.saturating_sub(2).max(1).min(100),
    };

    if !grids.is_empty() {
        // MAP_GRID resizing is handled by ChunkedMapGrid::prepare_grid below.
        grids[STATUS_GRID].resize(new_status_size);
        grids[ITEM_GRID].resize(new_item_size);
        grids[MSG_FRAME_GRID].resize(new_msg_frame_size);
        grids[MSG_GRID].resize(new_msg_size);
    } else {
        // Use a bogus size for MAP_GRID; ChunkedMapGrid::prepare_grid will resize it below.
        grids.push(TileGrid::new(
            Size { w: 1, h: 1 },
            tilesets,
            map_tileset_index as usize,
        ));

        grids.push(TileGrid::new(
            new_status_size,
            tilesets,
            ui_tileset_index as usize,
        ));
        grids[STATUS_GRID].view.clear_color = Some(Color::BLACK);

        grids.push(TileGrid::new(
            new_item_size,
            tilesets,
            ui_tileset_index as usize,
        ));
        grids[ITEM_GRID].view.clear_color = Some(Color::BLACK);

        grids.push(TileGrid::new(
            new_msg_frame_size,
            tilesets,
            ui_tileset_index as usize,
        ));
        grids[MSG_FRAME_GRID].view.clear_color = Some(Color::BLACK);

        grids.push(TileGrid::new(
            new_msg_size,
            tilesets,
            ui_tileset_index as usize,
        ));
        grids[MSG_GRID].view.clear_color = Some(Color::BLACK);
    }

    chunked_map_grid.prepare_grid(
        world,
        &mut grids[MAP_GRID],
        tilesets,
        Position { x: 0, y: 0 },
        Size {
            w: window_size.w.saturating_sub(sidebar_px_w).max(1),
            h: window_size.h,
        },
    );

    grids[STATUS_GRID].set_tileset(tilesets, ui_tileset_index as usize);
    grids[STATUS_GRID].view.pos = Position {
        x: window_size.w as i32 - sidebar_px_w as i32,
        y: 0,
    };
    grids[STATUS_GRID].view.size = Size {
        w: sidebar_px_w,
        h: grids[STATUS_GRID].height() * ui_tileset.tile_height() * text_zoom,
    };
    grids[STATUS_GRID].view.zoom = text_zoom;

    grids[ITEM_GRID].set_tileset(tilesets, ui_tileset_index as usize);
    grids[ITEM_GRID].view.pos = Position {
        x: window_size.w as i32 - sidebar_px_w as i32,
        y: grids[STATUS_GRID].view.pos.y + grids[STATUS_GRID].view.size.h as i32,
    };
    grids[ITEM_GRID].view.size = Size {
        w: sidebar_px_w,
        h: grids[ITEM_GRID].height() * ui_tileset.tile_height() * text_zoom,
    };
    grids[ITEM_GRID].view.zoom = text_zoom;

    grids[MSG_FRAME_GRID].set_tileset(tilesets, ui_tileset_index as usize);
    grids[MSG_FRAME_GRID].view.pos = Position {
        x: window_size.w as i32 - sidebar_px_w as i32,
        y: grids[ITEM_GRID].view.pos.y + grids[ITEM_GRID].view.size.h as i32,
    };
    grids[MSG_FRAME_GRID].view.size = Size {
        w: sidebar_px_w,
        h: grids[MSG_FRAME_GRID].height() * ui_tileset.tile_height() * text_zoom,
    };
    grids[MSG_FRAME_GRID].view.zoom = text_zoom;

    grids[MSG_GRID].set_tileset(tilesets, ui_tileset_index as usize);
    grids[MSG_GRID].view.pos = Position {
        x: grids[MSG_FRAME_GRID].view.pos.x + (ui_tileset.tile_width() * text_zoom) as i32,
        y: grids[MSG_FRAME_GRID].view.pos.y + (ui_tileset.tile_height() * text_zoom) as i32,
    };
    grids[MSG_GRID].view.size = Size {
        w: grids[MSG_GRID].width() * ui_tileset.tile_width() * text_zoom,
        h: grids[MSG_GRID].height() * ui_tileset.tile_height() * text_zoom,
    };
    grids[MSG_GRID].view.zoom = text_zoom;
}
