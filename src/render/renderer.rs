use crate::{
  render::{
    buffer_container::BufferContainer, shaders::single_colored,
    swapchain_container::SwapchainContainer, vulkano_objects, Camera,
  },
  Scene,
};
use std::sync::Arc;
use vulkano::{
  device::{Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo},
  instance::Instance,
  pipeline::{graphics::viewport::Viewport, GraphicsPipeline},
  shader::ShaderModule,
  swapchain::{AcquireError, PresentFuture, Surface, SwapchainAcquireFuture},
  sync::{self, FenceSignalFuture, FlushError, GpuFuture, NowFuture},
};
use vulkano_win::VkSurfaceBuild;
use winit::{
  event_loop::EventLoop,
  window::{Window, WindowBuilder},
};

pub type Fence = FenceSignalFuture<PresentFuture<Box<dyn GpuFuture>, Window>>;

pub struct Renderer {
  surface: Arc<Surface<Window>>,
  _instance: Arc<Instance>,
  device: Arc<Device>,
  queue: Arc<Queue>,
  swapchain_container: SwapchainContainer,
  vertex_shader: Arc<ShaderModule>,
  fragment_shader: Arc<ShaderModule>,
  viewport: Viewport,
  pipeline: Arc<GraphicsPipeline>,
  pub buffer_container: BufferContainer,
}

impl<'a> Renderer {
  pub fn initialize(event_loop: &EventLoop<()>, scene: &Scene) -> Self {
    let instance = vulkano_objects::instance::create();

    let surface = WindowBuilder::new()
      .build_vk_surface(&event_loop, instance.clone())
      .unwrap();

    let device_extensions = DeviceExtensions {
      khr_swapchain: true,
      khr_storage_buffer_storage_class: true,
      ..DeviceExtensions::none()
    };

    let (physical_device, queue_family) =
      vulkano_objects::physical_device::select(&instance, surface.clone(), &device_extensions);

    let (device, mut queues) = Device::new(
      physical_device,
      DeviceCreateInfo {
        queue_create_infos: vec![QueueCreateInfo::family(queue_family)],
        enabled_extensions: physical_device
          .required_extensions()
          .union(&device_extensions),
        ..Default::default()
      },
    )
    .expect("failed to create device");

    let queue = queues.next().unwrap();

    let swapchain_container =
      SwapchainContainer::new(physical_device, device.clone(), surface.clone());

    let vertex_shader =
      single_colored::vs::load(device.clone()).expect("failed to create shader module");
    let fragment_shader =
      single_colored::fs::load(device.clone()).expect("failed to create shader module");

    let viewport = Viewport {
      origin: [0.0, 0.0],
      dimensions: surface.window().inner_size().into(),
      depth_range: 0.0..1.0,
    };

    let pipeline = vulkano_objects::pipeline::create(
      device.clone(),
      vertex_shader.clone(),
      fragment_shader.clone(),
      swapchain_container.get_render_pass(),
      viewport.clone(),
    );

    let buffer_container = BufferContainer::new(
      device.clone(),
      pipeline.clone(),
      swapchain_container.get_framebuffers(),
      queue.clone(),
      scene,
      [queue_family],
    );

    Self {
      surface,
      device,
      queue,
      swapchain_container,
      vertex_shader,
      fragment_shader,
      viewport,
      pipeline,
      buffer_container,
      _instance: instance,
    }
  }

  pub fn recreate_swapchain(&mut self) {
    self
      .swapchain_container
      .recreate_swapchain(self.device.clone(), self.surface.clone())
  }

  pub fn handle_window_resize(&mut self) {
    self
      .swapchain_container
      .recreate_swapchain(self.device.clone(), self.surface.clone());
    self.viewport.dimensions = self.surface.window().inner_size().into();

    self.pipeline = vulkano_objects::pipeline::create(
      self.device.clone(),
      self.vertex_shader.clone(),
      self.fragment_shader.clone(),
      self.swapchain_container.get_render_pass(),
      self.viewport.clone(),
    );

    self.buffer_container.handle_window_resize(
      self.device.clone(),
      self.pipeline.clone(),
      self.swapchain_container.get_framebuffers(),
      self.queue.clone(),
    );
  }

  pub fn get_image_count(&self) -> usize {
    self.swapchain_container.image_count()
  }

  pub fn acquire_next_swapchain_image(
    &self,
  ) -> Result<(usize, bool, SwapchainAcquireFuture<Window>), AcquireError> {
    self.swapchain_container.acquire_next_swapchain_image()
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

    let command_buffers = self.buffer_container.command_buffers();
    let boxed: Box<dyn GpuFuture> = Box::new(
      previous_future
        .then_execute(
          self.queue.clone(),
          command_buffers.instance_copy[image_i].clone(),
        )
        .unwrap()
        .then_signal_semaphore()
        .join(swapchain_acquire_future)
        .then_execute(self.queue.clone(), command_buffers.main[image_i].clone())
        .unwrap(),
    );

    boxed
      .then_swapchain_present(
        self.queue.clone(),
        self.swapchain_container.get_swapchain(),
        image_i,
      )
      .then_signal_fence_and_flush()
  }

  pub fn update_matrices(&mut self, buffer_i: usize, camera: &Camera, scene: &Scene) {
    self
      .buffer_container
      .update_matrices(buffer_i, scene, camera.get_projection_view());
  }

  pub fn get_surface_window(&self) -> &Window {
    self.surface.window()
  }
}
