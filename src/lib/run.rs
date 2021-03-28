use sdl2::{
    event::{Event, WindowEvent},
    image::Sdl2ImageContext,
    pixels::Color as Sdl2Color,
    rect::Rect,
    render::WindowCanvas,
    EventPump, Sdl, VideoSubsystem,
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

/// Holds main loop state with initialization and loop iteration as separate functions.
struct RunLoop<'b, 's, U, Y>
where
    U: FnMut(&mut InputBuffer, &mut Vec<TileGridLayer<Y>>, &[Tileset<Y>], Size) -> RunControl,
    Y: Symbol,
{
    update: U,

    sdl_context: Sdl,
    _video_subsystem: VideoSubsystem,
    _image_context: Sdl2ImageContext,

    // This *must* be above `canvas` to guarantee that textures held by TileGrids are dropped
    // before the canvas is dropped.
    layers: Vec<TileGridLayer<'b, Y>>,

    canvas: WindowCanvas,
    event_pump: EventPump,
    window_size: (u32, u32),
    min_window_size: (u32, u32),
    window_rect: Rect,

    tilesets: Vec<Tileset<'s, Y>>,
    inputs: InputBuffer,

    mouse_shown: bool,
    active_update: bool,

    frame_time: Duration,
    previous: Instant,
    lag: Duration,

    #[cfg(feature = "fps")]
    update_count: i32,
    #[cfg(feature = "fps")]
    frame_count: i32,
    #[cfg(feature = "fps")]
    last_fps_print: Instant,
}

impl<U, Y> RunLoop<'_, '_, U, Y>
where
    U: FnMut(&mut InputBuffer, &mut Vec<TileGridLayer<Y>>, &[Tileset<Y>], Size) -> RunControl,
    Y: Symbol,
{
    fn new(settings: RunSettings<Y>, update: U) -> Self {
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

        let canvas = window.into_canvas().build().unwrap();
        let event_pump = sdl_context.event_pump().unwrap();
        let window_size = canvas.output_size().unwrap();
        let window_rect = Rect::new(0, 0, window_size.0, window_size.1);

        assert!(!settings.tileset_infos.is_empty());

        let mut tilesets = Vec::with_capacity(settings.tileset_infos.len());
        for tileset_info in settings.tileset_infos {
            tilesets.push(Tileset::new(tileset_info));
        }

        assert!(settings.fps > 0);

        let frame_time = Duration::new(0, 1_000_000_000u32 / settings.fps);

        Self {
            update,

            sdl_context,
            _video_subsystem: video_subsystem,
            _image_context,

            layers: Vec::new(),

            canvas,
            event_pump,
            window_size,
            min_window_size: (settings.min_window_size.w, settings.min_window_size.h),
            window_rect,

            tilesets,
            inputs: InputBuffer::new(),

            mouse_shown: true,
            active_update: true,

            frame_time,
            previous: Instant::now(),
            lag: frame_time,

            #[cfg(feature = "fps")]
            update_count: 0,
            #[cfg(feature = "fps")]
            frame_count: 0,
            #[cfg(feature = "fps")]
            last_fps_print: Instant::now(),
        }
    }

    fn handle_event(
        event: &Event,
        new_window_size: &mut Option<(u32, u32)>,
        new_mouse_shown: &mut Option<bool>,
        need_texture_reset: &mut bool,
        need_texture_recreate: &mut bool,
    ) {
        match event {
            Event::Window {
                win_event: WindowEvent::Resized(w, h),
                ..
            } => {
                *new_window_size = Some((*w as u32, *h as u32));
            }
            Event::KeyDown { .. } | Event::KeyUp { .. } => *new_mouse_shown = Some(false),
            Event::MouseMotion { .. }
            | Event::MouseButtonDown { .. }
            | Event::MouseButtonUp { .. }
            | Event::MouseWheel { .. } => *new_mouse_shown = Some(true),
            Event::RenderTargetsReset { .. } => *need_texture_reset = true,
            Event::RenderDeviceReset { .. } => *need_texture_recreate = true,
            _ => {}
        }
    }

    /// Returns true when done.
    fn main_loop(&mut self) -> bool {
        let mut done = false;
        let mut new_window_size = None;
        let mut new_mouse_shown = None;
        let mut need_texture_reset = false;
        let mut need_texture_recreate = false;

        // Wait for an event if waiting is requested.
        if !self.active_update && !self.inputs.more_inputs() {
            let event = self.event_pump.wait_event();
            Self::handle_event(
                &event,
                &mut new_window_size,
                &mut new_mouse_shown,
                &mut need_texture_reset,
                &mut need_texture_recreate,
            );
            self.inputs.handle_event(&event);
        }

        // Poll for additional events.
        for event in self.event_pump.poll_iter() {
            Self::handle_event(
                &event,
                &mut new_window_size,
                &mut new_mouse_shown,
                &mut need_texture_reset,
                &mut need_texture_recreate,
            );
            self.inputs.handle_event(&event);
        }

        // Show or hide mouse cursor based on keyboard and mouse input.
        if let Some(new_mouse_shown) = new_mouse_shown {
            if self.mouse_shown != new_mouse_shown {
                self.sdl_context.mouse().show_cursor(new_mouse_shown);
                self.mouse_shown = new_mouse_shown;
            }
        }

        if let Some(new_window_size) = new_window_size {
            self.window_size = new_window_size;
        }

        // Guarantee minimum window dimensions, even if we have to fake it.
        if self.window_size.0 < self.min_window_size.0 {
            self.window_size.0 = self.min_window_size.0;
        }
        if self.window_size.1 < self.min_window_size.1 {
            self.window_size.1 = self.min_window_size.1;
        }

        if need_texture_reset {
            for layer in self.layers.iter_mut() {
                for grid in &mut layer.grids {
                    grid.flag_texture_reset();
                }
            }
        }

        if need_texture_recreate {
            for layer in self.layers.iter_mut() {
                for grid in &mut layer.grids {
                    grid.flag_texture_recreate();
                }
            }
        }

        // Perform update(s).
        let start = self.previous;
        if self.active_update {
            let mut update_limit = 10;
            let current = Instant::now();
            self.lag += current.duration_since(self.previous);
            self.previous = current;

            // Perform update(s) based on wall clock time.
            while self.lag >= self.frame_time {
                #[cfg(feature = "fps")]
                {
                    self.update_count += 1;
                }

                match (self.update)(
                    &mut self.inputs,
                    &mut self.layers,
                    &self.tilesets[..],
                    self.window_size.into(),
                ) {
                    RunControl::Update => self.lag -= self.frame_time,
                    RunControl::WaitForEvent => {
                        self.active_update = false;
                        self.lag = Duration::new(0, 0);
                    }
                    RunControl::Quit => {
                        done = true;
                        self.lag = Duration::new(0, 0);
                    }
                }

                // Avoid doing too much catch-up at once.
                update_limit -= 1;
                if update_limit == 0 {
                    self.lag = Duration::new(0, 0);
                }
            }
        } else {
            self.previous = Instant::now();

            #[cfg(feature = "fps")]
            {
                self.update_count += 1;
            }

            // Update once in response to events.
            match (self.update)(
                &mut self.inputs,
                &mut self.layers,
                &self.tilesets[..],
                self.window_size.into(),
            ) {
                RunControl::WaitForEvent => {}
                RunControl::Update => {
                    self.active_update = true;
                    self.lag = self.frame_time;
                }
                RunControl::Quit => done = true,
            }
        }

        // Skip rendering if we're going to exit anyway.
        if done {
            return true;
        }

        self.window_rect.set_width(self.window_size.0);
        self.window_rect.set_height(self.window_size.1);
        self.canvas.set_clip_rect(self.window_rect);
        self.canvas.set_draw_color(Sdl2Color::BLACK);
        self.canvas.clear();

        // Display the grids, starting from the lowest visible layer.
        let start_layer_draw_from = self
            .layers
            .iter()
            .rposition(|l| !l.draw_behind)
            .unwrap_or(0);

        for layer in &mut self.layers[start_layer_draw_from..] {
            for grid in &mut layer.grids {
                grid.display(&mut self.tilesets[..], &mut self.canvas);
            }
        }

        self.canvas.present();

        // Discard any current input to make way for the next one.
        self.inputs.clear_input();

        #[cfg(feature = "fps")]
        {
            self.frame_count += 1;
        }

        // Show updates and frames per second.
        #[cfg(feature = "fps")]
        if Instant::now().duration_since(self.last_fps_print) >= Duration::new(1, 0) {
            eprintln!(
                "FPS: {}{}\tUpdates: {}",
                self.frame_count,
                if self.frame_count < 100 { "\t" } else { "" },
                self.update_count,
            );
            self.last_fps_print = Instant::now();
            self.update_count = 0;
            self.frame_count = 0;
        }

        // Sleep until the next frame is due.
        let elapsed = Instant::now().duration_since(start);
        if elapsed < self.frame_time {
            std::thread::sleep(self.frame_time - elapsed);
        }

        done
    }
}

/// Create a window and run a main event loop that calls `update` repeatedly.
///
/// `update` should return a [RunControl] enum variant to control the loop behavior.
pub fn run<U, Y>(settings: RunSettings<Y>, update: U)
where
    U: FnMut(&mut InputBuffer, &mut Vec<TileGridLayer<Y>>, &[Tileset<Y>], Size) -> RunControl,
    Y: Symbol,
{
    let mut run_loop = RunLoop::new(settings, update);

    while !run_loop.main_loop() {}
}
