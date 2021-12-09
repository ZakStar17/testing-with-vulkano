use crate::vulkano_graphics::shaders;

use shaders::vertex_data::Vertex3d;
use shaders::Shaders;

use std::sync::Arc;
use vulkano::device::Device;
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::render_pass::{RenderPass, Subpass};

pub fn create(
  device: Arc<Device>,
  shaders: &Shaders,
  render_pass: Arc<RenderPass>,
  viewport: &Viewport,
) -> Arc<GraphicsPipeline> {
  Arc::new(
    GraphicsPipeline::start()
      .vertex_input_single_buffer::<Vertex3d>()
      .vertex_shader(shaders.vertex.main_entry_point(), ())
      .triangle_list()
      .viewports_dynamic_scissors_irrelevant(1)
      .viewports([viewport.clone()])
      .fragment_shader(shaders.fragment.main_entry_point(), ())
      // .depth_stencil_simple_depth()
      .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
      .build(device.clone())
      .unwrap(),
  )
}
