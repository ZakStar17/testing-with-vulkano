pub mod vs {
  vulkano_shaders::shader! {
      ty: "vertex",
      path: "src/render/shaders/simple_cube/vertex.glsl",
      types_meta: {
        use bytemuck::{Pod, Zeroable};
        #[derive(Clone, Copy, Zeroable, Pod)]
      },
  }
}

pub mod fs {
  vulkano_shaders::shader! {
      ty: "fragment",
      path: "src/render/shaders/simple_cube/fragment.glsl"
  }
}
