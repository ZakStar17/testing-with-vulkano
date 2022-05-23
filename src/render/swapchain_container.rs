use crate::render::vulkano_objects;
use std::sync::Arc;
use vulkano::{
  device::{physical::PhysicalDevice, Device},
  format::Format,
  image::{traits::ImageAccess, AttachmentImage, ImageDimensions, SwapchainImage},
  render_pass::{Framebuffer, RenderPass},
  swapchain::{
    self, AcquireError, Surface, Swapchain, SwapchainAcquireFuture, SwapchainCreateInfo,
    SwapchainCreationError,
  },
};
use winit::window::Window;

// contains everything that depends on the swapchain
pub struct SwapchainContainer {
  swapchain: Arc<Swapchain<Window>>,
  swapchain_images: Vec<Arc<SwapchainImage<winit::window::Window>>>,
  depth_image: Arc<AttachmentImage>,
  render_pass: Arc<RenderPass>,
  framebuffers: Vec<Arc<Framebuffer>>,
}

impl SwapchainContainer {
  pub fn new(
    physical_device: PhysicalDevice,
    device: Arc<Device>,
    surface: Arc<Surface<Window>>,
  ) -> Self {
    let (swapchain, swapchain_images) =
      vulkano_objects::swapchain::create(&physical_device, device.clone(), surface);

    let depth_image = AttachmentImage::transient_input_attachment(
      device.clone(),
      get_2d_image_dimensions(&swapchain_images[0]),
      Format::D32_SFLOAT, // todo: search for available depth format
    )
    .unwrap();

    let render_pass =
      vulkano_objects::render_pass::create(device, swapchain.clone(), depth_image.clone());
    let framebuffers = vulkano_objects::framebuffers::create(
      render_pass.clone(),
      &swapchain_images,
      depth_image.clone(),
    );

    Self {
      swapchain,
      swapchain_images,
      depth_image,
      render_pass,
      framebuffers,
    }
  }

  pub fn recreate_swapchain(&mut self, device: Arc<Device>, surface: Arc<Surface<Window>>) {
    let (new_swapchain, new_swapchain_images) = match self.swapchain.recreate(SwapchainCreateInfo {
      image_extent: surface.window().inner_size().into(),
      ..self.swapchain.create_info()
    }) {
      Ok(r) => r,
      Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => return,
      Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
    };

    self.swapchain = new_swapchain;

    self.depth_image = AttachmentImage::transient_input_attachment(
      device,
      get_2d_image_dimensions(&new_swapchain_images[0]),
      Format::D32_SFLOAT,
    )
    .unwrap();

    self.framebuffers = vulkano_objects::framebuffers::create(
      self.render_pass.clone(),
      &new_swapchain_images,
      self.depth_image.clone(),
    );
  }

  pub fn acquire_next_swapchain_image(
    &self,
  ) -> Result<(usize, bool, SwapchainAcquireFuture<Window>), AcquireError> {
    swapchain::acquire_next_image(self.swapchain.clone(), None)
  }

  pub fn get_render_pass(&self) -> Arc<RenderPass> {
    self.render_pass.clone()
  }

  pub fn get_framebuffers(&self) -> &Vec<Arc<Framebuffer>> {
    &self.framebuffers
  }

  pub fn get_swapchain(&self) -> Arc<Swapchain<Window>> {
    self.swapchain.clone()
  }

  pub fn image_count(&self) -> usize {
    self.swapchain_images.len()
  }
}

fn get_2d_image_dimensions(image: &dyn ImageAccess) -> [u32; 2] {
  if let ImageDimensions::Dim2d { width, height, .. } = image.dimensions() {
    [width, height]
  } else {
    panic!()
  }
}
