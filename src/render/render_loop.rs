use crate::{render::Camera, Scene};
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

    if let Some(fence) = &mut self.fences[self.previous_fence_i].clone() {
      let something_needs_all_gpu_resources = true;
      if something_needs_all_gpu_resources || !oldest_fence_exists {
        // here fence corresponds to the earliest flushed one, so waiting it will block the CPU until GPU finishes all operations
        fence.wait(None).unwrap();

        // here goes the code that updates all buffers
      }
      fence.cleanup_finished();  // this should do anything, but maybe it will help

      // code that updates buffers related to a single image

      // todo: Currently there is a single instance buffer where in the main execution future other buffers get copied to it
      // When a copy operation happens, it should only lock the current source buffer (and then unlock it after the fence)
      // this means that "cur_fence.wait()" should make the destination buffer useful again
      // but for some reason, flushing a copy buffer locks every single one of them (maybe because they all write to instance)
      // See "self.renderer.flush_next_future"
      // I will try to create multiple instance buffers and see if it works (or else I guess I should find an unsafe way to do this)
      self.renderer.update_matrices(image_i, camera, scene);
    }

    let previous_future = match self.fences[self.previous_fence_i].clone() {
      None => self.renderer.synchronize().boxed(),
      Some(fence) => fence.boxed(),
    };

    let result = self
      .renderer
      .flush_next_future(previous_future, acquire_future, image_i);

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

  pub fn handle_window_resize(&mut self) {
    // impacts the next update
    self.window_resized = true;
  }

  pub fn get_window(&self) -> &Window {
    self.renderer.get_surface_window()
  }
}
