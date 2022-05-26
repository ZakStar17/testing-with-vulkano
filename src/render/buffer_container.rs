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
use vulkano::buffer::TypedBufferAccess;
use vulkano::device::physical::QueueFamily;
use vulkano::{
  command_buffer::PrimaryAutoCommandBuffer,
  device::{Device, Queue},
  pipeline::GraphicsPipeline,
  render_pass::Framebuffer,
};

pub struct CommandBuffers {
  pub main: Vec<Arc<PrimaryAutoCommandBuffer>>,
  pub instance_copy: Vec<Arc<PrimaryAutoCommandBuffer>>,
}

impl CommandBuffers {
  pub fn create(
    device: Arc<Device>,
    pipeline: Arc<GraphicsPipeline>,
    framebuffers: &Vec<Arc<Framebuffer>>,
    queue: Arc<Queue>,
    buffers: &Buffers<Vertex3d, MatrixInstance>,
    instance_count_per_model: &Vec<u32>,
  ) -> Self {
    let main = vulkano_objects::command_buffers::create_main(
      device.clone(),
      queue.clone(),
      pipeline,
      &framebuffers,
      &buffers,
      &instance_count_per_model,
    );

    let instance_buffer = &buffers.get_main().instance;
    let instance_copy = (0..main.len())
      .map(|i| {
        vulkano_objects::command_buffers::create_slice_copy(
          device.clone(),
          queue.clone(),
          buffers.get_instance_source(i),
          0,
          instance_buffer[i].clone(),
          0,
          instance_buffer[i].len(),
        )
      })
      .collect();

    Self {
      main,
      instance_copy,
    }
  }

  pub fn recreate_main(
    &mut self,
    device: Arc<Device>,
    queue: Arc<Queue>,
    pipeline: Arc<GraphicsPipeline>,
    framebuffers: &Vec<Arc<Framebuffer>>,
    buffers: &Buffers<Vertex3d, MatrixInstance>,
    instance_count_per_model: &Vec<u32>,
  ) {
    self.main = vulkano_objects::command_buffers::create_main(
      device.clone(),
      queue,
      pipeline,
      &framebuffers,
      &buffers,
      &instance_count_per_model,
    );
  }
}

// responsible for managing data between existing buffers and command_buffers
pub struct BufferContainer {
  command_buffers: CommandBuffers,
  buffers: Buffers<Vertex3d, MatrixInstance>,
  instance_count_per_model_cache: Vec<u32>,
}

impl BufferContainer {
  pub fn new(
    device: Arc<Device>,
    pipeline: Arc<GraphicsPipeline>,
    framebuffers: &Vec<Arc<Framebuffer>>,
    queue: Arc<Queue>,
    scene: &Scene,
    queue_families: [QueueFamily; 1],
  ) -> Self {
    // uniform buffer count is assigned to the number of image, in this case the number of framebuffers
    let buffers = Buffers::<Vertex3d, MatrixInstance>::initialize(
      device.clone(),
      framebuffers.len(),
      queue.clone(),
      &RenderableScene::get_models(),
      1024,
      queue_families,
    );

    let instance_count_per_model: Vec<u32> = RenderableScene::instance_count_per_model(scene)
      .drain(0..)
      .map(|n| n as u32)
      .collect();

    let command_buffers = CommandBuffers::create(
      device,
      pipeline,
      framebuffers,
      queue,
      &buffers,
      &instance_count_per_model,
    );

    Self {
      buffers,
      command_buffers,
      instance_count_per_model_cache: instance_count_per_model,
    }
  }

  pub fn handle_window_resize(
    &mut self,
    device: Arc<Device>,
    pipeline: Arc<GraphicsPipeline>,
    framebuffers: &Vec<Arc<Framebuffer>>,
    queue: Arc<Queue>,
  ) {
    self.command_buffers.recreate_main(
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

  pub fn command_buffers(&self) -> &CommandBuffers {
    &self.command_buffers
  }
}
