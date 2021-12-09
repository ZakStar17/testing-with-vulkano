use crate::Camera;
use crate::VulkanoProgram;

use std::sync::Arc;
use std::time::Duration;

use cgmath::Matrix3;
use cgmath::Matrix4;
use cgmath::Point3;
use cgmath::Rad;

use vulkano::device::DeviceExtensions;
use vulkano::instance::Instance;
use vulkano::swapchain::Surface;
use vulkano_win::VkSurfaceBuild;
use winit::dpi::PhysicalPosition;
use winit::event::VirtualKeyCode;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

use crate::vulkano_graphics;
use crate::vulkano_graphics::QueueFamilies;

pub struct Mouse {
  pub delta_x: f32,
  pub delta_y: f32,
  in_window: bool,
  getting_grabbed: bool,
}

pub struct App {
  pub program: VulkanoProgram,
  pub vulkan_instance: Arc<Instance>,
  pub surface: Arc<Surface<Window>>,
  camera: Camera,
  mouse: Mouse,
  pressed_keys: [bool; 4],
  pub aspect_ratio: f32,
  pub window_half_screen_position: PhysicalPosition<f32>,
}

impl App {
  pub fn start(event_loop: &EventLoop<()>) -> Self {
    let instance = vulkano_graphics::create_instance();

    let surface = WindowBuilder::new()
      .build_vk_surface(&event_loop, instance.clone())
      .unwrap();

    let device_extensions = DeviceExtensions {
      khr_swapchain: true,
      ..DeviceExtensions::none()
    };

    let physical_device =
      vulkano_graphics::get_physical_device(&instance, &device_extensions, surface.clone());

    let queue_families = QueueFamilies::init(physical_device);

    let program = VulkanoProgram::start(
      device_extensions,
      physical_device,
      &queue_families,
      surface.clone(),
    );

    let camera = Camera::new(Point3 {
      x: 0.0,
      y: 0.0,
      z: 0.0,
    });

    let mouse = Mouse {
      delta_x: 0.0,
      delta_y: 0.0,
      in_window: false,
      getting_grabbed: true,
    };

    let pressed_keys = [false, false, false, false];

    let window_dimensions = surface.window().inner_size();
    let aspect_ratio = window_dimensions.width as f32 / window_dimensions.height as f32;
    let window_half_screen_position = PhysicalPosition {
      x: window_dimensions.width as f32 / 2.0,
      y: window_dimensions.height as f32 / 2.0,
    };

    App {
      program,
      vulkan_instance: instance,
      surface,
      camera,
      mouse,
      pressed_keys,
      aspect_ratio,
      window_half_screen_position,
    }
  }

  // returns true if the window should close
  pub fn handle_keyboard_input(&mut self, key_code: VirtualKeyCode, is_pressed: bool) -> bool {
    match key_code {
      VirtualKeyCode::W => self.pressed_keys[0] = is_pressed,
      VirtualKeyCode::A => self.pressed_keys[1] = is_pressed,
      VirtualKeyCode::S => self.pressed_keys[2] = is_pressed,
      VirtualKeyCode::D => self.pressed_keys[3] = is_pressed,
      VirtualKeyCode::M => {
        if !is_pressed {
          self.toggle_cursor_grab();
        }
      }
      _ => (),
    }
    match key_code {
      VirtualKeyCode::Escape => true,
      _ => false,
    }
  }

  pub fn handle_cursor_moved(&mut self, position: PhysicalPosition<f64>) {
    if self.mouse.getting_grabbed {
      if self.mouse.in_window {
        self.mouse.delta_x += position.x as f32 - self.window_half_screen_position.x;
        self.mouse.delta_y += position.y as f32 - self.window_half_screen_position.y;
      }
      self
        .surface
        .window()
        .set_cursor_position(self.window_half_screen_position)
        .unwrap();
    }
  }

  pub fn handle_mouse_wheel(&mut self, delta_y: f32) {
    self.camera.handle_zoom(delta_y);
  }

  pub fn handle_cursor_entered_window(&mut self) {
    if self.mouse.getting_grabbed {
      let window = self.surface.window();
      window.set_cursor_grab(true).unwrap();
      window
        .set_cursor_position(self.window_half_screen_position)
        .unwrap();
      self.mouse.delta_x = 0.0;
      self.mouse.delta_y = 0.0;
    }
    self.mouse.in_window = true;
  }

  pub fn handle_cursor_left_window(&mut self) {
    if self.mouse.getting_grabbed {
      let window = self.surface.window();
      window.set_cursor_grab(false).unwrap();
    }

    self.mouse.in_window = false;
  }

  fn toggle_cursor_grab(&mut self) {
    let window = self.surface.window();
    if self.mouse.getting_grabbed {
      self.mouse.getting_grabbed = false;

      window.set_cursor_grab(false).unwrap();
      window.set_cursor_visible(true);
    } else {
      self.mouse.getting_grabbed = true;

      window.set_cursor_grab(true).unwrap();
      window.set_cursor_visible(false);

      window
        .set_cursor_position(self.window_half_screen_position)
        .unwrap();
      self.mouse.delta_x = 0.0;
      self.mouse.delta_y = 0.0;
    }
  }

  pub fn update(&mut self, elapsed_time: &Duration, n_image: usize) {
    self.camera.handle_keys(&self.pressed_keys, elapsed_time);
    self.camera.handle_mouse_movement(&self.mouse);
    self.mouse.delta_x = 0.0;
    self.mouse.delta_y = 0.0;

    let proj = self.camera.get_projection_matrix(self.aspect_ratio);
    let view = self.camera.get_view_matrix();

    let rotation =
      elapsed_time.as_secs() as f64 + elapsed_time.subsec_nanos() as f64 / 1_000_000_000.0;
    let rotation = Matrix3::from_angle_y(Rad(rotation as f32));
    let scale = Matrix4::from_scale(1.0);
    let model = scale * Matrix4::from(rotation);

    let matrix = proj * view * model;
    self.program.update(elapsed_time, n_image, matrix);
  }

  pub fn handle_window_update(&mut self) {
    let new_window_dimensions = self.surface.window().inner_size();

    self.window_half_screen_position = PhysicalPosition {
      x: new_window_dimensions.width as f32 / 2.0,
      y: new_window_dimensions.height as f32 / 2.0,
    };

    self.aspect_ratio = new_window_dimensions.width as f32 / new_window_dimensions.height as f32;

    self.program.handle_window_update(new_window_dimensions);
  }
}
