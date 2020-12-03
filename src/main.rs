mod map;

use piston::input::{Button, Key};
use std::collections::HashSet;
use std::path::PathBuf;

use map::{Map, Tile};
use ruggle::{FovShape, InputEvent, KeyMods, RunSettings};

fn main() {
    let mut x: i32 = 40;
    let mut y: i32 = 18;
    let mut map = Map::new(80, 36);
    let mut fov = HashSet::new();

    map.generate();

    let settings = RunSettings {
        title: "Ruggle".to_string(),
        grid_size: [80, 36],
        font_path: PathBuf::from("assets/gohufont-uni-14.ttf"),
        font_size: 14.0,
        min_fps: 30,
        max_fps: 60,
    };

    ruggle::run(settings, |inputs, grid| {
        inputs.prepare_input();
        if let Some(e) = inputs.get_input() {
            let dist = if inputs.get_mods(KeyMods::SHIFT) {
                2
            } else if inputs.get_mods(KeyMods::CTRL) {
                3
            } else if inputs.get_mods(KeyMods::ALT) {
                5
            } else {
                1
            };

            if let InputEvent::Press(button) = e {
                if let Button::Keyboard(key) = button {
                    let mut moved = true;

                    match key {
                        Key::Up => y -= dist,
                        Key::Down => y += dist,
                        Key::Left => x -= dist,
                        Key::Right => x += dist,
                        _ => moved = false,
                    }

                    if moved {
                        fov.clear();
                        for (x, y, symmetric) in
                            ruggle::field_of_view(&map, (x, y), 8, FovShape::CirclePlus)
                        {
                            if symmetric || matches!(map.get_tile(x as u32, y as u32), &Tile::Wall)
                            {
                                fov.insert((x, y));
                            }
                        }
                    }
                }
            }
        }

        grid.clear();

        for (tx, ty, tile) in map.iter_bounds(x - 40, y - 18, x + 39, y + 17) {
            if let Some((ch, color)) = tile {
                let color = if fov.contains(&(tx, ty)) {
                    color
                } else {
                    let v = (0.3 * color[0] + 0.59 * color[1] + 0.11 * color[2]) / 2.;
                    [v, v, v, color[3]]
                };

                grid.put_color(
                    [(tx - x + 40) as u32, (ty - y + 18) as u32],
                    Some(color),
                    None,
                    ch,
                );
            }
        }

        grid.print_color(
            [32, 0],
            Some([1., 1., 0., 1.]),
            Some([0.3, 0.3, 0.3, 1.]),
            &format!("Hello world! {} {}", x, y),
        );
        grid.put_color([40, 18], Some([1., 1., 0., 1.]), None, '@');

        false
    });
}
