use crate::render::{
  models::{CubeModel, Model, SquareModel},
  shaders::UniformShader,
  vulkano_objects,
  vulkano_objects::buffers::ImmutableBuffers,
  Vertex3d,
};
use serde::Serialize;
use std::sync::Arc;
use vulkano::{
  buffer::BufferContents,
  command_buffer::PrimaryAutoCommandBuffer,
  descriptor_set::layout::DescriptorSetLayout,
  device::{Device, Queue},
  pipeline::GraphicsPipeline,
  render_pass::Framebuffer,
};

// data from different models gets stored in the same buffer, so this is meant to be
// an abstraction between concrete models and arrays of models
pub struct BufferContainer {
  pub buffers: ImmutableBuffers<Vertex3d>,
  pub command_buffers: Vec<Arc<PrimaryAutoCommandBuffer>>,
}

impl BufferContainer {
  pub fn new<U: BufferContents + Copy, S: UniformShader<U>>(
    device: Arc<Device>,
    pipeline: Arc<GraphicsPipeline>,
    descriptor_set_layout: Arc<DescriptorSetLayout>,
    framebuffers: &Vec<Arc<Framebuffer>>,
    queue: Arc<Queue>,
  ) -> Self {
    // the ordering is important while assigning uniforms
    let models: Vec<Box<dyn Model<Vertex3d>>> = vec![
      Box::new(CubeModel::new()),
      Box::new(SquareModel::new()),
      Box::new(CubeModel::new()),
    ];

    // uniform buffer count is assigned to the number of image, in this case the number of framebuffers
    let buffers = ImmutableBuffers::initialize::<U, S>(
      device.clone(),
      descriptor_set_layout,
      framebuffers.len(),
      queue.clone(),
      &models,
    );

    let command_buffers = vulkano_objects::command_buffers::create(
      device.clone(),
      queue,
      pipeline,
      &framebuffers,
      &buffers,
    );

    Self {
      buffers,
      command_buffers,
    }
  }

  pub fn handle_window_resize(
    &mut self,
    device: Arc<Device>,
    pipeline: Arc<GraphicsPipeline>,
    framebuffers: &Vec<Arc<Framebuffer>>,
    queue: Arc<Queue>,
  ) {
    self.command_buffers =
      vulkano_objects::command_buffers::create(device, queue, pipeline, framebuffers, &self.buffers)
  }

  pub fn update_uniform<U: BufferContents + Copy + Serialize>(
    &mut self,
    buffer_i: usize,
    cube1_data: U,
    square_data: U,
    cube2_data: U,
  ) {
    self.buffers.write_to_uniform(
      buffer_i,
      vec![(0, cube1_data), (1, square_data), (2, cube2_data)],
    );
  }
}
