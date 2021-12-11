use crate::vulkano_graphics::shaders;
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::vertex_input::BuffersDefinition;
use vulkano::pipeline::graphics::viewport::ViewportState;

use shaders::vertex_data::Vertex3d;
use shaders::Shaders;

use std::sync::Arc;
use vulkano::device::Device;
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::render_pass::{RenderPass, Subpass};

pub fn create(
  device: Arc<Device>,
  shaders: &Shaders,
  render_pass: Arc<RenderPass>,
  viewport: &Viewport,
) -> Arc<GraphicsPipeline> {
  GraphicsPipeline::start()
    .vertex_input_state(BuffersDefinition::new().vertex::<Vertex3d>())
    .vertex_shader(shaders.vertex.entry_point("main").unwrap(), ())
    .input_assembly_state(InputAssemblyState::new())
    .viewport_state(ViewportState::viewport_fixed_scissor_irrelevant([
      viewport.clone()
    ]))
    .fragment_shader(shaders.fragment.entry_point("main").unwrap(), ())
    .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
    .build(device.clone())
    .unwrap()
}
