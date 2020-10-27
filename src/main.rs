use glutin_window::GlutinWindow;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventLoop, EventSettings, Events};
use piston::input::{Button, Key, PressEvent, RenderEvent};
use piston::window::WindowSettings;
use rusttype::Font;
use std::fs;
use std::path::PathBuf;

use ruggle::CharGrid;

fn update(grid: &mut CharGrid, x: i32, y: i32) {
    grid.clear();
    grid.print_color(
        [32, 16],
        Some([1., 1., 0., 1.]),
        Some([0.3, 0.3, 0.3, 1.]),
        &format!("Hello world! {} {}", x, y),
    );
    grid.put([x as u32, y as u32], '@');
}

fn main() {
    let font_path = PathBuf::from("assets/gohufont-uni-14.ttf");
    let font_data = fs::read(font_path).unwrap();
    let font = Font::try_from_vec(font_data).unwrap();

    let mut grid = CharGrid::new([80, 36], &font, 14.0);
    let mut x: i32 = 40;
    let mut y: i32 = 18;

    let opengl = OpenGL::V3_2;
    let settings = WindowSettings::new("Ruggle", grid.size())
        .graphics_api(opengl)
        .exit_on_esc(true);
    let mut window: GlutinWindow = settings.build().expect("Could not create window");

    let mut events = Events::new(EventSettings::new().lazy(true));
    let mut gl = GlGraphics::new(opengl);

    update(&mut grid, x, y);

    while let Some(e) = events.next(&mut window) {
        if let Some(Button::Keyboard(key)) = e.press_args() {
            let mut do_update = true;

            match key {
                Key::Up => y -= 1,
                Key::Down => y += 1,
                Key::Left => x -= 1,
                Key::Right => x += 1,
                _ => do_update = false,
            }

            if do_update {
                update(&mut grid, x, y);
            }
        }

        if let Some(args) = e.render_args() {
            gl.draw(args.viewport(), |c, g| {
                use graphics::Graphics;

                g.clear_color([0.3, 0.3, 0.3, 1.]);
                grid.draw(&c, g);
            });
        }
    }
}
