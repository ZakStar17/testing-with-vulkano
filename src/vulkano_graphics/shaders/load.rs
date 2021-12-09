use crate::vulkano_graphics::shaders::simple_cube::{fs, vs};
use std::sync::Arc;
use vulkano::device::Device;

pub struct Shaders {
  pub vertex: vs::Shader,
  pub fragment: fs::Shader,
}

impl Shaders {
  pub fn load(device: Arc<Device>) -> Self {
    Self {
      vertex: vs::Shader::load(device.clone()).unwrap(),
      fragment: fs::Shader::load(device).unwrap(),
    }
  }
}
