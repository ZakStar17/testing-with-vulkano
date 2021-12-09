use std::sync::Arc;
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::device::DeviceExtensions;
use vulkano::instance::Instance;
use vulkano::swapchain::Surface;
use winit::window::Window;

pub fn get<'a>(
  instance: &'a Arc<Instance>,
  device_extensions: &DeviceExtensions,
  surface: Arc<Surface<Window>>,
) -> PhysicalDevice<'a> {
  let device = PhysicalDevice::enumerate(instance)
    .filter(|&p| p.supported_extensions().is_superset_of(device_extensions))
    .filter(|p| {
      p.queue_families()
        // find any queue that supports graphics
        .any(|q| q.supports_graphics() && surface.is_supported(q).unwrap_or(false))
        && p
          .queue_families()
          // find any queue only for compute
          .any(|q| q.supports_compute() && !q.supports_graphics())
    })
    // if there are multiple, match the most likely to be better
    .min_by_key(|p| match p.properties().device_type {
      PhysicalDeviceType::DiscreteGpu => 0,
      PhysicalDeviceType::IntegratedGpu => 1,
      PhysicalDeviceType::VirtualGpu => 2,
      PhysicalDeviceType::Cpu => 3,
      PhysicalDeviceType::Other => 4,
    })
    .unwrap();

  println!(
    "Using device: {} (type: {:?})",
    device.properties().device_name,
    device.properties().device_type,
  );

  device
}
