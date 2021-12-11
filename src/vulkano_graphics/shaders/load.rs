use crate::vulkano_graphics::shaders::simple_cube::{fs, vs};
use std::sync::Arc;
use vulkano::device::Device;
use vulkano::shader::ShaderModule;

pub struct Shaders {
  pub vertex: Arc<ShaderModule>,
  pub fragment: Arc<ShaderModule>,
}

impl Shaders {
  pub fn load(device: Arc<Device>) -> Self {
    Self {
      vertex: vs::load(device.clone()).unwrap(),
      fragment: fs::load(device).unwrap(),
    }
  }
}
