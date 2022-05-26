pub mod vs {
  vulkano_shaders::shader! {
      ty: "vertex",
      path: "src/render/shaders/single_colored/vertex.glsl",
  }
}

pub mod fs {
  vulkano_shaders::shader! {
      ty: "fragment",
      path: "src/render/shaders/single_colored/fragment.glsl"
  }
}
