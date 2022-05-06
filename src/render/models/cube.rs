use crate::render::models::Model;
use crate::render::Vertex3d;

pub struct CubeModel {
  vertices: Vec<Vertex3d>,
  indices: Vec<u16>,
}

impl CubeModel {
  pub fn new() -> Self {
    Self {
      vertices: vec![
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
      ],
      indices: vec![
        0, 1, 3, 3, 1, 2, 1, 5, 2, 2, 5, 6, 5, 4, 6, 6, 4, 7, 4, 0, 7, 7, 0, 3, 3, 2, 7, 7, 2, 6,
        4, 5, 0, 0, 5, 1,
      ],
    }
  }
}

impl Model<Vertex3d> for CubeModel {
  fn get_vertices(&self) -> &Vec<Vertex3d> {
    &self.vertices
  }

  fn get_indices(&self) -> &Vec<u16> {
    &self.indices
  }
}
