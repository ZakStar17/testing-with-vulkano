use std::sync::Arc;
use vulkano::render_pass::FramebufferCreateInfo;

use vulkano::{
  device::{physical::PhysicalDevice, Device},
  image::{view::ImageView, ImageUsage, SwapchainImage},
  render_pass::{Framebuffer, RenderPass},
  swapchain::{Surface, Swapchain, SwapchainCreateInfo},
};
use winit::window::Window;

pub fn create(
  physical_device: &PhysicalDevice,
  device: Arc<Device>,
  surface: Arc<Surface<Window>>,
) -> (Arc<Swapchain<Window>>, Vec<Arc<SwapchainImage<Window>>>) {
  let caps = physical_device
    .surface_capabilities(&surface, Default::default())
    .expect("failed to get surface capabilities");

  let composite_alpha = caps.supported_composite_alpha.iter().next().unwrap();
  let image_format = Some(
    physical_device
      .surface_formats(&surface, Default::default())
      .unwrap()[0]
      .0,
  );

  Swapchain::new(
    device,
    surface.clone(),
    SwapchainCreateInfo {
      min_image_count: caps.min_image_count,
      image_format,
      image_extent: surface.window().inner_size().into(),
      image_usage: ImageUsage::color_attachment(),
      composite_alpha,
      ..Default::default()
    },
  )
  .unwrap()
}

pub fn create_framebuffers_from_swapchain_images(
  images: &[Arc<SwapchainImage<Window>>],
  render_pass: Arc<RenderPass>,
) -> Vec<Arc<Framebuffer>> {
  images
    .iter()
    .map(|image| {
      let view = ImageView::new_default(image.clone()).unwrap();
      Framebuffer::new(
        render_pass.clone(),
        FramebufferCreateInfo {
          attachments: vec![view],
          ..Default::default()
        },
      )
      .unwrap()
    })
    .collect::<Vec<_>>()
}
