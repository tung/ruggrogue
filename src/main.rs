use sdl2_window::Sdl2Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventLoop, EventSettings, Events};
use piston::input::{Button, Key, RenderEvent};
use piston::window::WindowSettings;
use piston::{UpdateEvent, Window};
use rusttype::Font;
use std::fs;
use std::path::PathBuf;

use ruggle::CharGrid;
use ruggle::InputBuffer;

#[allow(clippy::single_match)]
fn update(grid: &mut CharGrid, x: &mut i32, y: &mut i32, inputs: &mut InputBuffer) -> bool {
    inputs.prepare_input();
    if let Some(e) = inputs.get_input() {
        match e {
            ruggle::InputEvent::Press(button) => match button {
                Button::Keyboard(key) => match key {
                    Key::Up => *y -= 1,
                    Key::Down => *y += 1,
                    Key::Left => *x -= 1,
                    Key::Right => *x += 1,
                    _ => {}
                }
                _ => {}
            }
            _ => {}
        }
    }

    grid.clear();
    grid.print_color(
        [32, 16],
        Some([1., 1., 0., 1.]),
        Some([0.3, 0.3, 0.3, 1.]),
        &format!("Hello world! {} {}", x, y),
    );
    grid.put([*x as u32, *y as u32], '@');

    false
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
    let mut window: Sdl2Window = settings.build().expect("Could not create window");

    let mut need_active = false;
    let mut active_events = false;
    let active_event_settings = EventSettings::new().ups(60).max_fps(60);
    let inactive_event_settings = EventSettings::new().lazy(true).max_fps(30);
    let mut events = Events::new(inactive_event_settings);

    let mut gl = GlGraphics::new(opengl);

    let mut inputs = InputBuffer::new(20);

    update(&mut grid, &mut x, &mut y, &mut inputs);

    while let Some(e) = events.next(&mut window) {
        inputs.handle_event(&e);

        // Update for buffered inputs and update events.
        if inputs.more_inputs() || e.update_args().is_some() {
            need_active = update(&mut grid, &mut x, &mut y, &mut inputs);
        }

        // Keep driving updates if more inputs are buffered.
        if inputs.more_inputs() {
            need_active = true;
        }

        if let Some(args) = e.render_args() {
            gl.draw(args.viewport(), |c, g| {
                use graphics::Graphics;

                let window_size = window.size();

                g.clear_color([0.3, 0.3, 0.3, 1.]);
                grid.draw(None, Some([window_size.width, window_size.height]), &c, g);
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
