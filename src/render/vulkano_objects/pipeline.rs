use crate::render::Vertex3d;
use std::sync::Arc;
use vulkano::descriptor_set::layout::DescriptorType;
use vulkano::device::Device;
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::vertex_input::BuffersDefinition;
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::GraphicsPipeline;
use vulkano::render_pass::{RenderPass, Subpass};
use vulkano::shader::ShaderModule;

pub fn create_pipeline(
  device: Arc<Device>,
  vs: Arc<ShaderModule>,
  fs: Arc<ShaderModule>,
  render_pass: Arc<RenderPass>,
  viewport: Viewport,
) -> Arc<GraphicsPipeline> {
  GraphicsPipeline::start()
    .vertex_input_state(BuffersDefinition::new().vertex::<Vertex3d>())
    .vertex_shader(vs.entry_point("main").unwrap(), ())
    .input_assembly_state(InputAssemblyState::new())
    .viewport_state(ViewportState::viewport_fixed_scissor_irrelevant([viewport]))
    .fragment_shader(fs.entry_point("main").unwrap(), ())
    .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
    .with_auto_layout(device.clone(), |layout_create_infos| {
      let binding = layout_create_infos[0].bindings.get_mut(&0).unwrap();
      binding.descriptor_type = DescriptorType::UniformBufferDynamic;
    })
    .unwrap()
}
