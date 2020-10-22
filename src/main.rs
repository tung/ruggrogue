use glutin_window::GlutinWindow;
use opengl_graphics::{Filter, GlGraphics, GlyphCache, OpenGL, TextureSettings};
use piston::event_loop::{EventLoop, EventSettings, Events};
use piston::input::RenderEvent;
use piston::window::WindowSettings;
use std::path::PathBuf;

use ruggle::CharGrid;

fn main() {
    let opengl = OpenGL::V3_2;
    let settings = WindowSettings::new("Ruggle", [640, 480])
        .graphics_api(opengl)
        .exit_on_esc(true);
    let mut window: GlutinWindow = settings.build().expect("Could not create window");

    let mut events = Events::new(EventSettings::new().lazy(true));
    let mut gl = GlGraphics::new(opengl);

    let font_path = PathBuf::from("assets/LiberationMono-Regular.ttf");
    let font_size: u32 = 11;
    let texture_settings = TextureSettings::new().filter(Filter::Linear);
    let mut glyphs = GlyphCache::new(font_path, (), texture_settings).expect("Could not load font");

    let mut grid = CharGrid::new([80, 43]);
    grid.print([34, 21], "Hello world!");

    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            gl.draw(args.viewport(), |c, g| {
                use graphics::clear;

                clear([0., 0., 1., 1.], g);
                grid.draw(font_size, &mut glyphs, &c, g);
            });
        }
    }
}
