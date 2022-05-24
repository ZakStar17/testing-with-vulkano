use crate::{
  render::{
    renderable_scene::RenderableScene,
    vertex_data::{MatrixInstance, Vertex3d},
    vulkano_objects,
    vulkano_objects::buffers::Buffers,
  },
  Scene,
};
use cgmath::Matrix4;
use std::sync::Arc;
use vulkano::{
  command_buffer::PrimaryAutoCommandBuffer,
  device::{Device, Queue},
  pipeline::GraphicsPipeline,
  render_pass::Framebuffer,
};

// responsible for managing data between existing buffers and command_buffers
pub struct BufferContainer {
  buffers: Buffers<Vertex3d, MatrixInstance>,
  command_buffers: Vec<Arc<PrimaryAutoCommandBuffer>>,
  instance_count_per_model_cache: Vec<u32>,
}

impl BufferContainer {
  pub fn new(
    device: Arc<Device>,
    pipeline: Arc<GraphicsPipeline>,
    framebuffers: &Vec<Arc<Framebuffer>>,
    queue: Arc<Queue>,
    scene: &Scene,
  ) -> Self {
    // uniform buffer count is assigned to the number of image, in this case the number of framebuffers
    let buffers = Buffers::<Vertex3d, MatrixInstance>::initialize(
      device.clone(),
      framebuffers.len(),
      queue.clone(),
      &RenderableScene::get_models(),
      1024,
    );

    let instance_count_per_model: Vec<u32> = RenderableScene::instance_count_per_model(scene)
      .drain(0..)
      .map(|n| n as u32)
      .collect();

    let command_buffers = vulkano_objects::command_buffers::create(
      device.clone(),
      queue,
      pipeline,
      &framebuffers,
      &buffers,
      &instance_count_per_model,
    );

    Self {
      buffers,
      command_buffers,
      instance_count_per_model_cache: instance_count_per_model.clone(),
    }
  }

  pub fn handle_window_resize(
    &mut self,
    device: Arc<Device>,
    pipeline: Arc<GraphicsPipeline>,
    framebuffers: &Vec<Arc<Framebuffer>>,
    queue: Arc<Queue>,
  ) {
    self.command_buffers = vulkano_objects::command_buffers::create(
      device,
      queue,
      pipeline,
      framebuffers,
      &self.buffers,
      &self.instance_count_per_model_cache,
    )
  }

  pub fn update_matrices(&mut self, buffer_i: usize, scene: &Scene, projection_view: Matrix4<f32>) {
    self.buffers.update_matrices(
      buffer_i,
      RenderableScene::into_matrices(scene)
        .map(|model| MatrixInstance {
          matrix: (projection_view * model).into(),
        })
        .collect(),
    )
  }

  pub fn get_command_buffer(&self, buffer_i: usize) -> Arc<PrimaryAutoCommandBuffer> {
    self.command_buffers[buffer_i].clone()
  }
}
