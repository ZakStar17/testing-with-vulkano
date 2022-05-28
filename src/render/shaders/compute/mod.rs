pub mod instance {
  vulkano_shaders::shader! {
      ty: "compute",
      path: "src/render/shaders/compute/instance.glsl",
  }
}
