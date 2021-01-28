use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventLoop, EventSettings, Events};
use piston::input::RenderEvent;
use piston::window::WindowSettings;
use piston::{MouseCursorEvent, PressEvent, ResizeEvent, UpdateEvent, Window};
use sdl2_window::Sdl2Window;

use crate::chargrid::CharGrid;
use crate::input_buffer::InputBuffer;

/// Return value for `update` callback sent into [run] that controls the main event loop.
pub enum RunControl {
    /// Wait for an event before calling `update` again.
    WaitForEvent,
    /// Call `update` again next frame.
    Update,
    /// Quit the run loop.
    Quit,
}

/// Window and event loop settings for [run].
pub struct RunSettings {
    /// Window title.
    pub title: String,
    /// Dimensions of the character grid.
    pub grid_size: [i32; 2],
    /// Path to font.
    pub font_path: std::path::PathBuf,
    /// FPS limit when waiting for an event to handle.  Most of the time, the event loop will be
    /// idle, but this limit can be reached when lots of unhandled events come in at once, e.g.
    /// mouse movement events.
    pub min_fps: u64,
    /// FPS limit when continuous updates are needed.  This occurs automatically when the input
    /// buffer is non-empty, but can also be requested by returning `true` from `update`.
    pub max_fps: u64,
}

/// Create a [CharGrid] window and run a main event loop that calls `update` and `draw` repeatedly.
///
/// `update` should return a [RunControl] enum variant to control the loop behavior.
pub fn run<U, D>(settings: RunSettings, mut update: U, mut draw: D)
where
    U: FnMut(&mut InputBuffer) -> RunControl,
    D: FnMut(&mut CharGrid),
{
    let mut grid = CharGrid::new(settings.grid_size, &settings.font_path);
    let grid_size = {
        let s = grid.size_px();
        assert!(s[0] > 0 && s[1] > 0);
        [s[0] as u32, s[1] as u32]
    };

    let opengl = OpenGL::V3_2;
    let window_settings = WindowSettings::new(settings.title, grid_size)
        .graphics_api(opengl)
        .exit_on_esc(true);
    let mut window: Sdl2Window = window_settings.build().unwrap();
    let mut gl = GlGraphics::new(opengl);
    let mut mouse_shown = true;

    let mut need_active = false;
    let mut active_events = false;
    let active_event_settings = EventSettings::new()
        .ups(settings.max_fps)
        .max_fps(settings.max_fps);
    let inactive_event_settings = EventSettings::new().lazy(true).max_fps(settings.min_fps);

    let mut inputs = InputBuffer::new();

    let mut events = Events::new(match update(&mut inputs) {
        RunControl::WaitForEvent => inactive_event_settings,
        RunControl::Update => active_event_settings,
        RunControl::Quit => return,
    });
    draw(&mut grid);

    while let Some(e) = events.next(&mut window) {
        // Show or hide mouse cursor based on keyboard and mouse input.
        if !mouse_shown && e.mouse_cursor_args().is_some() {
            mouse_shown = true;
            window.sdl_context.mouse().show_cursor(true);
        } else if mouse_shown && e.press_args().is_some() {
            mouse_shown = false;
            window.sdl_context.mouse().show_cursor(false);
        }

        inputs.handle_event(&e);

        // Update for buffered inputs and update events.
        if inputs.more_inputs() || e.update_args().is_some() {
            match update(&mut inputs) {
                RunControl::WaitForEvent => need_active = false,
                RunControl::Update => need_active = true,
                RunControl::Quit => window.set_should_close(true),
            }
        }

        // Keep driving updates if more inputs are buffered.
        if inputs.more_inputs() {
            need_active = true;
        }

        if let Some(args) = e.resize_args() {
            grid.resize_for_px([args.window_size[0] as i32, args.window_size[1] as i32]);
        }

        if let Some(args) = e.render_args() {
            draw(&mut grid);
            gl.draw(args.viewport(), |c, g| {
                use graphics::Graphics;

                g.clear_color([0., 0., 0., 1.]);
                grid.draw(&c, g);
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
        inputs.clear_input();
    }
}
