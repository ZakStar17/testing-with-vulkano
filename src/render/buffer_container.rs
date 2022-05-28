use crate::{
  render::{
    renderable_scene::RenderableScene,
    vertex_data::{MatrixInstance, Vertex3d},
    vulkano_objects,
    vulkano_objects::{buffers::Buffers, physical_device::QueueFamilies, Queues},
  },
  Scene, GENERATE_CUBES,
};
use std::sync::Arc;
use vulkano::{
  command_buffer::PrimaryAutoCommandBuffer,
  descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet},
  device::{Device, Queue},
  pipeline::{ComputePipeline, GraphicsPipeline, Pipeline},
  render_pass::Framebuffer,
};

pub struct CommandBuffers {
  pub main: Vec<Arc<PrimaryAutoCommandBuffer>>,
}

impl CommandBuffers {
  pub fn create(
    device: Arc<Device>,
    queues: &Queues,
    pipeline: Arc<GraphicsPipeline>,
    framebuffers: &Vec<Arc<Framebuffer>>,
    buffers: &Buffers<Vertex3d, MatrixInstance>,
    instance_count_per_model: &Vec<u32>,
  ) -> Self {
    let main = vulkano_objects::command_buffers::create_main(
      device.clone(),
      queues.graphics.clone(),
      pipeline,
      &framebuffers,
      &buffers,
      &instance_count_per_model,
    );

    Self { main }
  }

  pub fn recreate_main(
    &mut self,
    device: Arc<Device>,
    graphics_queue: Arc<Queue>,
    pipeline: Arc<GraphicsPipeline>,
    framebuffers: &Vec<Arc<Framebuffer>>,
    buffers: &Buffers<Vertex3d, MatrixInstance>,
    instance_count_per_model: &Vec<u32>,
  ) {
    self.main = vulkano_objects::command_buffers::create_main(
      device.clone(),
      graphics_queue,
      pipeline,
      &framebuffers,
      &buffers,
      &instance_count_per_model,
    );
  }
}

pub struct DescriptorSets {
  pub instance: Vec<Arc<PersistentDescriptorSet>>,
}

/// responsible for managing data between existing buffers and command_buffers
pub struct BufferContainer {
  command_buffers: CommandBuffers,
  buffers: Buffers<Vertex3d, MatrixInstance>,
  descriptor_sets: DescriptorSets,
  instance_count_per_model_cache: Vec<u32>,
}

impl BufferContainer {
  pub fn new(
    device: Arc<Device>,
    queue_families: &QueueFamilies,
    queues: &Queues,
    graphics_pipeline: Arc<GraphicsPipeline>,
    compute_pipeline: Arc<ComputePipeline>,
    framebuffers: &Vec<Arc<Framebuffer>>,
    scene: &Scene,
  ) -> Self {
    let max_instances = if let Some(value) = GENERATE_CUBES {
      (value * value * value) + 256
    } else {
      256
    };

    // uniform buffer count is assigned to the number of image, in this case the number of framebuffers
    let buffers = Buffers::<Vertex3d, MatrixInstance>::initialize(
      device.clone(),
      queue_families,
      queues.transfers.clone(),
      framebuffers.len(),
      &RenderableScene::get_models(),
      max_instances,
    );

    let layout = compute_pipeline.layout().set_layouts().get(0).unwrap();
    let descriptor_sets = DescriptorSets {
      instance: (0..3)
        .map(|i| {
          PersistentDescriptorSet::new(
            layout.clone(),
            [
              WriteDescriptorSet::buffer(0, buffers.get_instance_source_model(i).clone()),
              WriteDescriptorSet::buffer(1, buffers.get_main().instance.clone()),
            ],
          )
          .unwrap()
        })
        .collect(),
    };

    let instance_count_per_model: Vec<u32> = RenderableScene::instance_count_per_model(scene)
      .drain(0..)
      .map(|n| n as u32)
      .collect();

    let command_buffers = CommandBuffers::create(
      device,
      queues,
      graphics_pipeline,
      framebuffers,
      &buffers,
      &instance_count_per_model,
    );

    Self {
      buffers,
      descriptor_sets,
      command_buffers,
      instance_count_per_model_cache: instance_count_per_model,
    }
  }

  pub fn handle_window_resize(
    &mut self,
    device: Arc<Device>,
    graphics_queue: Arc<Queue>,
    pipeline: Arc<GraphicsPipeline>,
    framebuffers: &Vec<Arc<Framebuffer>>,
  ) {
    self.command_buffers.recreate_main(
      device,
      graphics_queue,
      pipeline,
      framebuffers,
      &self.buffers,
      &self.instance_count_per_model_cache,
    )
  }

  pub fn update_buffer_models(&mut self, buffer_i: usize, scene: &Scene) {
    self.buffers.update_instance_source_models(
      buffer_i,
      RenderableScene::into_matrices(scene)
        .map(|model| MatrixInstance {
          matrix: model.into(),
        })
        .collect(),
    )
  }

  pub fn command_buffers(&self) -> &CommandBuffers {
    &self.command_buffers
  }

  pub fn descriptor_sets(&self) -> &DescriptorSets {
    &self.descriptor_sets
  }
}
