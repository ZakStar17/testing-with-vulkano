use std::sync::Arc;

use vulkano::buffer::TypedBufferAccess;
use vulkano::command_buffer::{
  AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, SubpassContents,
};
use vulkano::device::{Device, Queue};
use vulkano::pipeline::graphics::vertex_input::VertexBuffersCollection;
use vulkano::pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint};
use vulkano::render_pass::Framebuffer;

use crate::render::vulkano_objects::buffers::Buffers;

pub fn create_simple_command_buffers<
  Vb: VertexBuffersCollection,
  Ib: TypedBufferAccess<Content = [u16]> + 'static
>(
  device: Arc<Device>,
  queue: Arc<Queue>,
  pipeline: Arc<GraphicsPipeline>,
  framebuffers: &Vec<Arc<Framebuffer>>,
  buffers: &dyn Buffers<Vb, Ib>,
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
          vec![[0.1, 0.1, 0.1, 1.0].into()],
        )
        .unwrap()
        .bind_pipeline_graphics(pipeline.clone());

      // bind index and vertex buffers
      let index_buffer = buffers.get_index();
      builder
        .bind_vertex_buffers(0, buffers.get_vertex())
        .bind_index_buffer(index_buffer);

      // draw with offsets
      let mut index_offset = 0;
      let mut vertex_offset = 0;
      for (model_i, (index_len, vertex_len)) in buffers.get_model_lengths().iter().enumerate() {
        builder
          .bind_descriptor_sets(
            PipelineBindPoint::Graphics,
            pipeline.layout().clone(),
            0,
            buffers.get_uniform_descriptor_set_offsets(buffer_i, model_i),
          )
          .draw_indexed(*index_len, 1, index_offset, vertex_offset, 0)
          .unwrap();

        index_offset += *index_len;
        vertex_offset += *vertex_len;
      }

      builder.end_render_pass().unwrap();

      Arc::new(builder.build().unwrap())
    })
    .collect()
}
