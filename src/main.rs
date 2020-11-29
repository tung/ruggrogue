mod map;

use piston::input::{Button, Key};
use std::collections::HashSet;
use std::path::PathBuf;

use map::{Map, Tile};
use ruggle::{App, AppContext, AppSettings, FovShape, InputEvent, KeyMods};

struct Game {
    x: i32,
    y: i32,
    map: Map,
    fov: HashSet<(i32, i32)>,
}

impl Game {
    fn new() -> Self {
        let mut map = Map::new(80, 36);

        map.generate();

        Self {
            x: 40,
            y: 18,
            map,
            fov: HashSet::new(),
        }
    }
}

impl App for Game {
    #[allow(clippy::single_match)]
    fn update(&mut self, ctx: &mut AppContext) -> bool {
        ctx.inputs.prepare_input();
        if let Some(e) = ctx.inputs.get_input() {
            let dist = if ctx.inputs.get_mods(KeyMods::SHIFT) {
                2
            } else if ctx.inputs.get_mods(KeyMods::CTRL) {
                3
            } else if ctx.inputs.get_mods(KeyMods::ALT) {
                5
            } else {
                1
            };

            match e {
                InputEvent::Press(button) => match button {
                    Button::Keyboard(key) => {
                        let mut moved = true;

                        match key {
                            Key::Up => self.y -= dist,
                            Key::Down => self.y += dist,
                            Key::Left => self.x -= dist,
                            Key::Right => self.x += dist,
                            _ => moved = false,
                        }

                        if moved {
                            self.fov.clear();
                            for (x, y, symmetric) in ruggle::field_of_view(
                                &self.map,
                                (self.x, self.y),
                                8,
                                FovShape::CirclePlus,
                            ) {
                                if symmetric
                                    || matches!(self.map.get_tile(x as u32, y as u32), &Tile::Wall)
                                {
                                    self.fov.insert((x, y));
                                }
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        ctx.grid.clear();

        for (x, y, ch, color) in self.map.iter() {
            let color = if self.fov.contains(&(x as i32, y as i32)) {
                color
            } else {
                let v = (0.3 * color[0] + 0.59 * color[1] + 0.11 * color[2]) / 2.;
                [v, v, v, color[3]]
            };
            ctx.grid.put_color([x, y], Some(color), None, ch);
        }

        ctx.grid.print_color(
            [32, 0],
            Some([1., 1., 0., 1.]),
            Some([0.3, 0.3, 0.3, 1.]),
            &format!("Hello world! {} {}", self.x, self.y),
        );
        ctx.grid.put_color(
            [self.x as u32, self.y as u32],
            Some([1., 1., 0., 1.]),
            None,
            '@',
        );

        false
    }
}

fn main() {
    let game = Game::new();
    let settings = AppSettings {
        title: "Ruggle".to_string(),
        grid_size: [80, 36],
        font_path: PathBuf::from("assets/gohufont-uni-14.ttf"),
        font_size: 14.0,
        min_fps: 30,
        max_fps: 60,
    };

    ruggle::run(settings, game);
}
