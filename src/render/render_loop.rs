use crate::{
  render::{renderer::Renderer, Camera},
  Scene,
};
use std::sync::Arc;
use vulkano::{
  swapchain::{AcquireError, PresentFuture},
  sync::{FenceSignalFuture, FlushError, GpuFuture},
};
use winit::{event_loop::EventLoop, window::Window};

type FenceFuture = FenceSignalFuture<PresentFuture<Box<dyn GpuFuture>, Window>>;

/// Manages synchronization with fences and is responsible for keeping all components working together
/// each frame.
///
/// The program operates on a number of main command buffers equal to the framebuffer count, as each
/// operates on that specific framebuffer. These are not recreated each frame, so (for simplicity) the number of frames
/// in flight is equal to the number of command buffers, and corelate directly with the swapchain image index.
pub struct RenderLoop {
  renderer: Renderer,
  recreate_swapchain: bool,
  window_resized: bool,
  fences: Vec<Option<Arc<FenceFuture>>>,
  previous_fence_i: usize,
  update_buffer_models: Vec<bool>,
}

impl<'a> RenderLoop {
  pub fn new(event_loop: &EventLoop<()>, scene: &Scene) -> Self {
    let renderer = Renderer::initialize(event_loop, scene);

    let frames_in_flight = renderer.get_image_count();
    let fences = vec![None; frames_in_flight];

    Self {
      renderer,
      recreate_swapchain: false,
      window_resized: false,
      fences,
      previous_fence_i: 0,
      update_buffer_models: vec![false; frames_in_flight],
    }
  }

  /// - Handles window resizing and swapchain recreation;
  /// - Acquires next swapchain image
  /// - Waits for fences
  /// - Updates components calls for buffer update commands
  /// - Flushes next future
  pub fn update(&mut self, camera: &Camera, scene: &Scene) {
    if self.window_resized {
      self.window_resized = false;
      self.recreate_swapchain = false;
      // automatically recreates the swapchain
      self.renderer.handle_window_resize();
    }
    if self.recreate_swapchain {
      self.recreate_swapchain = false;
      self.renderer.recreate_swapchain();
    }

    // for tests
    // std::thread::sleep(std::time::Duration::from_millis(1000));

    let (image_i, suboptimal, acquire_future) = match self.renderer.acquire_next_swapchain_image() {
      Ok(r) => r,
      Err(AcquireError::OutOfDate) => {
        self.recreate_swapchain = true;
        return;
      }
      Err(e) => panic!("Failed to acquire next image: {:?}", e),
    };

    if suboptimal {
      self.recreate_swapchain = true;
    }

    let oldest_fence_exists = if let Some(cur_fence) = &mut self.fences[image_i] {
      // current fence will be the oldest flushed one, so waiting it won't wait upon the other futures execution
      cur_fence.wait(None).unwrap();
      true
    } else {
      false
    };

    if scene.objects_changed {
      self.update_buffer_models.fill(true);
    }
    let update_buffer_models = self.update_buffer_models[image_i];
    self.update_buffer_models[image_i] = false;

    if let Some(fence) = &mut self.fences[self.previous_fence_i].clone() {
      let something_needs_all_gpu_resources = update_buffer_models;
      if something_needs_all_gpu_resources || !oldest_fence_exists {
        // This fence corresponds to the earliest flushed one, so waiting it will block the CPU until GPU finishes all operations
        fence.wait(None).unwrap();
      }

      // todo: Currently there is a single instance buffer where in the main execution future other buffers get copied to it
      // When a copy operation happens, the current source and destination buffers get locked, which means they should automatically
      // unlock after calling "cur_fence.wait()", because it is supposed to clean and unlock all unused resources
      // However, buffers for the current frame continue to be locked
      // See "self.renderer.flush_next_future" for more information about execution
      // I will do more research for this, but for now "something_needs_all_gpu_resources" will be true when this operation happens
      if update_buffer_models {
        self.renderer.update_buffer_models(image_i, scene);
      }
    }

    let previous_future = match self.fences[self.previous_fence_i].clone() {
      None => self.renderer.synchronize().boxed(),
      Some(fence) => fence.boxed(),
    };

    let result = self.renderer.flush_next_future(
      previous_future,
      acquire_future,
      image_i,
      camera,
      scene.total_object_count,
    );

    self.fences[image_i] = match result {
      Ok(fence) => Some(Arc::new(fence)),
      Err(FlushError::OutOfDate) => {
        println!("out of date");
        self.recreate_swapchain = true;
        None
      }
      Err(e) => {
        println!("Failed to flush future: {:?}", e);
        None
      }
    };

    self.previous_fence_i = image_i;
  }

  /// Signal that window should be handled in the next update
  pub fn handle_window_resize(&mut self) {
    self.window_resized = true;
  }

  /// Returns surface window
  pub fn get_window(&self) -> &Window {
    self.renderer.get_surface_window()
  }
}
