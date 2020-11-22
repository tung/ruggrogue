use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventLoop, EventSettings, Events};
use piston::input::RenderEvent;
use piston::window::WindowSettings;
use piston::{UpdateEvent, Window};
use rusttype::Font;
use sdl2_window::Sdl2Window;
use std::fs;

use crate::chargrid::CharGrid;
use crate::input_buffer::InputBuffer;

/// Window and event loop settings for [run].
pub struct AppSettings {
    /// Window title.
    pub title: String,
    /// Dimensions of the character grid.
    pub grid_size: [u32; 2],
    /// Path to font.
    pub font_path: std::path::PathBuf,
    /// Size of font.
    pub font_size: f32,
    /// FPS limit when waiting for an event to handle.  Most of the time, the event loop will be
    /// idle, but this limit can be reached when lots of unhandled events come in at once, e.g.
    /// mouse movement events.
    pub min_fps: u64,
    /// FPS limit when continuous updates are needed.  This occurs automatically when the input
    /// buffer is non-empty, but can also be requested by returning `true` from [App::update].
    pub max_fps: u64,
}

/// A context for [App::update] that allows an implementor to handle input and output.
pub struct AppContext<'a> {
    pub grid: CharGrid<'a>,
    pub inputs: InputBuffer,
}

/// Something that can be updated as part of a main event loop, for use with [run].
pub trait App {
    /// An update function that should read inputs and write to the grid in `ctx`.
    /// Return `true` to drive continuous updates, or `false` to wait for events.
    fn update(&mut self, ctx: &mut AppContext) -> bool;
}

/// Create a [CharGrid] window and run a main event loop that calls [App::update] on `app`
/// repeatedly.  The loop calls for updates continuously if the input buffer is non-empty, or the
/// previous [App::update] call returned `true`, otherwise it will wait for an input event.
pub fn run<T: App>(settings: AppSettings, mut app: T) {
    let font_data = fs::read(settings.font_path).unwrap();
    let font = Font::try_from_vec(font_data).unwrap();
    let grid = CharGrid::new(settings.grid_size, &font, settings.font_size);

    let opengl = OpenGL::V3_2;
    let window_settings = WindowSettings::new(settings.title, grid.size())
        .graphics_api(opengl)
        .exit_on_esc(true);
    let mut window: Sdl2Window = window_settings.build().unwrap();
    let mut gl = GlGraphics::new(opengl);

    let mut need_active = false;
    let mut active_events = false;
    let active_event_settings = EventSettings::new()
        .ups(settings.max_fps)
        .max_fps(settings.max_fps);
    let inactive_event_settings = EventSettings::new().lazy(true).max_fps(settings.min_fps);
    let mut events = Events::new(inactive_event_settings);

    let mut context = AppContext {
        grid,
        inputs: InputBuffer::new(),
    };

    app.update(&mut context);

    while let Some(e) = events.next(&mut window) {
        context.inputs.handle_event(&e);

        // Update for buffered inputs and update events.
        if context.inputs.more_inputs() || e.update_args().is_some() {
            need_active = app.update(&mut context);
        }

        // Keep driving updates if more inputs are buffered.
        if context.inputs.more_inputs() {
            need_active = true;
        }

        if let Some(args) = e.render_args() {
            gl.draw(args.viewport(), |c, g| {
                use graphics::Graphics;

                let window_size = window.size();

                g.clear_color([0.3, 0.3, 0.3, 1.]);
                context
                    .grid
                    .draw(None, Some([window_size.width, window_size.height]), &c, g);
            });
        }

        if !active_events && need_active {
            active_events = true;
            events.set_event_settings(active_event_settings);
        } else if active_events && !need_active {
            active_events = false;
            events.set_event_settings(inactive_event_settings);
        }

        // Discard any current input to make way for the next one.
        context.inputs.clear_input();
    }
}
