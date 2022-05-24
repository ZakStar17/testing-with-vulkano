use crate::Scene;
use crate::render::{Camera};
use std::sync::Arc;
use vulkano::{
  swapchain::AcquireError,
  sync::{FlushError, GpuFuture},
};
use winit::{event_loop::EventLoop, window::Window};

use crate::render::renderer::{Fence, Renderer};

// responsible for CPU level synchronization
pub struct RenderLoop {
  renderer: Renderer,
  recreate_swapchain: bool,
  window_resized: bool,
  fences: Vec<Option<Arc<Fence>>>,
  previous_fence_i: usize,
}

impl<'a> RenderLoop {
  pub fn new(event_loop: &EventLoop<()>, scene: &Scene) -> Self {
    let renderer = Renderer::initialize(event_loop, scene);

    // each main command buffer is created with a specific framebuffer in mid, which depend on the swapchain images
    // what this means is that the number of command buffers is equal to the image count

    // in that sense, it's most effective to use image count as the number of frames in flight, as there is no
    // need to perform distinctions between command buffers and fence futures
    let frames_in_flight = renderer.get_image_count();
    let fences: Vec<Option<Arc<Fence>>> = vec![None; frames_in_flight];

    Self {
      renderer,
      recreate_swapchain: false,
      window_resized: false,
      fences,
      previous_fence_i: 0,
    }
  }

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

    if let Some(cur_fence) = &self.fences[image_i] {
      // current fence will be the oldest flushed one, so waiting it won't wait upon the other futures execution
      cur_fence.wait(None).unwrap();
    }

    // code that uses objects not currently used by the GPU (corresponding to image_i)
    self.renderer.update_matrices(image_i, camera, scene);

    let something_needs_all_gpu_resources = false;
    let previous_future = match self.fences[self.previous_fence_i].clone() {
      None => self.renderer.synchronize().boxed(),
      Some(fence) => {
        if something_needs_all_gpu_resources {
          // here fence corresponds to the earliest flushed one, so waiting it will block the CPU until GPU finishes all operations
          fence.wait(None).unwrap();
        }
        fence.boxed()
      }
    };

    if something_needs_all_gpu_resources {
      // the gpu has been waited, so it's currently idle. Here it's possible to update anything
    }

    let result = self
      .renderer
      .flush_next_future(previous_future, acquire_future, image_i);

    self.fences[image_i] = match result {
      Ok(fence) => Some(Arc::new(fence)),
      Err(FlushError::OutOfDate) => {
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

  pub fn handle_window_resize(&mut self) {
    // impacts the next update
    self.window_resized = true;
  }

  pub fn get_window(&self) -> &Window {
    self.renderer.get_surface_window()
  }
}
