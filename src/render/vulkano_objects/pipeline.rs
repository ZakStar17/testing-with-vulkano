use crate::render::vertex_data::{MatrixInstance, Vertex3d};
use std::sync::Arc;
use vulkano::{
  device::Device,
  pipeline::{
    graphics::{
      depth_stencil::DepthStencilState,
      input_assembly::InputAssemblyState,
      rasterization::{CullMode, RasterizationState},
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
    .vertex_input_state(
      BuffersDefinition::new()
        .vertex::<Vertex3d>()
        .instance::<MatrixInstance>(),
    )
    .vertex_shader(vs.entry_point("main").unwrap(), ())
    .input_assembly_state(InputAssemblyState::new())
    .viewport_state(ViewportState::viewport_fixed_scissor_irrelevant([viewport]))
    .rasterization_state(RasterizationState::new().cull_mode(CullMode::Back))
    .depth_stencil_state(DepthStencilState::simple_depth_test())
    .fragment_shader(fs.entry_point("main").unwrap(), ())
    .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
    .build(device.clone())
    // todo: reimplement dynamic uniforms
    // .with_auto_layout(device.clone(), |layout_create_infos| {
    //   let binding = layout_create_infos[0].bindings.get_mut(&0).unwrap();
    //   binding.descriptor_type = DescriptorType::UniformBufferDynamic;
    // })
    .unwrap()
}
