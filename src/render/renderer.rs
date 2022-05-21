use crate::app::Scene;
use crate::game_objects::RenderableIn3d;
use crate::render::shaders::single_colored;
use crate::render::vulkano_objects;
use crate::render::BufferContainer;
use crate::render::Camera;
use std::sync::Arc;
use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo};
use vulkano::image::SwapchainImage;
use vulkano::instance::Instance;
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::pipeline::{GraphicsPipeline, Pipeline};
use vulkano::render_pass::{Framebuffer, RenderPass};
use vulkano::shader::ShaderModule;
use vulkano::swapchain::{
  self, AcquireError, PresentFuture, Surface, Swapchain, SwapchainAcquireFuture,
  SwapchainCreateInfo, SwapchainCreationError,
};
use vulkano::sync::{self, FenceSignalFuture, FlushError, GpuFuture, NowFuture};
use vulkano_win::VkSurfaceBuild;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

pub type Fence = FenceSignalFuture<PresentFuture<Box<dyn GpuFuture>, Window>>;

pub struct Renderer {
  surface: Arc<Surface<Window>>,
  _instance: Arc<Instance>,
  device: Arc<Device>,
  queue: Arc<Queue>,
  swapchain: Arc<Swapchain<Window>>,
  images: Vec<Arc<SwapchainImage<winit::window::Window>>>,
  render_pass: Arc<RenderPass>,
  framebuffers: Vec<Arc<Framebuffer>>,
  vertex_shader: Arc<ShaderModule>,
  fragment_shader: Arc<ShaderModule>,
  viewport: Viewport,
  pipeline: Arc<GraphicsPipeline>,
  buffer_container: BufferContainer,
}

impl<'a> Renderer {
  pub fn initialize(event_loop: &EventLoop<()>) -> Self {
    let instance = vulkano_objects::instance::get_instance();

    let surface = WindowBuilder::new()
      .build_vk_surface(&event_loop, instance.clone())
      .unwrap();

    let device_extensions = DeviceExtensions {
      khr_swapchain: true,
      khr_storage_buffer_storage_class: true,
      ..DeviceExtensions::none()
    };

    let (physical_device, queue_family) = vulkano_objects::physical_device::select_physical_device(
      &instance,
      surface.clone(),
      &device_extensions,
    );

    let (device, mut queues) = Device::new(
      physical_device,
      DeviceCreateInfo {
        queue_create_infos: vec![QueueCreateInfo::family(queue_family)],
        enabled_extensions: physical_device
          .required_extensions()
          .union(&device_extensions), // new
        ..Default::default()
      },
    )
    .expect("failed to create device");

    let queue = queues.next().unwrap();

    let (swapchain, images) = vulkano_objects::swapchain::create_swapchain(
      &physical_device,
      device.clone(),
      surface.clone(),
    );

    let render_pass =
      vulkano_objects::render_pass::create_render_pass(device.clone(), swapchain.clone());
    let framebuffers = vulkano_objects::swapchain::create_framebuffers_from_swapchain_images(
      &images,
      render_pass.clone(),
    );

    let vertex_shader =
      single_colored::vs::load(device.clone()).expect("failed to create shader module");
    let fragment_shader =
      single_colored::fs::load(device.clone()).expect("failed to create shader module");

    let viewport = Viewport {
      origin: [0.0, 0.0],
      dimensions: surface.window().inner_size().into(),
      depth_range: 0.0..1.0,
    };

    let pipeline = vulkano_objects::pipeline::create_pipeline(
      device.clone(),
      vertex_shader.clone(),
      fragment_shader.clone(),
      render_pass.clone(),
      viewport.clone(),
    );

    let descriptor_set_layout = pipeline.layout().set_layouts().get(0).unwrap().clone();

    let buffer_container = BufferContainer::new::<single_colored::vs::ty::Data, single_colored::S>(
      device.clone(),
      pipeline.clone(),
      descriptor_set_layout,
      &framebuffers,
      queue.clone(),
    );

    Self {
      surface,
      device,
      queue,
      swapchain,
      images,
      render_pass,
      framebuffers,
      vertex_shader,
      fragment_shader,
      viewport,
      pipeline,
      buffer_container,
      _instance: instance,
    }
  }

  pub fn recreate_swapchain(&mut self) {
    let (new_swapchain, new_images) = match self.swapchain.recreate(SwapchainCreateInfo {
      image_extent: self.surface.window().inner_size().into(),
      ..self.swapchain.create_info()
    }) {
      Ok(r) => r,
      Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => return,
      Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
    };

    self.swapchain = new_swapchain;
    self.framebuffers = vulkano_objects::swapchain::create_framebuffers_from_swapchain_images(
      &new_images,
      self.render_pass.clone(),
    );
  }

  pub fn handle_window_resize(&mut self) {
    self.recreate_swapchain();
    self.viewport.dimensions = self.surface.window().inner_size().into();

    self.pipeline = vulkano_objects::pipeline::create_pipeline(
      self.device.clone(),
      self.vertex_shader.clone(),
      self.fragment_shader.clone(),
      self.render_pass.clone(),
      self.viewport.clone(),
    );

    self.buffer_container.handle_window_resize(
      self.device.clone(),
      self.pipeline.clone(),
      &self.framebuffers,
      self.queue.clone(),
    );
  }

  pub fn get_image_count(&self) -> usize {
    self.images.len()
  }

  pub fn acquire_swapchain_image(
    &self,
  ) -> Result<(usize, bool, SwapchainAcquireFuture<Window>), AcquireError> {
    swapchain::acquire_next_image(self.swapchain.clone(), None)
  }

  pub fn synchronize(&self) -> NowFuture {
    let mut now = sync::now(self.device.clone());
    now.cleanup_finished();

    now
  }

  pub fn flush_next_future(
    &self,
    previous_future: Box<dyn GpuFuture>,
    swapchain_acquire_future: SwapchainAcquireFuture<Window>,
    image_i: usize,
  ) -> Result<Fence, FlushError> {
    // join with swapchain future, draw and then present, signal fence and flush
    let boxed: Box<dyn GpuFuture> = Box::new(
      previous_future
        .join(swapchain_acquire_future)
        .then_execute(
          self.queue.clone(),
          self.buffer_container.command_buffers[image_i].clone(),
        )
        .unwrap(),
    );

    boxed
      .then_swapchain_present(self.queue.clone(), self.swapchain.clone(), image_i)
      .then_signal_fence_and_flush()
  }

  pub fn update_uniform(&mut self, buffer_i: usize, camera: &Camera, scene_objects: &Scene) {
    let projection_view = camera.get_projection_view();

    let cube = &scene_objects.cube;
    let cube_matrix = projection_view * cube.get_model_matrix();
    let cube_data = single_colored::vs::ty::Data {
      color: cube.color,
      matrix: cube_matrix.into(),
    };

    let square = &scene_objects.square;
    let square_matrix = projection_view * square.get_model_matrix();
    let square_data = single_colored::vs::ty::Data {
      color: square.color,
      matrix: square_matrix.into(),
    };

    self
      .buffer_container
      .update_uniform(buffer_i, cube_data, square_data);
  }

  pub fn get_surface_window(&self) -> &Window {
    self.surface.window()
  }
}
