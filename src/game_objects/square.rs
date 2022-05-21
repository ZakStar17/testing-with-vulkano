use crate::game_objects::{Renderable3dObject, RenderableIn3d};
use cgmath::{Matrix4, Point3};
use rand::Rng;

pub struct Square {
  pub color: [f32; 3],
  pub object: Renderable3dObject,
  pub speed: f32,
}

impl Square {
  pub fn new() -> Self {
    Self {
      color: [1.0, 0.0, 0.0],
      object: Renderable3dObject::new(Point3::new(0.0, 0.0, 0.0)),
      speed: 1.3,
    }
  }

  pub fn change_to_random_color(&mut self) {
    let get_random_float = || rand::thread_rng().gen_range(0..100) as f32 / 100.0;
    self.color = [get_random_float(), get_random_float(), get_random_float()];
  }

  pub fn move_right(&mut self, seconds_passed: f32) {
    self
      .object
      .move_relative(Point3::new(seconds_passed * self.speed, 0.0, 0.0));
  }

  pub fn move_left(&mut self, seconds_passed: f32) {
    self
      .object
      .move_relative(Point3::new(seconds_passed * -self.speed, 0.0, 0.0));
  }

  pub fn move_up(&mut self, seconds_passed: f32) {
    self
      .object
      .move_relative(Point3::new(0.0, seconds_passed * -self.speed, 0.0));
  }

  pub fn move_down(&mut self, seconds_passed: f32) {
    self
      .object
      .move_relative(Point3::new(0.0, seconds_passed * self.speed, 0.0));
  }
}

impl RenderableIn3d for Square {
  fn get_model_matrix(&self) -> Matrix4<f32> {
    self.object.get_model_matrix()
  }
}
