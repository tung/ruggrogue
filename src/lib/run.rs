use sdl2::{
    event::{Event, WindowEvent},
    image::LoadSurface,
    pixels::Color as Sdl2Color,
    surface::Surface,
};
use std::time::{Duration, Instant};

use crate::{
    chargrid::{CharGrid, Font},
    input_buffer::InputBuffer,
    util::Size,
};

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
    pub grid_size: Size,
    /// Minimum dimensions of the character grid.
    pub min_grid_size: Size,
    /// Path to font.
    pub font_path: std::path::PathBuf,
    /// Frames per second.
    pub fps: u32,
}

fn handle_event(
    event: &Event,
    grid: &mut CharGrid,
    window_size: &mut (u32, u32),
    new_mouse_shown: &mut Option<bool>,
) {
    match event {
        Event::Window {
            win_event: WindowEvent::Resized(w, h),
            ..
        } => {
            *window_size = (*w as u32, *h as u32);
        }
        Event::KeyDown { .. } | Event::KeyUp { .. } => *new_mouse_shown = Some(false),
        Event::MouseMotion { .. }
        | Event::MouseButtonDown { .. }
        | Event::MouseButtonUp { .. }
        | Event::MouseWheel { .. } => *new_mouse_shown = Some(true),
        Event::RenderTargetsReset { .. } => grid.flag_texture_reset(),
        Event::RenderDeviceReset { .. } => grid.flag_texture_recreate(),
        _ => {}
    }
}

/// Create a [CharGrid] window and run a main event loop that calls `update` repeatedly.
///
/// `update` should return a [RunControl] enum variant to control the loop behavior.
pub fn run<U>(settings: &RunSettings, mut update: U)
where
    U: FnMut(&mut InputBuffer, &mut CharGrid) -> RunControl,
{
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let _image_context = sdl2::image::init(sdl2::image::InitFlag::PNG).unwrap();

    let font_surface = Surface::from_file(&settings.font_path).unwrap();
    let mut font = Font::new(font_surface);
    let [grid_px_width, grid_px_height] =
        CharGrid::size_px::<Size, Size>(&font, settings.grid_size, settings.min_grid_size);

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

    let mut grid = CharGrid::new(&font, settings.grid_size, settings.min_grid_size);
    let mut inputs = InputBuffer::new();

    let mut mouse_shown = true;
    let mut active_update = true;
    let mut window_size = canvas.output_size().unwrap();
    let mut done = false;

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
        let mut new_mouse_shown = None;

        // Wait for an event if waiting is requested.
        if !active_update && !inputs.more_inputs() {
            let event = event_pump.wait_event();
            handle_event(&event, &mut grid, &mut window_size, &mut new_mouse_shown);
            inputs.handle_event(&event);
        }

        // Poll for additional events.
        for event in event_pump.poll_iter() {
            handle_event(&event, &mut grid, &mut window_size, &mut new_mouse_shown);
            inputs.handle_event(&event);
        }

        // Show or hide mouse cursor based on keyboard and mouse input.
        if let Some(new_mouse_shown) = new_mouse_shown {
            if mouse_shown != new_mouse_shown {
                sdl_context.mouse().show_cursor(new_mouse_shown);
                mouse_shown = new_mouse_shown;
            }
        }

        // Prepare internal CharGrid buffers, resizing if necessary.
        grid.prepare(&font, window_size);

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

                match update(&mut inputs, &mut grid) {
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
            match update(&mut inputs, &mut grid) {
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

        // Draw the grid onto the screen.
        canvas.set_draw_color(Sdl2Color::BLACK);
        canvas.clear();
        grid.draw(&mut font, &mut canvas, &texture_creator);
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
