use crate::render::shaders::UniformShader;
use bytemuck::Pod;
use std::sync::Arc;
use vulkano::buffer::{
  BufferContents, BufferUsage, CpuAccessibleBuffer, ImmutableBuffer, TypedBufferAccess,
};
use vulkano::command_buffer::{CommandBufferExecFuture, PrimaryAutoCommandBuffer};
use vulkano::descriptor_set::layout::DescriptorSetLayout;
use vulkano::descriptor_set::{
  DescriptorSetsCollection, PersistentDescriptorSet, WriteDescriptorSet,
};
use vulkano::device::{Device, Queue};
use vulkano::pipeline::graphics::vertex_input::VertexBuffersCollection;
use vulkano::sync::{GpuFuture, NowFuture};

use crate::render::models::Model;

pub type Uniform<U> = (Arc<CpuAccessibleBuffer<U>>, Arc<PersistentDescriptorSet>);

// This trait will apply to all structs that contain vertex, index and uniform buffers
pub trait Buffers<Vb, Ib, D>
where
  Vb: VertexBuffersCollection,                      // Vertex buffer
  Ib: TypedBufferAccess<Content = [u16]> + 'static, // Index buffer
  D: DescriptorSetsCollection,
{
  fn get_vertex(&self) -> Vb;

  // Vb and D have their own collection, so they are implicitly wrapped in an Arc, but Ib should be wrapped explicitly
  fn get_index(&self) -> Arc<Ib>;
  fn get_model_lengths(&self) -> &Vec<(u32, i32)>;
  fn get_uniform_descriptor_set(&self, i: usize) -> D;
}

// Struct with immutable vertex and index buffer and a cpu accessible uniform buffer, with generic (V)ertices and (U)niforms
pub struct ImmutableBuffers<V: BufferContents + Pod, U: BufferContents> {
  pub vertex: Arc<ImmutableBuffer<[V]>>,
  pub index: Arc<ImmutableBuffer<[u16]>>,
  pub model_lengths: Vec<(u32, i32)>,
  pub uniforms: Vec<Uniform<U>>,
}

impl<V: BufferContents + Pod, U: BufferContents + Copy> ImmutableBuffers<V, U> {
  pub fn initialize<S: UniformShader<U>>(
    device: Arc<Device>,
    descriptor_set_layout: Arc<DescriptorSetLayout>,
    uniform_buffer_count: usize,
    transfer_queue: Arc<Queue>,
    models: &Vec<Box<dyn Model<V>>>,
  ) -> Self {
    let (vertex, vertex_future) = create_immutable_vertex::<V>(transfer_queue.clone(), models);
    let (index, index_future) = create_immutable_index::<V>(transfer_queue, models);

    let model_lengths = models
      .iter()
      .map(|model| {
        (
          model.get_indices().len() as u32,
          model.get_vertices().len() as i32,
        )
      })
      .collect();

    let fence = vertex_future
      .join(index_future)
      .then_signal_fence_and_flush()
      .unwrap();

    fence.wait(None).unwrap();

    Self {
      vertex,
      index,
      model_lengths,
      uniforms: create_cpu_accessible_uniforms::<U, S>(
        device,
        descriptor_set_layout,
        uniform_buffer_count,
      ),
    }
  }
}

impl<'a, V, U>
  Buffers<Arc<ImmutableBuffer<[V]>>, ImmutableBuffer<[u16]>, Arc<PersistentDescriptorSet>>
  for ImmutableBuffers<V, U>
where
  V: BufferContents + Pod,
  U: BufferContents,
{
  fn get_vertex(&self) -> Arc<ImmutableBuffer<[V]>> {
    self.vertex.clone()
  }

  fn get_index(&self) -> Arc<ImmutableBuffer<[u16]>> {
    self.index.clone()
  }

  fn get_model_lengths(&self) -> &Vec<(u32, i32)> {
    &self.model_lengths
  }

  fn get_uniform_descriptor_set(&self, i: usize) -> Arc<PersistentDescriptorSet> {
    self.uniforms[i].1.clone()
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
  let vertices: Vec<V> = models.iter().map(|m| m.get_vertices().clone()).flatten().collect();
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
  let indices: Vec<u16> = models.iter().map(|m| m.get_indices().clone()).flatten().collect();
  ImmutableBuffer::from_iter(indices.into_iter(), BufferUsage::index_buffer(), queue).unwrap()
}

fn create_cpu_accessible_uniforms<U, S>(
  device: Arc<Device>,
  descriptor_set_layout: Arc<DescriptorSetLayout>,
  buffer_count: usize,
) -> Vec<Uniform<U>>
where
  U: BufferContents + Copy,
  S: UniformShader<U>,
{
  (0..buffer_count)
    .map(|_| {
      let buffer = CpuAccessibleBuffer::from_data(
        device.clone(),
        BufferUsage::uniform_buffer(),
        false,
        S::get_initial_uniform_data(),
      )
      .unwrap();

      let descriptor_set = PersistentDescriptorSet::new(
        descriptor_set_layout.clone(),
        [WriteDescriptorSet::buffer(0, buffer.clone())],
      )
      .unwrap();

      (buffer, descriptor_set)
    })
    .collect()
}
