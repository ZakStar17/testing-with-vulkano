use std::sync::Arc;
use vulkano::{image::AttachmentImage, render_pass::FramebufferCreateInfo};

use vulkano::{
  image::{view::ImageView, SwapchainImage},
  render_pass::{Framebuffer, RenderPass},
};
use winit::window::Window;

pub fn create(
  render_pass: Arc<RenderPass>,
  swapchain_images: &[Arc<SwapchainImage<Window>>],
  depth_image: Arc<AttachmentImage>,
) -> Vec<Arc<Framebuffer>> {
  let depth_view = ImageView::new_default(depth_image).unwrap();
  swapchain_images
    .iter()
    .map(|image| {
      let view = ImageView::new_default(image.clone()).unwrap();
      Framebuffer::new(
        render_pass.clone(),
        FramebufferCreateInfo {
          attachments: vec![view, depth_view.clone()],
          ..Default::default()
        },
      )
      .unwrap()
    })
    .collect::<Vec<_>>()
}
