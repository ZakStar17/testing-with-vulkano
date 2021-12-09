use crate::vulkano_graphics::{Buffers, Queues};

use std::sync::Arc;
use vulkano::buffer::TypedBufferAccess;
use vulkano::command_buffer::PrimaryAutoCommandBuffer;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, SubpassContents};
use vulkano::device::Device;
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::PipelineBindPoint;
use vulkano::render_pass::FramebufferAbstract;
use vulkano::sync;
use vulkano::sync::GpuFuture;

pub struct CommandBuffers {
  pub main: Vec<Arc<PrimaryAutoCommandBuffer>>,
}

impl CommandBuffers {
  pub fn init(
    device: Arc<Device>,
    queues: &Queues,
    buffers: &Buffers,
    framebuffers: &Vec<Arc<dyn FramebufferAbstract>>,
    viewport: &Viewport,
    pipeline: Arc<GraphicsPipeline>,
  ) -> Self {
    Self {
      main: Self::create_main(device, queues, buffers, framebuffers, viewport, pipeline),
    }
  }

  fn create_main(
    device: Arc<Device>,
    queues: &Queues,
    buffers: &Buffers,
    framebuffers: &Vec<Arc<dyn FramebufferAbstract>>,
    viewport: &Viewport,
    pipeline: Arc<GraphicsPipeline>,
  ) -> Vec<Arc<PrimaryAutoCommandBuffer>> {
    let clear_values = vec![[0.0, 0.0, 1.0, 1.0].into()];
    framebuffers
      .iter()
      .enumerate()
      .map(|(i, framebuffer)| {
        let mut builder = AutoCommandBufferBuilder::primary(
          device.clone(),
          queues.graphical.family(),
          CommandBufferUsage::SimultaneousUse,
        )
        .unwrap();
        builder
          .begin_render_pass(
            framebuffer.clone(),
            SubpassContents::Inline,
            clear_values.clone(),
          )
          .unwrap()
          .set_viewport(0, [viewport.clone()])
          .bind_pipeline_graphics(pipeline.clone())
          .bind_descriptor_sets(
            PipelineBindPoint::Graphics,
            pipeline.layout().clone(),
            0,
            buffers.uniforms[i].1.clone(),
          )
          .bind_vertex_buffers(0, buffers.models.cube.vertex_staging.clone())
          .bind_index_buffer(buffers.models.cube.index_staging.clone())
          .draw_indexed(buffers.models.cube.index_staging.len() as u32, 1, 0, 0, 0)
          .unwrap()
          .end_render_pass()
          .unwrap();
        builder.build().unwrap()
      })
      .map(|c_b| Arc::new(c_b))
      .collect()
  }

  pub fn update_window_dependent(
    &mut self,
    device: Arc<Device>,
    queues: &Queues,
    buffers: &Buffers,
    framebuffers: &Vec<Arc<dyn FramebufferAbstract>>,
    viewport: &Viewport,
    pipeline: Arc<GraphicsPipeline>,
  ) {
    self.main = Self::create_main(device, queues, buffers, framebuffers, viewport, pipeline)
  }
}

// Executes a command buffer copying the vertex buffer into the staging buffer,
// then waits for the gpu to finish
// This is a costly opperation, and it should not be done between frames
pub fn transfer_buffer_contents_on_gpu<T, O, D>(
  device: Arc<Device>,
  queues: &Queues,
  origin_buffer: &Arc<O>,
  destination_buffer: &Arc<D>,
) where
  O: TypedBufferAccess<Content = [T]> + 'static,
  D: TypedBufferAccess<Content = [T]> + 'static,
{
  let trasnfer_command_buffer = {
    let mut builder = AutoCommandBufferBuilder::primary(
      device.clone(),
      queues.transfers.family(),
      CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();

    builder
      .copy_buffer(origin_buffer.clone(), destination_buffer.clone())
      .unwrap();

    Arc::new(builder.build().unwrap())
  };

  let future = sync::now(device)
    .then_execute(queues.transfers.clone(), trasnfer_command_buffer.clone())
    .unwrap()
    .then_signal_fence_and_flush()
    .unwrap();

  future.wait(None).unwrap();
}

// Executes a command buffer copying the vertex buffer into the staging buffer,
// then waits for the gpu to finish
// This is a costly opperation, and it should not be done between frames
// todo: change this to use buffers pairs of buffers of different contents,
// or else it's almost useless
pub fn _transfer_multiple_buffer_contents_on_gpu<B>(
  device: &Arc<Device>,
  queues: &Queues,
  buffers: Vec<(&Arc<B>, &Arc<B>)>,
) where
  B: TypedBufferAccess + 'static,
{
  let trasnfer_command_buffer = {
    let mut builder = AutoCommandBufferBuilder::primary(
      device.clone(),
      queues.transfers.family(),
      CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();

    for (origin_buffer, destination_buffer) in buffers {
      builder
        .copy_buffer(origin_buffer.clone(), destination_buffer.clone())
        .unwrap();
    }

    Arc::new(builder.build().unwrap())
  };

  let future = sync::now(device.clone())
    .then_execute(queues.transfers.clone(), trasnfer_command_buffer.clone())
    .unwrap()
    .then_signal_fence_and_flush()
    .unwrap();

  future.wait(None).unwrap();
}
