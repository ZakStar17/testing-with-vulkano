use bytemuck::{Pod, Zeroable};
use vulkano::impl_vertex;

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, Zeroable, Pod)]
pub struct Vertex3d {
  pub position: [f32; 3],
}

impl_vertex!(Vertex3d, position);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct MatrixInstance {
  pub matrix: [[f32; 4]; 4],
}
impl_vertex!(MatrixInstance, matrix);
