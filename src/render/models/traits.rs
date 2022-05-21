use vulkano::buffer::BufferContents;

pub trait Model<V: BufferContents> {
  fn get_indices(&self) -> &Vec<u16>;
  fn get_vertices(&self) -> &Vec<V>;
}
