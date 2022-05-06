use crate::render::Vertex3d;
use crate::render::models::SquareModel;
use crate::render::models::CubeModel;
use crate::render::models::Model;
use crate::render::shaders::UniformShader;
use crate::render::vulkano_objects;
use crate::render::vulkano_objects::buffers::ImmutableBuffers;
use std::sync::Arc;
use vulkano::buffer::BufferContents;
use vulkano::command_buffer::PrimaryAutoCommandBuffer;
use vulkano::descriptor_set::layout::DescriptorSetLayout;
use vulkano::device::Device;
use vulkano::device::Queue;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::render_pass::Framebuffer;

// todo: Come up with a better name for this
pub struct BufferContainer<U: BufferContents + Copy> {
  pub buffers: ImmutableBuffers<Vertex3d, U>,
  pub command_buffers: Vec<Arc<PrimaryAutoCommandBuffer>>,
}

impl<U: BufferContents + Copy> BufferContainer<U> {
  pub fn new<S: UniformShader<U>>(
    device: Arc<Device>,
    pipeline: Arc<GraphicsPipeline>,
    descriptor_set_layout: Arc<DescriptorSetLayout>,
    framebuffers: &Vec<Arc<Framebuffer>>,
    queue: Arc<Queue>,
  ) -> Self {
    let models: Vec<Box<dyn Model<Vertex3d>>> = vec![Box::new(CubeModel::new()), Box::new(SquareModel::new())];

    let buffers = ImmutableBuffers::initialize::<S>(
      device.clone(),
      descriptor_set_layout,
      framebuffers.len(),
      queue.clone(),
      &models
    );

    let command_buffers = vulkano_objects::command_buffers::create_simple_command_buffers(
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
    self.command_buffers = vulkano_objects::command_buffers::create_simple_command_buffers(
      device,
      queue,
      pipeline,
      framebuffers,
      &self.buffers,
    )
  }
}