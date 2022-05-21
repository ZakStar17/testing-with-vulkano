use std::f32::consts::PI;

use cgmath::{InnerSpace, Matrix4, PerspectiveFov, Point3, Rad, Vector3};

use crate::{app::Mouse, Keys, Pressed};

const HALF_PI: f32 = PI / 2.0;

pub struct Camera {
  pub position: Point3<f32>,
  pub front: Vector3<f32>,
  pub up: Vector3<f32>,
  pub yaw: f32,
  pub pitch: f32,
  pub speed: f32,
  pub sensitivity: f32,
  pub fov: f32,
  aspect_ratio: f32,
  projection_matrix: Matrix4<f32>,
}

impl Camera {
  pub fn new(position: Point3<f32>, speed: f32, fov: f32, aspect_ratio: f32) -> Camera {
    Camera {
      position,
      front: Vector3::new(0.0, 0.0, -1.0),
      up: Vector3::new(0.0, 1.0, 0.0),
      yaw: 0.0,
      pitch: 0.0,
      speed,
      sensitivity: 0.003,
      fov,
      aspect_ratio,
      projection_matrix: get_projection_matrix(fov, aspect_ratio),
    }
  }

  pub fn get_view_matrix(&self) -> Matrix4<f32> {
    Matrix4::look_at_rh(self.position, self.position + self.front, self.up)
  }

  pub fn get_projection_matrix(&self) -> Matrix4<f32> {
    self.projection_matrix
  }

  pub fn get_projection_view(&self) -> Matrix4<f32> {
    self.get_projection_matrix() * self.get_view_matrix()
  }

  pub fn set_aspect_ratio(&mut self, value: f32) {
    self.aspect_ratio = value;
    self.projection_matrix = get_projection_matrix(self.fov, value);
  }

  pub fn handle_zoom(&mut self, amount: f32) {
    self.fov -= amount * 0.07;
    if self.fov > PI / 1.1 {
      self.fov = PI / 1.1;
    } else if self.fov < PI / 15.0 {
      self.fov = PI / 15.0;
    }

    self.projection_matrix = get_projection_matrix(self.fov, self.aspect_ratio);
  }

  pub fn handle_mouse_movement(&mut self, mouse: &Mouse) {
    self.yaw += mouse.delta_x * self.sensitivity;
    self.pitch += mouse.delta_y * self.sensitivity;

    if self.pitch > HALF_PI - 0.1 {
      self.pitch = HALF_PI - 0.1;
    } else if self.pitch < -HALF_PI + 0.1 {
      self.pitch = -HALF_PI + 0.1;
    }

    self.front = {
      let pitch_cos = self.pitch.cos();
      let x = self.yaw.cos() * pitch_cos;
      let y = self.pitch.sin();
      let z = self.yaw.sin() * pitch_cos;
      Vector3::new(x, y, z)
    };
  }

  pub fn handle_keys(&mut self, keys: &Keys, delta_time: &std::time::Duration) {
    let delta_speed = self.speed * delta_time.as_secs_f32();
    if keys.w == Pressed {
      self.position += self.front * delta_speed;
    }
    if keys.a == Pressed {
      self.position -= self.front.cross(self.up).normalize() * delta_speed;
    }
    if keys.s == Pressed {
      self.position -= self.front * delta_speed;
    }
    if keys.d == Pressed {
      self.position += self.front.cross(self.up).normalize() * delta_speed;
    }
    if keys.space == Pressed {
      self.position -= self.up * delta_speed;
    }
    if keys.l_ctrl == Pressed {
      self.position += self.up * delta_speed;
    }
  }
}

fn get_projection_matrix(fov: f32, aspect_ratio: f32) -> Matrix4<f32> {
  PerspectiveFov {
    fovy: Rad(fov),
    aspect: aspect_ratio,
    far: 1000.0,
    near: 0.1,
  }
  .into()
}
