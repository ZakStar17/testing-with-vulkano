use crate::render::{models::Model, Vertex3d};

pub struct SquareModel {
  vertices: Vec<Vertex3d>,
  indices: Vec<u16>,
}

impl SquareModel {
  pub fn new() -> Self {
    Self {
      vertices: vec![
        Vertex3d {
          position: [-0.5, -0.5, 0.0],
        },
        Vertex3d {
          position: [0.5, -0.5, 0.0],
        },
        Vertex3d {
          position: [-0.5, 0.5, 0.0],
        },
        Vertex3d {
          position: [0.5, 0.5, 0.0],
        },
      ],
      // draw clockwise and counter-clockwise
      // 0  1
      // 2  3
      indices: vec![0, 2, 1, 1, 2, 3, 0, 1, 2, 1, 3, 2],
    }
  }
}

impl Model<Vertex3d> for SquareModel {
  fn get_vertices(&self) -> &Vec<Vertex3d> {
    &self.vertices
  }

  fn get_indices(&self) -> &Vec<u16> {
    &self.indices
  }
}
