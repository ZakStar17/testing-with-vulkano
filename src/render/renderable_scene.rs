use crate::{
  game_objects::RenderableIn3d,
  render::{
    models::{CubeModel, Model, SquareModel},
    vertex_data::Vertex3d,
  },
  Scene,
};
use cgmath::Matrix4;
use std::iter::Iterator;

// ordered abstraction of all scene objects
// todo: should be a macro to implement all Scene objects automatically
pub struct RenderableScene;

impl RenderableScene {
  // this function took me 2 hours because of a simple error
  pub fn into_matrices<'a>(scene: &'a Scene) -> impl Iterator<Item = Matrix4<f32>> + '_ {
    // transform into iterator
    let renderable_cubes = scene.cubes.iter().map(|cube| cube as &dyn RenderableIn3d);

    let renderable_squares = scene
      .squares
      .iter()
      .map(|square| square as &dyn RenderableIn3d);

    renderable_cubes
      .chain(renderable_squares)
      .map(|obj| obj.get_model_matrix())
  }

  pub fn instance_count_per_model(scene: &Scene) -> Vec<usize> {
    vec![scene.cubes.len(), scene.squares.len()]
  }

  pub fn get_models() -> Vec<Box<dyn Model<Vertex3d>>> {
    let cube_model: Box<dyn Model<Vertex3d>> = Box::new(CubeModel::new());
    let square_model: Box<dyn Model<Vertex3d>> = Box::new(SquareModel::new());
    vec![cube_model, square_model]
  }
}
