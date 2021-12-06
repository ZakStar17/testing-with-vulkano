use crate::vulkano_graphics::initialization as init;
use crate::vulkano_graphics::shaders;
use std::time::Duration;
use vulkano::descriptor_set::PersistentDescriptorSet;

use shaders::triangle::{fs, vs};
use shaders::utils::Vertex2d;

use std::sync::Arc;
use vulkano::buffer::{CpuAccessibleBuffer, DeviceLocalBuffer};
use vulkano::command_buffer::PrimaryAutoCommandBuffer;
use vulkano::device::physical::PhysicalDevice;
use vulkano::device::physical::QueueFamily;
use vulkano::device::DeviceExtensions;
use vulkano::device::{Device, Queue};
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::render_pass::{FramebufferAbstract, RenderPass};
use vulkano::swapchain::Surface;
use vulkano::swapchain::Swapchain;
use vulkano::swapchain::SwapchainCreationError;
use winit::window::Window;

pub struct QueueFamilies<'a> {
  pub graphical: QueueFamily<'a>,
  pub compute: QueueFamily<'a>,
  pub transfers: QueueFamily<'a>,
}

pub struct Queues {
  pub graphical: Arc<Queue>,
  pub compute: Arc<Queue>,
  pub transfers: Arc<Queue>,
}

pub struct Buffers {
  pub vertex: Arc<CpuAccessibleBuffer<[Vertex2d]>>,
  pub staging: Arc<DeviceLocalBuffer<[Vertex2d]>>,
  pub uniform: Vec<Arc<CpuAccessibleBuffer<vs::ty::Data>>>,
}

pub struct DescriptorSets {
  pub uniform: Vec<Arc<PersistentDescriptorSet>>,
}

pub struct CommandBuffers {
  pub stage_vertices: Arc<PrimaryAutoCommandBuffer>,
  pub main: Vec<Arc<PrimaryAutoCommandBuffer>>,
}

pub struct Shaders {
  pub vertex: vs::Shader,
  pub fragment: fs::Shader,
}

// This can't have stored both a reference and a physical device
// because the physical device has internally a reference to the instance
// It also can't store only the physical device because it then needs to move to a thread and leaves the reference behind
// So the solution is to keep the instance in the main loop
pub struct VulkanoProgram {
  pub device: Arc<Device>,
  pub queues: Queues,
  pub swapchain: Arc<Swapchain<Window>>,
  pub buffers: Buffers,
  pub command_buffers: CommandBuffers,
  pub descriptor_sets: DescriptorSets,
  pub shaders: Shaders,
  pub render_pass: Arc<RenderPass>,
  pub viewport: Viewport,
  pub framebuffers: Vec<Arc<dyn FramebufferAbstract>>,
  pub pipeline: Arc<GraphicsPipeline>,
  pub image_count: usize,
}

impl VulkanoProgram {
  pub fn start(
    device_extensions: DeviceExtensions,
    physical_device: PhysicalDevice,
    queue_families: &QueueFamilies,
    surface: &Arc<Surface<Window>>,
  ) -> Self {
    let (device, queues) =
      init::create_logical_device_and_queues(&physical_device, &device_extensions, &queue_families);

    let surface_capabilities = surface.capabilities(physical_device).unwrap();
    let image_count = (surface_capabilities.min_image_count + 1) as usize;

    let (swapchain, images) =
      init::create_swapchain(&physical_device, &device, surface, &queues, image_count as u32);

    let shaders = init::load_shaders(&device);

    let render_pass = init::create_render_pass(&device, &swapchain);

    let dimensions = images[0].dimensions();
    let viewport = Viewport {
      origin: [0.0, 0.0],
      dimensions: [dimensions[0] as f32, dimensions[1] as f32],
      depth_range: 0.0..1.0,
    };

    let framebuffers = init::create_framebuffers(&images, render_pass.clone());
    let pipeline = init::create_pipeline(&device, &shaders, &render_pass, &images[0].dimensions());

    let buffers = init::create_and_initalize_buffers(&device, &queues, image_count);

    let descriptor_sets = init::create_descriptor_sets(
      &buffers,
      pipeline.layout().descriptor_set_layouts().get(0).unwrap(),
    );

    let command_buffers = init::create_command_buffers(
      &device,
      &queues,
      &buffers,
      &framebuffers,
      &viewport,
      &pipeline,
      &descriptor_sets,
    );

    // populate staging vertex buffer for the first time
    init::update_staging_buffer(&device, &queues, &command_buffers);

    Self {
      device,
      queues,
      swapchain,
      buffers,
      command_buffers,
      shaders,
      render_pass,
      viewport,
      framebuffers,
      pipeline,
      descriptor_sets,
      image_count,
    }
  }

  pub fn update_with_window(&mut self, surface: &Arc<Surface<Window>>) {
    let dimensions: [u32; 2] = surface.window().inner_size().into();

    let (new_swapchain, new_images) = match self.swapchain.recreate().dimensions(dimensions).build()
    {
      Ok(r) => r,
      Err(SwapchainCreationError::UnsupportedDimensions) => return,
      Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
    };

    // recreate swapchain
    self.swapchain = new_swapchain;
    self.framebuffers = init::create_framebuffers(&new_images, self.render_pass.clone());
    let dimensions = new_images[0].dimensions();
    self.viewport.dimensions = [dimensions[0] as f32, dimensions[1] as f32];

    // recreate pipeline
    self.pipeline = init::create_pipeline(
      &self.device,
      &self.shaders,
      &self.render_pass,
      &new_images[0].dimensions(),
    );

    // recreate command buffers that depend on the objects recreated
    self.command_buffers.main = init::create_main_command_buffers(
      &self.device,
      &self.queues,
      &self.buffers,
      &self.framebuffers,
      &self.viewport,
      &self.pipeline,
      &self.descriptor_sets,
    );
  }

  pub fn update(&self, elapsed_time: &Duration, n_image: usize, colors: &[[f32; 3]; 4]) {
    let new_color = colors[(elapsed_time).as_secs() as usize % colors.len()];

    let mut buffer_content = self.buffers.uniform[n_image]
      .write()
      .unwrap_or_else(|e| panic!("Failed to write to uniform buffer\n{}", e));

    buffer_content.color = new_color;
  }
}
