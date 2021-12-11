use crate::vulkano_graphics;
use crate::vulkano_graphics::shaders::Shaders;
use crate::vulkano_graphics::{Buffers, CommandBuffers, QueueFamilies, Queues, SwapchainData};
use std::time::Duration;
use winit::dpi::PhysicalSize;

use std::sync::Arc;

use cgmath::Matrix4;

use vulkano::device::physical::PhysicalDevice;
use vulkano::device::Device;
use vulkano::device::DeviceExtensions;
use vulkano::image::ImageAccess;
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::Pipeline;
use vulkano::swapchain::Surface;
use winit::window::Window;

// This can't have stored both a reference and a physical device
// because the physical device has internally a reference to the instance
// It also can't store only the physical device because it then needs to move to a thread and leaves the reference behind
// So the solution is to keep the instance in the main loop
pub struct VulkanoProgram {
  pub device: Arc<Device>,
  pub queues: Queues,
  pub swapchain_data: SwapchainData,
  pub buffers: Buffers,
  pub command_buffers: CommandBuffers,
  pub shaders: Shaders,
  pub viewport: Viewport,
  pub pipeline: Arc<GraphicsPipeline>,
  pub image_count: usize,
}

impl VulkanoProgram {
  pub fn start(
    device_extensions: DeviceExtensions,
    physical_device: PhysicalDevice,
    queue_families: &QueueFamilies,
    surface: Arc<Surface<Window>>,
  ) -> Self {
    let (device, queues) = vulkano_graphics::create_logical_device_and_queues(
      &physical_device,
      &device_extensions,
      &queue_families,
    );

    let surface_capabilities = surface.capabilities(physical_device).unwrap();
    let image_count = (surface_capabilities.min_image_count + 1) as usize;

    let swapchain_data = SwapchainData::init(
      physical_device,
      device.clone(),
      surface.clone(),
      &queues,
      image_count as u32,
    );

    let shaders = Shaders::load(device.clone());

    let dimensions = swapchain_data.images[0].dimensions();
    let viewport = Viewport {
      origin: [0.0, 0.0],
      dimensions: [dimensions.width() as f32, dimensions.height() as f32],
      depth_range: 0.0..1.0,
    };

    let pipeline = vulkano_graphics::create_pipeline(
      device.clone(),
      &shaders,
      swapchain_data.render_pass.clone(),
      &viewport,
    );

    let buffers = Buffers::init(
      device.clone(),
      &queues,
      image_count,
      pipeline.layout().descriptor_set_layouts().get(0).unwrap(),
    );

    let command_buffers = CommandBuffers::init(
      device.clone(),
      &queues,
      &buffers,
      &swapchain_data.framebuffers,
      &viewport,
      pipeline.clone(),
    );

    Self {
      device,
      queues,
      swapchain_data,
      buffers,
      command_buffers,
      shaders,
      viewport,
      pipeline,
      image_count,
    }
  }

  pub fn handle_window_update(&mut self, window_dimensions: PhysicalSize<u32>) {
    self.swapchain_data.recreate(&window_dimensions.into());

    self.viewport.dimensions = window_dimensions.into();

    // recreate pipeline
    self.pipeline = vulkano_graphics::create_pipeline(
      self.device.clone(),
      &self.shaders,
      self.swapchain_data.render_pass.clone(),
      &self.viewport,
    );

    // recreate command buffers that depend on the objects recreated
    self.command_buffers.update_window_dependent(
      self.device.clone(),
      &self.queues,
      &self.buffers,
      &self.swapchain_data.framebuffers,
      &self.viewport,
      self.pipeline.clone(),
    );
  }

  pub fn update(&self, _elapsed_time: &Duration, n_image: usize, matrix: Matrix4<f32>) {
    let mut buffer_content = self.buffers.uniforms[n_image]
      .0
      .write()
      .unwrap_or_else(|e| panic!("Failed to write to uniform buffer\n{}", e));

    buffer_content.matrix = matrix.into();
  }
}
