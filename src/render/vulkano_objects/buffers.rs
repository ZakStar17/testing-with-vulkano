use serde::Serialize;
use crate::render::shaders::UniformShader;
use bytemuck::Pod;
use std::mem::size_of;
use std::sync::Arc;
use vulkano::buffer::{
  BufferContents, BufferUsage, CpuAccessibleBuffer, ImmutableBuffer, TypedBufferAccess,
};
use vulkano::command_buffer::{CommandBufferExecFuture, PrimaryAutoCommandBuffer};
use vulkano::descriptor_set::layout::DescriptorSetLayout;
use vulkano::descriptor_set::DescriptorSet;
use vulkano::descriptor_set::DescriptorSetWithOffsets;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::device::{Device, Queue};
use vulkano::pipeline::graphics::vertex_input::VertexBuffersCollection;
use vulkano::sync::{GpuFuture, NowFuture};

use crate::render::models::Model;

// This trait will apply to all structs that contain vertex, index and uniform buffers
pub trait Buffers<Vb, Ib>
where
  Vb: VertexBuffersCollection, // Vertex buffer
  Ib: TypedBufferAccess<Content = [u16]> + 'static,
{
  fn get_vertex(&self) -> Vb;

  // Vb and D have their own collection, so they are implicitly wrapped in an Arc, but Ib should be wrapped explicitly
  fn get_index(&self) -> Arc<Ib>;
  fn get_model_lengths(&self) -> &Vec<(u32, i32)>;
  fn get_uniform_descriptor_set_offsets(
    &self,
    buffer_i: usize,
    model_i: usize,
  ) -> DescriptorSetWithOffsets;
}

// Struct with immutable vertex and index buffer and a cpu accessible uniform buffer, with generic (V)ertices and (U)niforms
pub struct ImmutableBuffers<V: BufferContents + Pod> {
  vertex: Arc<ImmutableBuffer<[V]>>,
  index: Arc<ImmutableBuffer<[u16]>>,
  model_lengths: Vec<(u32, i32)>,
  uniforms: Vec<(Arc<CpuAccessibleBuffer<[u8]>>, Arc<PersistentDescriptorSet>)>,
  uniform_align: usize,
}

impl<V: BufferContents + Pod> ImmutableBuffers<V> {
  pub fn initialize<U: BufferContents + Copy, S: UniformShader<U>>(
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

    let (uniform_buffers, uniform_align) = create_cpu_accessible_uniforms::<U, S>(
      device,
      descriptor_set_layout,
      uniform_buffer_count,
      models.len(),
    );

    fence.wait(None).unwrap();

    Self {
      vertex,
      index,
      model_lengths,
      uniforms: uniform_buffers,
      uniform_align,
    }
  }

  pub fn write_to_uniform<U: BufferContents + Copy + Serialize>(&mut self, buffer_i: usize, data: Vec<(usize, U)>) {
    let mut data_bytes = Vec::with_capacity(data.len());
    for (model_i, uniform_struct) in data {
      data_bytes.push((model_i, bincode::serialize(&uniform_struct).unwrap()))
    }

    let mut uniform_content = self.uniforms[buffer_i]
      .0
      .write()
      .unwrap_or_else(|e| panic!("Failed to write to uniform buffer\n{}", e));

    for (model_i, bytes) in data_bytes {
      let mut offset = model_i * self.uniform_align;

      for byte in bytes {
        uniform_content[offset] = byte;
        offset += 1;
      }
    }
  }
}

impl<'a, V> Buffers<Arc<ImmutableBuffer<[V]>>, ImmutableBuffer<[u16]>> for ImmutableBuffers<V>
where
  V: BufferContents + Pod,
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

  fn get_uniform_descriptor_set_offsets(
    &self,
    buffer_i: usize,
    model_i: usize,
  ) -> DescriptorSetWithOffsets {
    self.uniforms[buffer_i]
      .1
      .clone()
      .offsets([(model_i * self.uniform_align) as u32])
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

fn create_cpu_accessible_uniforms<U, S>(
  device: Arc<Device>,
  descriptor_set_layout: Arc<DescriptorSetLayout>,
  buffer_count: usize,
  model_count: usize,
) -> (
  Vec<(Arc<CpuAccessibleBuffer<[u8]>>, Arc<PersistentDescriptorSet>)>,
  usize,
)
where
  U: BufferContents + Copy,
  S: UniformShader<U>,
{
  let mut uniform_bytes = S::get_initial_uniform_bytes();

  // dynamic uniform buffers will be aligned by a specific amount
  let min_dynamic_align = device
    .physical_device()
    .properties()
    .min_uniform_buffer_offset_alignment as usize;
  let align = (size_of::<U>() + min_dynamic_align - 1) & !(min_dynamic_align - 1);

  // set uniform_bytes to have the same size as align
  uniform_bytes.append(&mut vec![0; align - uniform_bytes.len()]);
  assert_eq!(uniform_bytes.len(), align);
  let aligned_data = uniform_bytes.repeat(model_count);

  let buffers: Vec<(Arc<CpuAccessibleBuffer<[u8]>>, Arc<PersistentDescriptorSet>)> = (0
    ..buffer_count)
    .map(|_| {
      let buffer = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::uniform_buffer(),
        false,
        aligned_data.clone(),
      )
      .unwrap();

      let descriptor_set = PersistentDescriptorSet::new(
        descriptor_set_layout.clone(),
        [WriteDescriptorSet::buffer(0, buffer.clone())],
      )
      .unwrap();

      (buffer, descriptor_set)
    })
    .collect();

  (buffers, align)
}
