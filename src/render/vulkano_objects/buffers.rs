use crate::render::{models::Model, vulkano_objects::QueueFamilies};
use bytemuck::Pod;
use std::sync::Arc;
use vulkano::{
  buffer::{BufferContents, BufferUsage, CpuAccessibleBuffer, DeviceLocalBuffer, ImmutableBuffer},
  command_buffer::{CommandBufferExecFuture, PrimaryAutoCommandBuffer},
  device::{Device, Queue},
  sync::{GpuFuture, NowFuture},
};

// used in the main command buffer
pub struct MainBuffers<V: BufferContents + Pod, I: BufferContents + Pod> {
  pub vertex: Arc<ImmutableBuffer<[V]>>,
  pub index: Arc<ImmutableBuffer<[u16]>>,
  pub instance: Arc<DeviceLocalBuffer<[I]>>,
  pub model_lengths: Vec<(u32, i32)>,
}

impl<V: BufferContents + Pod, I: BufferContents + Pod + Default> MainBuffers<V, I> {
  pub fn new(
    device: Arc<Device>,
    queue_families: &QueueFamilies,
    transfer_queue: Arc<Queue>,
    models: &Vec<Box<dyn Model<V>>>,
    max_instance_count: usize,
  ) -> Self {
    let (vertex, vertex_future) = create_immutable_vertex::<V>(transfer_queue.clone(), models);
    let (index, index_future) = create_immutable_index::<V>(transfer_queue, models);

    let fence = vertex_future
      .join(index_future)
      .then_signal_fence_and_flush()
      .unwrap();

    let model_lengths = models
      .iter()
      .map(|model| {
        (
          model.get_indices().len() as u32,
          model.get_vertices().len() as i32,
        )
      })
      .collect();

    let instance =
      create_device_instance(device.clone(), max_instance_count as u64, queue_families);

    fence.wait(None).unwrap();

    Self {
      vertex,
      index,
      instance,
      model_lengths,
    }
  }
}

pub struct Buffers<V: BufferContents + Pod, I: BufferContents + Pod> {
  main: MainBuffers<V, I>,

  // used to load data to instance
  instance_source: Vec<Arc<CpuAccessibleBuffer<[I]>>>,
}

impl<V: BufferContents + Pod, I: BufferContents + Pod + Default> Buffers<V, I> {
  pub fn initialize(
    device: Arc<Device>,
    queue_families: &QueueFamilies,
    transfer_queue: Arc<Queue>,
    buffer_count: usize,
    models: &Vec<Box<dyn Model<V>>>,
    max_instance_count: usize,
  ) -> Self {
    let instance_source =
      vec![create_cpu_accessible_instance_source(device.clone(), max_instance_count); buffer_count];

    Self {
      main: MainBuffers::new(
        device,
        queue_families,
        transfer_queue,
        models,
        max_instance_count,
      ),
      instance_source,
    }
  }

  pub fn update_matrices(&mut self, buffer_i: usize, data: Vec<I>) {
    let mut content = self.instance_source[buffer_i]
      .write()
      .unwrap_or_else(|e| panic!("Failed to write to instance buffer\n{}", e));

    content[0..data.len()].copy_from_slice(data.as_slice());
  }

  pub fn get_main(&self) -> &MainBuffers<V, I> {
    &self.main
  }

  pub fn get_instance_source(&self, buffer_i: usize) -> Arc<CpuAccessibleBuffer<[I]>> {
    self.instance_source[buffer_i].clone()
  }
}

fn create_immutable_vertex<V>(
  queue: Arc<Queue>,
  models: &Vec<Box<dyn Model<V>>>,
) -> (
  Arc<ImmutableBuffer<[V]>>,
  CommandBufferExecFuture<NowFuture, PrimaryAutoCommandBuffer>,
)
where
  V: BufferContents + Pod,
{
  let vertices: Vec<V> = models
    .iter()
    .map(|m| m.get_vertices().clone())
    .flatten()
    .collect();
  ImmutableBuffer::from_iter(vertices.into_iter(), BufferUsage::vertex_buffer(), queue).unwrap()
}

fn create_immutable_index<V>(
  queue: Arc<Queue>,
  models: &Vec<Box<dyn Model<V>>>,
) -> (
  Arc<ImmutableBuffer<[u16]>>,
  CommandBufferExecFuture<NowFuture, PrimaryAutoCommandBuffer>,
)
where
  V: BufferContents,
{
  let indices: Vec<u16> = models
    .iter()
    .map(|m| m.get_indices().clone())
    .flatten()
    .collect();
  ImmutableBuffer::from_iter(indices.into_iter(), BufferUsage::index_buffer(), queue).unwrap()
}

fn create_device_instance<'a, I>(
  device: Arc<Device>,
  max_total_instances: u64,
  queue_families: &QueueFamilies,
) -> Arc<DeviceLocalBuffer<[I]>>
where
  I: BufferContents + Pod + Default,
{
  DeviceLocalBuffer::array(
    device.clone(),
    max_total_instances,
    BufferUsage::vertex_buffer_transfer_destination(),
    [
      queue_families.compute,
      queue_families.transfers,
    ],
  )
  .unwrap()
}

fn create_cpu_accessible_instance_source<I>(
  device: Arc<Device>,
  max_total_instances: usize,
) -> Arc<CpuAccessibleBuffer<[I]>>
where
  I: BufferContents + Pod + Default,
{
  let data = vec![I::default(); max_total_instances];
  CpuAccessibleBuffer::from_iter(
    device.clone(),
    BufferUsage::transfer_source(),
    false,
    data.into_iter(),
  )
  .unwrap()
}
