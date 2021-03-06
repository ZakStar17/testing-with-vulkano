use crate::other::add_points;
use cgmath::{EuclideanSpace, Euler, Matrix4, Point3, Rad};

pub trait RenderableIn3d {
  fn get_model_matrix(&self) -> Matrix4<f32>;
}

/// Object information suitable for rendering in 3D. Caches certain matrices
/// in order to perform less calculations while rendering.
pub struct Renderable3dObject {
  position: Point3<f32>,
  translation_matrix: Matrix4<f32>,
  rotation: Euler<Rad<f32>>,
  rotation_matrix: Matrix4<f32>,
  scale: f32,
  scale_matrix: Matrix4<f32>,
  model_matrix: Matrix4<f32>,
}

impl Renderable3dObject {
  pub fn new(position: Point3<f32>) -> Self {
    let rotation = Euler {
      x: Rad(0.0),
      y: Rad(0.0),
      z: Rad(0.0),
    };
    let scale = 0.5;

    let translation_matrix = Matrix4::from_translation(position.to_vec());
    let rotation_matrix = Matrix4::from(rotation);
    let scale_matrix = Matrix4::from_scale(scale);

    Self {
      position,
      translation_matrix,
      rotation,
      rotation_matrix,
      scale,
      scale_matrix,
      model_matrix: translation_matrix * rotation_matrix * scale_matrix,
    }
  }

  pub fn from_full(position: Point3<f32>, rotation: Euler<Rad<f32>>, scale: f32) -> Self {
    let translation_matrix = Matrix4::from_translation(position.to_vec());
    let rotation_matrix = Matrix4::from(rotation);
    let scale_matrix = Matrix4::from_scale(scale);

    Self {
      position,
      translation_matrix,
      rotation,
      rotation_matrix,
      scale,
      scale_matrix,
      model_matrix: translation_matrix * rotation_matrix * scale_matrix,
    }
  }

  pub fn get_position(&self) -> Point3<f32> {
    self.position
  }

  pub fn get_rotation(&self) -> Euler<Rad<f32>> {
    self.rotation
  }

  pub fn get_scale(&self) -> f32 {
    self.scale
  }

  pub fn move_relative(&mut self, relative: Point3<f32>) {
    self.position = add_points(self.position, relative);
    self.update_translation_matrix();
    self.update_model_matrix();
  }

  // todo: too much ambiguous code
  pub fn move_relative_x(&mut self, relative_x: f32) {
    self.position.x += relative_x;
    self.update_translation_matrix();
    self.update_model_matrix();
  }

  pub fn move_relative_y(&mut self, relative_y: f32) {
    self.position.y += relative_y;
    self.update_translation_matrix();
    self.update_model_matrix();
  }

  pub fn move_relative_z(&mut self, relative_z: f32) {
    self.position.z += relative_z;
    self.update_translation_matrix();
    self.update_model_matrix();
  }

  pub fn r#move(&mut self, new_position: Point3<f32>) {
    self.position = new_position;
    self.update_translation_matrix();
    self.update_model_matrix();
  }

  pub fn rotate(&mut self, new_rotation: Euler<Rad<f32>>) {
    self.rotation = new_rotation;
    self.update_rotation_matrix();
    self.update_model_matrix();
  }

  pub fn scale(&mut self, new_scale: f32) {
    self.scale = new_scale;
    self.update_scale_matrix();
    self.update_model_matrix();
  }

  pub fn move_and_rotate(&mut self, new_position: Point3<f32>, new_rotation: Euler<Rad<f32>>) {
    self.position = new_position;
    self.rotation = new_rotation;
    self.update_translation_matrix();
    self.update_rotation_matrix();
    self.update_model_matrix();
  }

  pub fn update(
    &mut self,
    new_position: Point3<f32>,
    new_rotation: Euler<Rad<f32>>,
    new_scale: f32,
  ) {
    self.position = new_position;
    self.rotation = new_rotation;
    self.scale = new_scale;
    self.update_translation_matrix();
    self.update_rotation_matrix();
    self.update_scale_matrix();
    self.update_model_matrix();
  }

  fn update_translation_matrix(&mut self) {
    self.translation_matrix = Matrix4::from_translation(self.position.to_vec());
  }

  fn update_rotation_matrix(&mut self) {
    self.rotation_matrix = Matrix4::from(self.rotation);
  }

  fn update_scale_matrix(&mut self) {
    self.scale_matrix = Matrix4::from_scale(self.scale);
  }

  fn update_model_matrix(&mut self) {
    self.model_matrix = self.translation_matrix * self.rotation_matrix * self.scale_matrix;
  }
}

impl RenderableIn3d for Renderable3dObject {
  fn get_model_matrix(&self) -> Matrix4<f32> {
    self.model_matrix
  }
}
