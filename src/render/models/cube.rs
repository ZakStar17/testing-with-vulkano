use crate::render::models::Model;
use crate::render::shaders::simple_cube;
use crate::render::Vertex3d;
use cgmath::Matrix4;

pub struct CubeModel;

type UniformData = simple_cube::vs::ty::Data;

impl Model<Vertex3d, UniformData> for CubeModel {
  fn get_vertices() -> Vec<Vertex3d> {
    vec![
      Vertex3d {
        position: [-1.0, -1.0, -1.0],
      },
      Vertex3d {
        position: [1.0, -1.0, -1.0],
      },
      Vertex3d {
        position: [1.0, 1.0, -1.0],
      },
      Vertex3d {
        position: [-1.0, 1.0, -1.0],
      },
      Vertex3d {
        position: [-1.0, -1.0, 1.0],
      },
      Vertex3d {
        position: [1.0, -1.0, 1.0],
      },
      Vertex3d {
        position: [1.0, 1.0, 1.0],
      },
      Vertex3d {
        position: [-1.0, 1.0, 1.0],
      },
    ]
  }

  fn get_indices() -> Vec<u16> {
    vec![
      0, 1, 3, 3, 1, 2, 1, 5, 2, 2, 5, 6, 5, 4, 6, 6, 4, 7, 4, 0, 7, 7, 0, 3, 3, 2, 7, 7, 2, 6, 4,
      5, 0, 0, 5, 1,
    ]
  }

  fn get_initial_uniform_data() -> UniformData {
    UniformData {
      color: [0.0; 3],
      matrix: Matrix4::from_scale(1.0).into(),
    }
  }
}
