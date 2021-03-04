use sdl2::{
    event::{Event, WindowEvent},
    image::LoadSurface,
    pixels::Color,
    surface::Surface,
};
use std::time::{Duration, Instant};

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
    /// Minimum dimensions of the character grid.
    pub min_grid_size: [i32; 2],
    /// Path to font.
    pub font_path: std::path::PathBuf,
    /// Frames per second.
    pub fps: u32,
}

/// Create a [CharGrid] window and run a main event loop that calls `update` and `draw` repeatedly.
///
/// `update` should return a [RunControl] enum variant to control the loop behavior.
pub fn run<U, D>(settings: &RunSettings, mut update: U, mut draw: D)
where
    U: FnMut(&mut InputBuffer) -> RunControl,
    D: FnMut(&mut CharGrid),
{
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let _image_context = sdl2::image::init(sdl2::image::InitFlag::PNG).unwrap();

    let font = Surface::from_file(&settings.font_path).unwrap();
    let [grid_px_width, grid_px_height] =
        CharGrid::size_px(&font, settings.grid_size, settings.min_grid_size);

    assert!(grid_px_width > 0 && grid_px_height > 0);

    let window = video_subsystem
        .window(&settings.title, grid_px_width, grid_px_height)
        .resizable()
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut grid = CharGrid::new(font, settings.grid_size, settings.min_grid_size);
    let mut inputs = InputBuffer::new();

    let mut mouse_shown = true;
    let mut new_mouse_shown = None;
    let mut active_update = true;
    let mut done = false;

    let should_show_mouse = |event: &Event| match event {
        Event::KeyDown { .. } | Event::KeyUp { .. } => Some(false),
        Event::MouseMotion { .. }
        | Event::MouseButtonDown { .. }
        | Event::MouseButtonUp { .. }
        | Event::MouseWheel { .. } => Some(true),
        _ => None,
    };
    let should_resize = |event: &Event| {
        if let Event::Window {
            win_event: WindowEvent::Resized(w, h),
            ..
        } = event
        {
            Some([*w, *h])
        } else {
            None
        }
    };

    assert!(settings.fps > 0);

    let frame_time = Duration::new(0, 1_000_000_000u32 / settings.fps);
    let mut previous = Instant::now();
    let mut lag = frame_time; // Update once to start with.

    #[cfg(feature = "fps")]
    let mut update_count = 0;
    #[cfg(feature = "fps")]
    let mut frame_count = 0;
    #[cfg(feature = "fps")]
    let mut last_fps_print = Instant::now();

    while !done {
        let mut new_window_size = None;

        // Wait for an event if waiting is requested.
        if !active_update && !inputs.more_inputs() {
            let event = event_pump.wait_event();
            if let Some(show_it) = should_show_mouse(&event) {
                new_mouse_shown = Some(show_it);
            }
            if let Some(new_size) = should_resize(&event) {
                new_window_size = Some(new_size);
            }
            inputs.handle_event(&event);
        }

        // Poll for additional events.
        for event in event_pump.poll_iter() {
            if let Some(show_it) = should_show_mouse(&event) {
                new_mouse_shown = Some(show_it);
            }
            if let Some(new_size) = should_resize(&event) {
                new_window_size = Some(new_size);
            }
            inputs.handle_event(&event);
        }

        // Show or hide mouse cursor based on keyboard and mouse input.
        if new_mouse_shown.is_some() {
            let show_it = new_mouse_shown.unwrap();
            if mouse_shown != show_it {
                sdl_context.mouse().show_cursor(show_it);
                mouse_shown = show_it;
            }
            new_mouse_shown = None;
        }

        // Prepare internal CharGrid buffers, resizing if necessary.
        grid.prepare(&texture_creator, new_window_size);

        // Perform update(s).
        let start = previous;
        if active_update {
            let current = Instant::now();
            lag += current.duration_since(previous);
            previous = current;

            // Perform update(s) based on wall clock time.
            while lag >= frame_time {
                #[cfg(feature = "fps")]
                {
                    update_count += 1;
                }

                match update(&mut inputs) {
                    RunControl::Update => lag -= frame_time,
                    RunControl::WaitForEvent => {
                        active_update = false;
                        lag = Duration::new(0, 0);
                    }
                    RunControl::Quit => {
                        done = true;
                        lag = Duration::new(0, 0);
                    }
                }
            }
        } else {
            previous = Instant::now();

            #[cfg(feature = "fps")]
            {
                update_count += 1;
            }

            // Update once in response to events.
            match update(&mut inputs) {
                RunControl::WaitForEvent => {}
                RunControl::Update => {
                    active_update = true;
                    lag = frame_time;
                }
                RunControl::Quit => done = true,
            }
        }

        // Skip rendering if we're going to exit anyway.
        if done {
            break;
        }

        // Draw the grid...
        draw(&mut grid);

        // ... and draw it onto the screen.
        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        grid.draw(&mut canvas);
        canvas.present();

        // Discard any current input to make way for the next one.
        inputs.clear_input();

        #[cfg(feature = "fps")]
        {
            frame_count += 1;
        }

        // Show updates and frames per second.
        #[cfg(feature = "fps")]
        if Instant::now().duration_since(last_fps_print) >= Duration::new(1, 0) {
            eprintln!(
                "FPS: {}{}\tUpdates: {}",
                frame_count,
                if frame_count < 100 { "\t" } else { "" },
                update_count,
            );
            last_fps_print = Instant::now();
            update_count = 0;
            frame_count = 0;
        }

        // Sleep until the next frame is due.
        let elapsed = Instant::now().duration_since(start);
        if elapsed < frame_time {
            std::thread::sleep(frame_time - elapsed);
        }
    }
}
