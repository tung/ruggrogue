use glutin_window::GlutinWindow;
use opengl_graphics::{Filter, GlGraphics, GlyphCache, OpenGL, TextureSettings};
use piston::event_loop::{EventLoop, EventSettings, Events};
use piston::input::{Button, Key, PressEvent, RenderEvent};
use piston::window::WindowSettings;
use std::path::PathBuf;

use ruggle::CharGrid;

fn update(grid: &mut CharGrid, x: i32, y: i32) {
    grid.clear();
    grid.print([30, 17], &format!("Hello world! {} {}", x, y));
    grid.put([x as u32, y as u32], '@');
}

fn main() {
    let opengl = OpenGL::V3_2;
    let settings = WindowSettings::new("Ruggle", [810, 510])
        .graphics_api(opengl)
        .exit_on_esc(true);
    let mut window: GlutinWindow = settings.build().expect("Could not create window");

    let mut events = Events::new(EventSettings::new().lazy(true));
    let mut gl = GlGraphics::new(opengl);

    let font_path = PathBuf::from("assets/LiberationMono-Regular.ttf");
    let font_size: u32 = 11;
    let texture_settings = TextureSettings::new().filter(Filter::Linear);
    let mut glyphs = GlyphCache::new(font_path, (), texture_settings).expect("Could not load font");

    let mut grid = CharGrid::new([80, 36]);
    let mut x: i32 = 39;
    let mut y: i32 = 20;

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
                use graphics::clear;

                clear([0., 0., 1., 1.], g);
                grid.draw(font_size, &mut glyphs, &c, g);
            });
        }
    }
}
