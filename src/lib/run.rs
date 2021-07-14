use sdl2::{
    event::{Event, WindowEvent},
    pixels::Color as Sdl2Color,
    rect::Rect,
};
use std::time::{Duration, Instant};

use crate::{
    input_buffer::InputBuffer,
    tilegrid::{Symbol, TileGridLayer, Tileset, TilesetInfo},
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
pub struct RunSettings<Y: Symbol> {
    /// Window title.
    pub title: String,
    /// Initial pixel width and height of the window.
    pub window_size: Size,
    /// Minimum pixel width and height of the window.
    pub min_window_size: Size,
    /// Frames per second.
    pub fps: u32,
    /// Tilesets to draw TileGrids with.
    pub tileset_infos: Vec<TilesetInfo<Y>>,
}

/// Create a window and run a main event loop that calls `update` repeatedly.
///
/// `update` should return a [RunControl] enum variant to control the loop behavior.
pub fn run<U, Y>(settings: RunSettings<Y>, mut update: U)
where
    U: FnMut(&mut InputBuffer, &mut Vec<TileGridLayer<Y>>, &[Tileset<Y>], Size) -> RunControl,
    Y: Symbol,
{
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let _image_context = sdl2::image::init(sdl2::image::InitFlag::PNG).unwrap();

    assert!(settings.window_size.w > 0 && settings.window_size.w <= i32::MAX as u32);
    assert!(settings.window_size.h > 0 && settings.window_size.h <= i32::MAX as u32);
    assert!(settings.min_window_size.w > 0 && settings.min_window_size.w <= i32::MAX as u32);
    assert!(settings.min_window_size.h > 0 && settings.min_window_size.h <= i32::MAX as u32);
    assert!(settings.window_size.w >= settings.min_window_size.w);
    assert!(settings.window_size.h >= settings.min_window_size.h);

    let mut window = video_subsystem
        .window(
            &settings.title,
            settings.window_size.w,
            settings.window_size.h,
        )
        .resizable()
        .position_centered()
        .build()
        .unwrap();

    window
        .set_minimum_size(settings.window_size.w, settings.window_size.h)
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut event_pump = sdl_context.event_pump().unwrap();

    assert!(!settings.tileset_infos.is_empty());

    let mut tilesets = Vec::with_capacity(settings.tileset_infos.len());
    for tileset_info in settings.tileset_infos {
        tilesets.push(Tileset::new(tileset_info));
    }

    let mut window_size = canvas.output_size().unwrap();
    let mut window_rect = Rect::new(0, 0, window_size.0, window_size.1);
    let mut layers: Vec<TileGridLayer<Y>> = Vec::new();
    let mut inputs = InputBuffer::new();

    let mut mouse_shown = true;
    let mut active_update = true;
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
        let waited_event = if !active_update && !inputs.more_inputs() {
            Some(event_pump.wait_event())
        } else {
            None
        };

        // Poll for additional events and handle all events.
        for event in waited_event.into_iter().chain(event_pump.poll_iter()) {
            match event {
                Event::Window {
                    win_event: WindowEvent::Resized(w, h),
                    ..
                } => {
                    window_size = (w as u32, h as u32);
                }
                Event::KeyDown { .. } | Event::KeyUp { .. } => new_mouse_shown = Some(false),
                Event::MouseMotion { .. }
                | Event::MouseButtonDown { .. }
                | Event::MouseButtonUp { .. }
                | Event::MouseWheel { .. } => new_mouse_shown = Some(true),
                Event::RenderTargetsReset { .. } => {
                    for layer in layers.iter_mut() {
                        for grid in &mut layer.grids {
                            grid.flag_texture_reset();
                        }
                    }
                }
                Event::RenderDeviceReset { .. } => {
                    for layer in layers.iter_mut() {
                        for grid in &mut layer.grids {
                            grid.flag_texture_recreate();
                        }
                    }
                }
                _ => {}
            }

            inputs.handle_event(&event);
        }

        // Show or hide mouse cursor based on keyboard and mouse input.
        if let Some(new_mouse_shown) = new_mouse_shown {
            if mouse_shown != new_mouse_shown {
                sdl_context.mouse().show_cursor(new_mouse_shown);
                mouse_shown = new_mouse_shown;
            }
        }

        // Guarantee minimum window dimensions, even if we have to fake it.
        if window_size.0 < settings.min_window_size.w {
            window_size.0 = settings.min_window_size.w;
        }
        if window_size.1 < settings.min_window_size.h {
            window_size.1 = settings.min_window_size.h;
        }

        // Perform update(s).
        let start = previous;
        if active_update {
            let mut update_limit = 10;
            let current = Instant::now();
            lag += current.duration_since(previous);
            previous = current;

            // Perform update(s) based on wall clock time.
            while lag >= frame_time {
                #[cfg(feature = "fps")]
                {
                    update_count += 1;
                }

                match update(&mut inputs, &mut layers, &tilesets[..], window_size.into()) {
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

                // Avoid doing too much catch-up at once.
                update_limit -= 1;
                if update_limit == 0 {
                    lag = Duration::new(0, 0);
                }
            }
        } else {
            previous = Instant::now();

            #[cfg(feature = "fps")]
            {
                update_count += 1;
            }

            // Update once in response to events.
            match update(&mut inputs, &mut layers, &tilesets[..], window_size.into()) {
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

        window_rect.set_width(window_size.0);
        window_rect.set_height(window_size.1);
        canvas.set_clip_rect(window_rect);
        canvas.set_draw_color(Sdl2Color::BLACK);
        canvas.clear();

        // Display the grids, starting from the lowest visible layer.
        let start_layer_draw_from = layers.iter().rposition(|l| !l.draw_behind).unwrap_or(0);

        for layer in &mut layers[start_layer_draw_from..] {
            for grid in &mut layer.grids {
                grid.display(&mut tilesets[..], &mut canvas, &texture_creator);
            }
        }

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
