use crate::render::vulkano_objects::buffers::Buffers;
use bytemuck::Pod;
use std::sync::Arc;
use vulkano::buffer::BufferContents;

use vulkano::{
  command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, SubpassContents,
  },
  device::{Device, Queue},
  format::ClearValue,
  pipeline::GraphicsPipeline,
  render_pass::Framebuffer,
};

pub fn create<V: BufferContents + Pod, I: BufferContents + Pod + Default>(
  device: Arc<Device>,
  queue: Arc<Queue>,
  pipeline: Arc<GraphicsPipeline>,
  framebuffers: &Vec<Arc<Framebuffer>>,
  buffers: &Buffers<V, I>,
  instance_count_per_model: &Vec<u32>,
) -> Vec<Arc<PrimaryAutoCommandBuffer>> {
  framebuffers
    .iter()
    .enumerate()
    .map(|(buffer_i, framebuffer)| {
      let mut builder = AutoCommandBufferBuilder::primary(
        device.clone(),
        queue.family(),
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
      let index_buffer = buffers.get_index();
      builder
        .bind_vertex_buffers(0, (buffers.get_vertex(), buffers.get_instance(buffer_i)))
        .bind_index_buffer(index_buffer);

      // draw with offsets
      let mut index_offset = 0;
      let mut vertex_offset = 0;
      let mut instance_offset = 0;
      for (&(index_len, vertex_len), &instance_count) in buffers
        .get_model_lengths()
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
