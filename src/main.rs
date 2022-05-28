//! Welcome to the Vulkano testing project docs!
//! 
//! The main struct is [`App`]. You can start by looking at it, and then progress to
//! the other modules.


pub mod app;
pub mod game_objects;
mod keys;
pub mod other;
pub mod render;
mod scene;

pub use keys::{
  KeyState::{Pressed, Released},
  Keys,
};

pub use scene::Scene;
pub use app::App;

use std::time::{Duration, Instant};
use winit::{
  event::{Event, MouseScrollDelta, WindowEvent},
  event_loop::{ControlFlow, EventLoop},
};

const PRINT_FPS: bool = true;
const MILLIS_BETWEEN_FPS_PRINTS: u64 = 1500;

/// When a window is resized or moved, the program needs to recreate some of its
/// objects, which takes a bit of time
/// In order to not completely lag the window, the program waits this specific
/// amount of time in order for the user to finish his action and then resumes drawing
const MILLIS_BETWEEN_MOVING: u64 = 200;
/// See [`MILLIS_BETWEEN_MOVING`]
const MILLIS_BETWEEN_RESIZING: u64 = 50;

pub const CAMERA_NORMAL_SPEED: f32 = 2.0;
pub const CAMERA_FAST_SPEED: f32 = 10.0;

/// Contains the main event loop and matches events that get handled by [`App`]
fn main() {
  let event_loop = EventLoop::new();
  let mut app = App::start(&event_loop);
  let mut draw_next_frame = true;
  let mut time_to_resume_drawing = Duration::from_millis(0);

  let mut time_until_next_fps_print = Duration::from_millis(MILLIS_BETWEEN_FPS_PRINTS);

  let mut previous_frame_time = Instant::now();
  event_loop.run(move |event, _, control_flow| match event {
    Event::WindowEvent {
      event: WindowEvent::CloseRequested,
      ..
    } => {
      *control_flow = ControlFlow::Exit;
    }
    Event::WindowEvent {
      event: WindowEvent::Resized(_),
      ..
    } => {
      draw_next_frame = false;
      time_to_resume_drawing = Duration::from_millis(MILLIS_BETWEEN_RESIZING);
      app.handle_window_resize();
    }
    Event::WindowEvent {
      event: WindowEvent::KeyboardInput { input, .. },
      ..
    } => {
      if let Some(key_code) = input.virtual_keycode {
        if app.handle_keyboard_input(key_code, input.state) {
          *control_flow = ControlFlow::Exit;
        }
      }
    }
    Event::WindowEvent {
      event: WindowEvent::CursorMoved { position, .. },
      ..
    } => {
      app.handle_mouse_movement(position);
    }
    Event::WindowEvent {
      event: WindowEvent::MouseWheel { delta, .. },
      ..
    } => {
      if let MouseScrollDelta::LineDelta(_, y) = delta {
        app.handle_mouse_wheel(y);
      }
    }
    Event::WindowEvent {
      event: WindowEvent::CursorLeft { .. },
      ..
    } => app.handle_cursor_left_window(),
    Event::WindowEvent {
      event: WindowEvent::CursorEntered { .. },
      ..
    } => app.handle_cursor_entered_window(),
    Event::WindowEvent {
      event: WindowEvent::Moved(..),
      ..
    } => {
      time_to_resume_drawing = Duration::from_millis(MILLIS_BETWEEN_MOVING);
      draw_next_frame = false;
    }
    Event::MainEventsCleared => {
      let this_frame_time = Instant::now();
      let delta_time = this_frame_time - previous_frame_time;

      if draw_next_frame {
        app.update(&delta_time);
      } else {
        if time_to_resume_drawing > delta_time {
          time_to_resume_drawing -= delta_time;
        } else {
          draw_next_frame = true;
        }
      }

      if PRINT_FPS {
        if time_until_next_fps_print > delta_time {
          time_until_next_fps_print -= delta_time;
        } else {
          time_until_next_fps_print = Duration::from_millis(MILLIS_BETWEEN_FPS_PRINTS);
          println!("fps: {}", 1000000.0 / ((delta_time).as_micros() as f32));
        }
      }

      previous_frame_time = this_frame_time;
    }
    _ => (),
  });
}
