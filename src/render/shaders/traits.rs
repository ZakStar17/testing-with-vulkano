use std::mem::size_of;
use vulkano::buffer::BufferContents;

pub trait UniformShader<U: BufferContents> {
  fn get_initial_uniform_data() -> U;

  fn get_initial_uniform_bytes() -> Vec<u8> {
    vec![0; size_of::<U>()]
  }
}
