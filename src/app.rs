use crate::game_objects::{Cube, Square};
use crate::render::{Camera, RenderLoop};
use crate::Keys;
use crate::{Pressed, Released};
use cgmath::Point3;
use std::time::Duration;
use winit::dpi::LogicalSize;
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, VirtualKeyCode};
use winit::event_loop::EventLoop;

const CAMERA_NORMAL_SPEED: f32 = 2.0;
const CAMERA_FAST_SPEED: f32 = 10.0;

pub struct Mouse {
  pub delta_x: f32,
  pub delta_y: f32,
  in_window: bool,
  getting_grabbed: bool,
}

// Additional information related to the window
struct Screen {
  middle: PhysicalPosition<f32>,
}

pub struct Scene {
  pub cube: Cube,
  pub square: Square,
}

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
    let render_loop = RenderLoop::new(event_loop);

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
      render_loop: RenderLoop::new(event_loop),
      scene: Scene {
        cube: Cube::new(Point3::new(5.0, 1.0, 0.0), [0.0, 0.0, 1.0]),
        square: Square::new(),
      },
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
    if self.keys.up_key == Pressed && self.keys.s == Released {
      self.scene.square.move_up(delta_seconds)
    }
    if self.keys.down_key == Pressed && self.keys.w == Released {
      self.scene.square.move_down(delta_seconds)
    }
    if self.keys.left_key == Pressed && self.keys.d == Released {
      self.scene.square.move_left(delta_seconds)
    }
    if self.keys.right_key == Pressed && self.keys.a == Released {
      self.scene.square.move_right(delta_seconds)
    }
  }

  pub fn handle_keyboard_input(&mut self, key_code: VirtualKeyCode, state: ElementState) -> bool {
    let state = match state {
      ElementState::Pressed => Pressed,
      ElementState::Released => Released,
    };

    let cube_obj = &mut self.scene.cube.object;

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
      VirtualKeyCode::C => {
        if state == Released {
          self.toggle_cursor_grab();
        }
      }
      VirtualKeyCode::Numpad8 => {
        if state == Released {
          let mut previous = cube_obj.get_position();
          previous.x += 1.0;
          cube_obj.update_position(previous);
        }
      }
      VirtualKeyCode::Numpad2 => {
        if state == Released {
          let mut previous = cube_obj.get_position();
          previous.x -= 1.0;
          cube_obj.update_position(previous);
        }
      }
      VirtualKeyCode::Numpad4 => {
        if state == Released {
          let mut previous = cube_obj.get_position();
          previous.z -= 1.0;
          cube_obj.update_position(previous);
        }
      }
      VirtualKeyCode::Numpad6 => {
        if state == Released {
          let mut previous = cube_obj.get_position();
          previous.z += 1.0;
          cube_obj.update_position(previous);
        }
      }
      VirtualKeyCode::Numpad9 => {
        if state == Released {
          let mut previous = cube_obj.get_position();
          previous.y -= 1.0;
          cube_obj.update_position(previous);
        }
      }
      VirtualKeyCode::Numpad3 => {
        if state == Released {
          let mut previous = cube_obj.get_position();
          previous.y += 1.0;
          cube_obj.update_position(previous);
        }
      }
      VirtualKeyCode::Numpad5 => {
        if state == Released {
          self.scene.cube.change_to_random_color();
        }
      }
      VirtualKeyCode::Escape => return true,
      _ => {}
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
