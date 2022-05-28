use crate::{
  render::{
    buffer_container::BufferContainer,
    shaders::{compute, single_colored},
    swapchain_container::SwapchainContainer,
    vertex_data::{MatrixInstance, Vertex3d},
    vulkano_objects,
    vulkano_objects::{QueueFamilies, Queues},
    Camera,
  },
  Scene,
};
use std::sync::Arc;
use vulkano::{
  device::{Device, DeviceCreateInfo, DeviceExtensions},
  instance::Instance,
  pipeline::{graphics::viewport::Viewport, ComputePipeline, GraphicsPipeline},
  shader::ShaderModule,
  swapchain::{AcquireError, PresentFuture, Surface, SwapchainAcquireFuture},
  sync::{self, FenceSignalFuture, FlushError, GpuFuture, NowFuture},
};
use vulkano_win::VkSurfaceBuild;
use winit::{
  event_loop::EventLoop,
  window::{Window, WindowBuilder},
};

/// Contains arbitrary functions that modify certain Vulkano objects.
///
/// Doesn't handle synchronization (see `RenderLoop`).
pub struct Renderer {
  surface: Arc<Surface<Window>>,
  _instance: Arc<Instance>,
  device: Arc<Device>,
  queues: Queues,
  swapchain_container: SwapchainContainer,
  vertex_shader: Arc<ShaderModule>,
  fragment_shader: Arc<ShaderModule>,
  viewport: Viewport,
  graphics_pipeline: Arc<GraphicsPipeline>,
  compute_pipeline: Arc<ComputePipeline>,
  buffer_container: BufferContainer,
}

impl<'a> Renderer {
  /// Creates all main Vulkano objects and saves them
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

    let (physical_device, queue_families) =
      vulkano_objects::physical_device::select(&instance, surface.clone(), &device_extensions);

    let (device, mut iter) = Device::new(
      physical_device,
      DeviceCreateInfo {
        queue_create_infos: queue_families.get_queue_create_info(),
        enabled_extensions: physical_device
          .required_extensions()
          .union(&device_extensions),
        ..Default::default()
      },
    )
    .expect("failed to create device");

    let queues = QueueFamilies::get_queues(&mut iter);

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

    let graphics_pipeline = vulkano_objects::pipeline::create_graphics(
      device.clone(),
      vertex_shader.clone(),
      fragment_shader.clone(),
      swapchain_container.get_render_pass(),
      viewport.clone(),
    );

    let compute_pipeline = vulkano_objects::pipeline::create_compute(
      device.clone(),
      compute::instance::load(device.clone()).unwrap(),
    );

    let buffer_container = BufferContainer::new(
      device.clone(),
      &queue_families,
      &queues,
      graphics_pipeline.clone(),
      compute_pipeline.clone(),
      swapchain_container.get_framebuffers(),
      scene,
    );

    Self {
      surface,
      device,
      queues,
      swapchain_container,
      vertex_shader,
      fragment_shader,
      viewport,
      graphics_pipeline,
      compute_pipeline,
      buffer_container,
      _instance: instance,
    }
  }

  pub fn recreate_swapchain(&mut self) {
    self
      .swapchain_container
      .recreate_swapchain(self.device.clone(), self.surface.clone())
  }

  /// Recreates swapchain, pipeline and everything that depends on them
  pub fn handle_window_resize(&mut self) {
    self
      .swapchain_container
      .recreate_swapchain(self.device.clone(), self.surface.clone());
    self.viewport.dimensions = self.surface.window().inner_size().into();

    self.graphics_pipeline = vulkano_objects::pipeline::create_graphics(
      self.device.clone(),
      self.vertex_shader.clone(),
      self.fragment_shader.clone(),
      self.swapchain_container.get_render_pass(),
      self.viewport.clone(),
    );

    self.buffer_container.handle_window_resize(
      self.device.clone(),
      self.queues.graphics.clone(),
      self.graphics_pipeline.clone(),
      self.swapchain_container.get_framebuffers(),
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

  /// Returns a future representing "now" and cleans unused resources
  pub fn synchronize(&self) -> NowFuture {
    let mut now = sync::now(self.device.clone());
    now.cleanup_finished();

    now
  }

  /// Takes a future and appends all commands that will get executed this frame (in flight)
  pub fn flush_next_future(
    &self,
    previous_future: Box<dyn GpuFuture>,
    swapchain_acquire_future: SwapchainAcquireFuture<Window>,
    image_i: usize,
    camera: &Camera,
    instance_count: usize,
  ) -> Result<FenceSignalFuture<PresentFuture<Box<dyn GpuFuture>, Window>>, FlushError> {
    // join with swapchain future, draw and then present, signal fence and flush

    let command_buffers = self.buffer_container.command_buffers();
    let descriptor_sets = self.buffer_container.descriptor_sets();

    // updates instance buffer by calculating projection-view-model matrices
    let instance_compute_command_buffer =
      vulkano_objects::command_buffers::create_instance_compute::<Vertex3d, MatrixInstance, _>(
        self.device.clone(),
        self.queues.compute.clone(),
        self.compute_pipeline.clone(),
        descriptor_sets.instance[image_i].clone(),
        compute::instance::ty::PushConstantData {
          projection_view: camera.get_projection_view().into(),
        },
        instance_count,
      );

    let with_main: Box<dyn GpuFuture> = Box::new(
      previous_future
        .then_execute(self.queues.compute.clone(), instance_compute_command_buffer)
        .unwrap()
        .then_signal_semaphore()
        .join(swapchain_acquire_future)
        .then_execute(
          self.queues.graphics.clone(),
          command_buffers.main[image_i].clone(),
        )
        .unwrap(),
    );

    with_main
      .then_swapchain_present(
        self.queues.graphics.clone(),
        self.swapchain_container.get_swapchain(),
        image_i,
      )
      .then_signal_fence_and_flush()
  }

  pub fn update_buffer_models(&mut self, buffer_i: usize, scene: &Scene) {
    self
      .buffer_container
      .update_buffer_models(buffer_i, scene);
  }

  pub fn get_surface_window(&self) -> &Window {
    self.surface.window()
  }
}
