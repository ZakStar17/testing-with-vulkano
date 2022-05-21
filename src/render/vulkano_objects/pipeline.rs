use crate::render::Vertex3d;
use std::sync::Arc;
use vulkano::{
  descriptor_set::layout::DescriptorType,
  device::Device,
  pipeline::{
    graphics::{
      input_assembly::InputAssemblyState,
      vertex_input::BuffersDefinition,
      viewport::{Viewport, ViewportState},
    },
    GraphicsPipeline,
  },
  render_pass::{RenderPass, Subpass},
  shader::ShaderModule,
};

pub fn create(
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
