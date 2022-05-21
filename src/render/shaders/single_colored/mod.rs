use crate::render::shaders::UniformShader;
use bytemuck::{Pod, Zeroable};
use cgmath::Matrix4;

pub mod vs {
  vulkano_shaders::shader! {
      ty: "vertex",
      path: "src/render/shaders/single_colored/vertex.glsl",
      types_meta: {
        use bytemuck::{Pod, Zeroable};
        use serde::{Serialize, Deserialize};
        #[derive(Clone, Copy, Zeroable, Pod, Serialize, Deserialize)]
      },
  }
}

pub mod fs {
  vulkano_shaders::shader! {
      ty: "fragment",
      path: "src/render/shaders/single_colored/fragment.glsl"
  }
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
pub struct S;

impl UniformShader<vs::ty::Data> for S {
  fn get_initial_uniform_data() -> vs::ty::Data {
    vs::ty::Data {
      color: [0.0; 3],
      matrix: Matrix4::from_scale(0.0).into(),
    }
  }
}
// Matrix4 [[0.0, 0.0, 0.50009996, 0.5], [0.0, 1.1826111, 0.0, 0.0], [0.8869583, 0.0, 0.0, 0.0], [0.0, 2.3652222, 4.8009796, 5.0]]
