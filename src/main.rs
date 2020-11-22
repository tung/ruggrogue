use piston::input::{Button, Key};
use std::path::PathBuf;

use ruggle::{App, AppContext, AppSettings, InputEvent, KeyMods};

struct Game {
    x: i32,
    y: i32,
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
                    Button::Keyboard(key) => match key {
                        Key::Up => self.y -= dist,
                        Key::Down => self.y += dist,
                        Key::Left => self.x -= dist,
                        Key::Right => self.x += dist,
                        _ => {}
                    },
                    _ => {}
                },
                _ => {}
            }
        }

        ctx.grid.clear();
        ctx.grid.print_color(
            [32, 16],
            Some([1., 1., 0., 1.]),
            Some([0.3, 0.3, 0.3, 1.]),
            &format!("Hello world! {} {}", self.x, self.y),
        );
        ctx.grid.put([self.x as u32, self.y as u32], '@');

        false
    }
}

fn main() {
    let game = Game { x: 40, y: 18 };
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
