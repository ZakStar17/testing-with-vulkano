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

