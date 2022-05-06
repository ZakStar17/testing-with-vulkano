use vulkano::buffer::BufferContents;

pub trait UniformShader<U: BufferContents> {
  fn get_initial_uniform_data() -> U;
}