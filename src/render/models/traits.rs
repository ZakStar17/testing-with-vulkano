use vulkano::buffer::BufferContents;

/// Main trait of every model.
/// 
/// Has functions for retrieving indices and vertices
pub trait Model<V: BufferContents> {
  fn get_indices(&self) -> &Vec<u16>;
  fn get_vertices(&self) -> &Vec<V>;
}
