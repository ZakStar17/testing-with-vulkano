use crate::render::vulkano_objects::buffers::Buffers;
use bytemuck::Pod;
use std::sync::Arc;
use vulkano::{
  buffer::{BufferContents, TypedBufferAccess},
  DeviceSize,
};

use vulkano::{
  command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, SubpassContents,
  },
  device::{Device, Queue},
  format::ClearValue,
  pipeline::GraphicsPipeline,
  render_pass::Framebuffer,
};

pub fn create_main<V: BufferContents + Pod, I: BufferContents + Pod + Default>(
  device: Arc<Device>,
  graphics_queue: Arc<Queue>,
  pipeline: Arc<GraphicsPipeline>,
  framebuffers: &Vec<Arc<Framebuffer>>,
  buffers: &Buffers<V, I>,
  instance_count_per_model: &Vec<u32>,
) -> Vec<Arc<PrimaryAutoCommandBuffer>> {
  framebuffers
    .iter()
    .enumerate()
    .map(|(_, framebuffer)| {
      let main_buffers = buffers.get_main();

      let mut builder = AutoCommandBufferBuilder::primary(
        device.clone(),
        graphics_queue.family(),
        CommandBufferUsage::MultipleSubmit,
      )
      .unwrap();

      builder
        .begin_render_pass(
          framebuffer.clone(),
          SubpassContents::Inline,
          vec![[0.1, 0.1, 0.1, 1.0].into(), ClearValue::Depth(1.0)],
        )
        .unwrap()
        .bind_pipeline_graphics(pipeline.clone());

      // bind index and vertex buffers
      builder
        .bind_vertex_buffers(
          0,
          (main_buffers.vertex.clone(), main_buffers.instance.clone()),
        )
        .bind_index_buffer(main_buffers.index.clone());

      // draw with offsets
      let mut index_offset = 0;
      let mut vertex_offset = 0;
      let mut instance_offset = 0;
      for (&(index_len, vertex_len), &instance_count) in main_buffers
        .model_lengths
        .iter()
        .zip(instance_count_per_model.iter())
      {
        builder
          .draw_indexed(
            index_len,
            instance_count,
            index_offset,
            vertex_offset,
            instance_offset,
          )
          .unwrap();

        index_offset += index_len;
        vertex_offset += vertex_len;
        instance_offset += instance_count;
      }

      builder.end_render_pass().unwrap();

      Arc::new(builder.build().unwrap())
    })
    .collect()
}

#[allow(dead_code)]
pub fn create_copy<T, S, D>(
  device: Arc<Device>,
  transfers_queue: Arc<Queue>,
  source: Arc<S>,
  destination: Arc<D>,
) -> Arc<PrimaryAutoCommandBuffer>
where
  S: TypedBufferAccess<Content = T> + 'static,
  D: TypedBufferAccess<Content = T> + 'static,
{
  let mut builder = AutoCommandBufferBuilder::primary(
    device.clone(),
    transfers_queue.family(),
    CommandBufferUsage::MultipleSubmit,
  )
  .unwrap();

  builder.copy_buffer(source, destination).unwrap();

  Arc::new(builder.build().unwrap())
}

pub fn create_slice_copy<T, S, D>(
  device: Arc<Device>,
  transfers_queue: Arc<Queue>,
  source: Arc<S>,
  source_offset: DeviceSize,
  destination: Arc<D>,
  destination_offset: DeviceSize,
  count: DeviceSize,
) -> Arc<PrimaryAutoCommandBuffer>
where
  S: TypedBufferAccess<Content = [T]> + 'static,
  D: TypedBufferAccess<Content = [T]> + 'static,
{
  let mut builder = AutoCommandBufferBuilder::primary(
    device.clone(),
    transfers_queue.family(),
    CommandBufferUsage::MultipleSubmit,
  )
  .unwrap();

  builder
    .copy_buffer_dimensions(
      source,
      source_offset,
      destination,
      destination_offset,
      count,
    )
    .unwrap();

  Arc::new(builder.build().unwrap())
}
