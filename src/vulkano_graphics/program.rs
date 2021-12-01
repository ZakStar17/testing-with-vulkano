use crate::vulkano_graphics::initialization as init;
use crate::vulkano_graphics::shaders;

use shaders::triangle::{fs, vs};
use shaders::utils::Vertex2d;

use std::sync::Arc;
use vulkano::buffer::{CpuAccessibleBuffer, CpuBufferPool, DeviceLocalBuffer};
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
use winit::window::{Window};

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
  pub uniform: CpuBufferPool<vs::ty::Data>,
}

pub struct CommandBuffers {
  pub stage_vertices: Arc<PrimaryAutoCommandBuffer>,
}

pub struct Shaders {
  pub vertex: vs::Shader,
  pub fragment: fs::Shader,
}


// This can't have stored both a reference and a physical device 
// because the physical device has internally a reference to the instance
// It also can't store only the physical device because it then needs to move to a thread and leaves the reference behind
// So the only fix I could find is to throw everything that uses directly the instance in the main loop
pub struct VulkanProgram {
  pub device: Arc<Device>,
  pub queues: Queues,
  pub swapchain: Arc<Swapchain<Window>>,
  pub buffers: Buffers,
  pub command_buffers: CommandBuffers,
  pub shaders: Shaders,
  pub render_pass: Arc<RenderPass>,
  pub viewport: Viewport,
  pub framebuffers: Vec<Arc<dyn FramebufferAbstract>>,
  pub pipeline: Arc<GraphicsPipeline>,
}

impl VulkanProgram {
  pub fn start(device_extensions: DeviceExtensions, physical_device: PhysicalDevice, queue_families: &QueueFamilies, surface: &Arc<Surface<Window>>) -> Self {

    let (device, queues) =
      init::get_logical_device_and_queues(&physical_device, &device_extensions, &queue_families);

    let (swapchain, images) = init::get_swapchain(&physical_device, &device, surface, &queues);

    let buffers = init::initialize_and_get_buffers(&device, &queue_families);
    let command_buffers = init::load_command_buffers(&device, &queue_families, &buffers);

    // populate staging vertex buffer for the first time
    init::update_staging_buffer(&device, &queues, &command_buffers);

    let shaders = init::load_shaders(&device);

    let render_pass = init::get_render_pass(&device, &swapchain);

    let dimensions = images[0].dimensions();
    let viewport = Viewport {
      origin: [0.0, 0.0],
      dimensions: [dimensions[0] as f32, dimensions[1] as f32],
      depth_range: 0.0..1.0,
    };

    let framebuffers = init::get_framebuffers(&images, render_pass.clone());
    let pipeline = init::get_pipeline(&device, &shaders, &render_pass, &images[0].dimensions());

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

    self.swapchain = new_swapchain;
    self.framebuffers = init::get_framebuffers(&new_images, self.render_pass.clone());
    let dimensions = new_images[0].dimensions();
    self.viewport.dimensions = [dimensions[0] as f32, dimensions[1] as f32];
    self.pipeline = init::get_pipeline(
      &self.device,
      &self.shaders,
      &self.render_pass,
      &new_images[0].dimensions(),
    );
  }
}
