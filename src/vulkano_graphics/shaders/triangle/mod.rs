pub mod vs {
  vulkano_shaders::shader! {
      ty: "vertex",
      path: "src/vulkano_graphics/shaders/triangle/vertex.glsl"
  }
}

pub mod fs {
  vulkano_shaders::shader! {
      ty: "fragment",
      path: "src/vulkano_graphics/shaders/triangle/fragment.glsl"
  }
}