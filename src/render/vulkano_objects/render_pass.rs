use std::sync::Arc;
use vulkano::{
  device::Device,
  image::{traits::ImageAccess, AttachmentImage},
  render_pass::RenderPass,
  swapchain::Swapchain,
};
use winit::window::Window;

pub fn create(
  device: Arc<Device>,
  swapchain: Arc<Swapchain<Window>>,
  depth_image: Arc<AttachmentImage>,
) -> Arc<RenderPass> {
  vulkano::single_pass_renderpass!(
    device.clone(),
    attachments: {
      color: {
        load: Clear,
        store: Store,
        format: swapchain.image_format(),
        samples: 1,
      },
      depth: {
        load: Clear,
        store: DontCare,
        format: depth_image.format(),
        samples: 1,
      }
    },
    pass: {
      color: [color],
      depth_stencil: {depth}
    }
  )
  .unwrap()
}
