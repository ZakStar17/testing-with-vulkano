use crate::vulkano_graphics::Queues;

use std::sync::Arc;
use vulkano::device::physical::PhysicalDevice;
use vulkano::device::Device;
use vulkano::image::view::ImageView;
use vulkano::image::{ImageUsage, SwapchainImage};
use vulkano::render_pass::{Framebuffer, RenderPass};
use vulkano::swapchain::Surface;
use vulkano::swapchain::Swapchain;
use vulkano::swapchain::SwapchainCreationError;
use winit::window::Window;

pub struct SwapchainData {
  pub swapchain: Arc<Swapchain<Window>>,
  pub image_count: u32,
  pub images: Vec<Arc<SwapchainImage<Window>>>,
  pub render_pass: Arc<RenderPass>,
  pub framebuffers: Vec<Arc<Framebuffer>>,
}

impl SwapchainData {
  pub fn init(
    physical_device: PhysicalDevice,
    logical_device: Arc<Device>,
    surface: Arc<Surface<Window>>,
    queues: &Queues,
    image_count: u32,
  ) -> Self {
    let caps = surface.capabilities(physical_device).unwrap();

    let composite_alpha = caps.supported_composite_alpha.iter().next().unwrap();

    let format = caps.supported_formats[0].0;

    let dimensions: [u32; 2] = surface.window().inner_size().into();

    let (swapchain, images) = Swapchain::start(logical_device.clone(), surface.clone())
      .num_images(image_count)
      .format(format)
      .dimensions(dimensions)
      .usage(ImageUsage::color_attachment())
      .sharing_mode(&queues.graphical)
      .composite_alpha(composite_alpha)
      .build()
      .unwrap();

    let render_pass = Self::get_render_pass(logical_device, swapchain.clone());

    let framebuffers = Self::get_framebuffers(&images, render_pass.clone());

    Self {
      swapchain,
      image_count,
      images,
      render_pass,
      framebuffers,
    }
  }

  fn get_render_pass(device: Arc<Device>, swapchain: Arc<Swapchain<Window>>) -> Arc<RenderPass> {
    vulkano::single_pass_renderpass!(
        device.clone(),
        attachments: {
            color: {
                load: Clear,
                store: Store,
                format: swapchain.format(),
                samples: 1,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {}
        }
    )
    .unwrap()
  }

  fn get_framebuffers(
    images: &[Arc<SwapchainImage<Window>>],
    render_pass: Arc<RenderPass>,
  ) -> Vec<Arc<Framebuffer>> {
    images
      .iter()
      .map(|image| {
        let view = ImageView::new(image.clone()).unwrap();
        Framebuffer::start(render_pass.clone())
          .add(view)
          .unwrap()
          .build()
          .unwrap()
      })
      .collect::<Vec<_>>()
  }

  pub fn recreate(&mut self, new_dimensions: &[u32; 2]) {
    let (new_swapchain, new_images) = match self
      .swapchain
      .recreate()
      .dimensions(*new_dimensions)
      .build()
    {
      Ok(r) => r,
      Err(SwapchainCreationError::UnsupportedDimensions) => return,
      Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
    };

    // recreate swapchain
    self.swapchain = new_swapchain;
    self.framebuffers = Self::get_framebuffers(&new_images, self.render_pass.clone());
  }
}
