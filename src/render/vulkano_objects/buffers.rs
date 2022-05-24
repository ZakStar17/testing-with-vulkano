use crate::render::models::Model;
use bytemuck::Pod;
use std::sync::Arc;
use vulkano::{
  buffer::{BufferContents, BufferUsage, CpuAccessibleBuffer, ImmutableBuffer},
  command_buffer::{CommandBufferExecFuture, PrimaryAutoCommandBuffer},
  device::{Device, Queue},
  sync::{GpuFuture, NowFuture},
};

pub struct Buffers<V: BufferContents + Pod, I: BufferContents + Pod> {
  vertex: Arc<ImmutableBuffer<[V]>>,
  index: Arc<ImmutableBuffer<[u16]>>,
  instance: Vec<Arc<CpuAccessibleBuffer<[I]>>>,
  model_lengths: Vec<(u32, i32)>,
}

impl<V: BufferContents + Pod, I: BufferContents + Pod + Default> Buffers<V, I> {
  pub fn initialize(
    device: Arc<Device>,
    uniform_buffer_count: usize,
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

    let instance = vec![
      create_cpu_accessible_instance(device.clone(), max_instance_count);
      uniform_buffer_count
    ];

    fence.wait(None).unwrap();

    Self {
      vertex,
      index,
      instance,
      model_lengths,
    }
  }

  pub fn update_matrices(&mut self, buffer_i: usize, data: Vec<I>) {
    let mut instance_content = self.instance[buffer_i]
      .write()
      .unwrap_or_else(|e| panic!("Failed to write to instance buffer\n{}", e));

    instance_content[0..data.len()].copy_from_slice(data.as_slice());
  }

  pub fn get_vertex(&self) -> Arc<ImmutableBuffer<[V]>> {
    self.vertex.clone()
  }

  pub fn get_index(&self) -> Arc<ImmutableBuffer<[u16]>> {
    self.index.clone()
  }

  pub fn get_instance(&self, buffer_i: usize) -> Arc<CpuAccessibleBuffer<[I]>> {
    self.instance[buffer_i].clone()
  }

  pub fn get_model_lengths(&self) -> &Vec<(u32, i32)> {
    &self.model_lengths
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

fn create_cpu_accessible_instance<I>(
  device: Arc<Device>,
  max_total_instances: usize,
) -> Arc<CpuAccessibleBuffer<[I]>>
where
  I: BufferContents + Pod + Default,
{
  let data = vec![I::default(); max_total_instances];
  CpuAccessibleBuffer::from_iter(
    device.clone(),
    BufferUsage::vertex_buffer(),
    false,
    data.into_iter(),
  )
  .unwrap()
}
