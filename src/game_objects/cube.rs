use cgmath::{Matrix4, Point3};

use crate::game_objects::{Renderable3dObject, RenderableIn3d};

pub struct Cube {
  object: Renderable3dObject,
}

impl Cube {
  pub fn new(position: Point3<f32>) -> Self {
    Self {
      object: Renderable3dObject::new(position),
    }
  }

  pub fn from_full(object: Renderable3dObject) -> Self {
    Self { object }
  }

  // pub fn change_to_random_color(&mut self) {
  //   let get_random_float = || rand::thread_rng().gen_range(0..100) as f32 / 100.0;
  //   self.color = [get_random_float(), get_random_float(), get_random_float()];
  // }

  pub fn as_renderable(&mut self) -> &mut Renderable3dObject {
    &mut self.object
  }
}

impl RenderableIn3d for Cube {
  fn get_model_matrix(&self) -> Matrix4<f32> {
    self.object.get_model_matrix()
  }
}
