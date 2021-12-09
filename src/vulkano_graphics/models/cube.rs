use crate::vulkano_graphics::shaders::vertex_data::Vertex3d;

pub const VERTICES: [Vertex3d; 8] = [
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
];

pub const INDICES: [u16; 36] = [
  0, 1, 3, 3, 1, 2, 1, 5, 2, 2, 5, 6, 5, 4, 6, 6, 4, 7, 4, 0, 7, 7, 0, 3, 3, 2, 7, 7, 2, 6, 4, 5,
  0, 0, 5, 1,
];
