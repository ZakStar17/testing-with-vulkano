use crate::{
  render::{Camera, RenderLoop},
  Keys, Pressed, Released, Scene, CAMERA_FAST_SPEED, CAMERA_NORMAL_SPEED,
};
use cgmath::Point3;
use std::time::Duration;
use winit::{
  dpi::{LogicalSize, PhysicalPosition},
  event::{ElementState, VirtualKeyCode},
  event_loop::EventLoop,
};

pub struct Mouse {
  pub delta_x: f32,
  pub delta_y: f32,
  in_window: bool,
  getting_grabbed: bool,
}

/// Additional information related to the window
struct Screen {
  middle: PhysicalPosition<f32>,
}

/// # Main application
/// Contains most of the program objects and handles events related with the window.
pub struct App {
  render_loop: RenderLoop,
  scene: Scene,
  keys: Keys,
  camera: Camera,
  mouse: Mouse,
  screen: Screen,
}

impl App {
  pub fn start(event_loop: &EventLoop<()>) -> Self {
    let scene = Scene::load();

    let render_loop = RenderLoop::new(event_loop, &scene);

    // initial window configuration
    let window = render_loop.get_window();
    window.set_title("Really cool game");

    // this will trigger an initial resize
    window.set_inner_size(LogicalSize::new(600.0f32, 600.0));

    let window_dimensions = window.inner_size();
    let aspect_ratio = window_dimensions.width as f32 / window_dimensions.height as f32;
    let middle_position = PhysicalPosition {
      x: window_dimensions.width as f32 / 2.0,
      y: window_dimensions.height as f32 / 2.0,
    };

    let camera = Camera::new(
      Point3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
      },
      CAMERA_NORMAL_SPEED,
      0.8,
      aspect_ratio,
    );

    Self {
      render_loop,
      scene,
      keys: Keys::default(),
      camera,
      screen: Screen {
        middle: middle_position,
      },
      mouse: Mouse {
        delta_x: 0.0,
        delta_y: 0.0,
        in_window: false,
        getting_grabbed: false,
      },
    }
  }

  pub fn update(&mut self, delta_time: &Duration) {
    self.camera.handle_keys(&self.keys, delta_time);
    self.camera.handle_mouse_movement(&self.mouse);
    self.mouse.delta_x = 0.0;
    self.mouse.delta_y = 0.0;

    self.update_square_position((delta_time.as_micros() as f32) / 1000000.0);

    self.render_loop.update(&self.camera, &self.scene);
  }

  fn update_square_position(&mut self, delta_seconds: f32) {
    let square = self.scene.get_square_mut(0);
    if self.keys.up_key == Pressed && self.keys.s == Released {
      square.move_up(delta_seconds)
    }
    if self.keys.down_key == Pressed && self.keys.w == Released {
      square.move_down(delta_seconds)
    }
    if self.keys.left_key == Pressed && self.keys.d == Released {
      square.move_left(delta_seconds)
    }
    if self.keys.right_key == Pressed && self.keys.a == Released {
      square.move_right(delta_seconds)
    }
  }

  pub fn handle_keyboard_input(&mut self, key_code: VirtualKeyCode, state: ElementState) -> bool {
    let state = match state {
      ElementState::Pressed => Pressed,
      ElementState::Released => Released,
    };

    let cube = self.scene.get_cube_mut(0);
    let renderable_cube = cube.as_renderable();

    match key_code {
      VirtualKeyCode::W => self.keys.w = state,
      VirtualKeyCode::A => self.keys.a = state,
      VirtualKeyCode::S => self.keys.s = state,
      VirtualKeyCode::D => self.keys.d = state,
      VirtualKeyCode::Space => self.keys.space = state,
      VirtualKeyCode::LControl => self.keys.l_ctrl = state,
      VirtualKeyCode::LShift => {
        if state == Pressed {
          self.camera.speed = CAMERA_FAST_SPEED;
        } else {
          self.camera.speed = CAMERA_NORMAL_SPEED;
        }
      }
      VirtualKeyCode::Up => self.keys.up_key = state,
      VirtualKeyCode::Down => self.keys.down_key = state,
      VirtualKeyCode::Left => self.keys.left_key = state,
      VirtualKeyCode::Right => self.keys.right_key = state,
      VirtualKeyCode::Escape => return true,
      _ => {}
    }

    if state == Released {
      match key_code {
        VirtualKeyCode::C => {
          self.toggle_cursor_grab();
        }
        VirtualKeyCode::Numpad8 => {
          renderable_cube.move_relative_x(1.0);
        }
        VirtualKeyCode::Numpad2 => {
          renderable_cube.move_relative_x(-1.0);
        }
        VirtualKeyCode::Numpad4 => {
          renderable_cube.move_relative_z(-1.0);
        }
        VirtualKeyCode::Numpad6 => {
          renderable_cube.move_relative_z(1.0);
        }
        VirtualKeyCode::Numpad9 => {
          renderable_cube.move_relative_y(-1.0);
        }
        VirtualKeyCode::Numpad3 => {
          renderable_cube.move_relative_y(1.0);
        }
        VirtualKeyCode::Numpad5 => {
          cube.change_to_random_color();
        }
        _ => {}
      }
    }

    false
  }

  pub fn handle_window_resize(&mut self) {
    self.render_loop.handle_window_resize();
    self.camera.set_aspect_ratio(self.get_aspect_ratio());
  }

  pub fn handle_mouse_movement(&mut self, position: PhysicalPosition<f64>) {
    if self.mouse.getting_grabbed {
      if self.mouse.in_window {
        self.mouse.delta_x += position.x as f32 - self.screen.middle.x;
        self.mouse.delta_y += position.y as f32 - self.screen.middle.y;
      }
      self
        .render_loop
        .get_window()
        .set_cursor_position(self.screen.middle)
        .unwrap();
    }
  }

  pub fn handle_mouse_wheel(&mut self, delta: f32) {
    self.camera.handle_zoom(delta);
  }

  pub fn handle_cursor_entered_window(&mut self) {
    if self.mouse.getting_grabbed {
      let window = self.render_loop.get_window();
      window.set_cursor_grab(true).unwrap();
      window.set_cursor_position(self.screen.middle).unwrap();
      self.mouse.delta_x = 0.0;
      self.mouse.delta_y = 0.0;
    }
    self.mouse.in_window = true;
  }

  pub fn handle_cursor_left_window(&mut self) {
    if self.mouse.getting_grabbed {
      let window = self.render_loop.get_window();
      window.set_cursor_grab(false).unwrap();
    }

    self.mouse.in_window = false;
  }

  fn toggle_cursor_grab(&mut self) {
    let window = self.render_loop.get_window();
    if self.mouse.getting_grabbed {
      self.mouse.getting_grabbed = false;

      window.set_cursor_grab(false).unwrap();
      window.set_cursor_visible(true);
    } else {
      self.mouse.getting_grabbed = true;

      window.set_cursor_grab(true).unwrap();
      window.set_cursor_visible(false);

      window.set_cursor_position(self.screen.middle).unwrap();
      self.mouse.delta_x = 0.0;
      self.mouse.delta_y = 0.0;
    }
  }

  fn get_aspect_ratio(&self) -> f32 {
    let window_size = self.render_loop.get_window().inner_size();
    window_size.width as f32 / window_size.height as f32
  }
}
