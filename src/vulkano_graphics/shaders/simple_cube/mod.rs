pub mod vs {
  vulkano_shaders::shader! {
      ty: "vertex",
      path: "src/vulkano_graphics/shaders/simple_cube/vertex.glsl"
  }
}

pub mod fs {
  vulkano_shaders::shader! {
      ty: "fragment",
      path: "src/vulkano_graphics/shaders/simple_cube/fragment.glsl"
  }
}