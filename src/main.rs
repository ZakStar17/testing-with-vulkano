pub mod app;
pub mod game_objects;
pub mod render;
mod keys;
pub mod other;

pub use keys::KeyState::{Pressed, Released};
pub use keys::Keys;

use std::time::{Duration, Instant};

use winit::event::{Event, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use crate::app::App;

const PRINT_FPS: bool = true;
const MILLIS_BETWEEN_FPS_PRINTS: u64 = 1500;

// If a window is resized or moved, there are some calculations
// in rendering that need to be done
// In order to wait for the user to finish performing his action,
// the app waits a specific amount of time before resuming drawing
const MILLIS_BETWEEN_MOVING: u64 = 200;
const MILLIS_BETWEEN_RESIZING: u64 = 50;

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
