use std::sync::Arc;
use vulkano::device::{Queue, QueueCreateInfo};

use vulkano::{
  device::{
    physical::{PhysicalDevice, PhysicalDeviceType, QueueFamily},
    DeviceExtensions,
  },
  instance::Instance,
  swapchain::Surface,
};
use winit::window::Window;

pub struct QueueFamilies<'a> {
  pub graphics: QueueFamily<'a>,
  pub compute: QueueFamily<'a>,
  pub transfers: QueueFamily<'a>,
}

pub struct Queues {
  pub graphics: Arc<Queue>,
  pub compute: Arc<Queue>,
  pub transfers: Arc<Queue>,
}

impl<'a> QueueFamilies<'a> {
  pub fn get_queue_create_info(&self) -> Vec<QueueCreateInfo<'a>> {
    vec![
      QueueCreateInfo::family(self.graphics),
      QueueCreateInfo::family(self.compute),
      QueueCreateInfo::family(self.transfers),
    ]
  }

  pub fn get_queues(iter: &mut (impl ExactSizeIterator + Iterator<Item = Arc<Queue>>)) -> Queues {
    // same order as above
    Queues {
      graphics: iter.next().unwrap(),
      compute: iter.next().unwrap(),
      transfers: iter.next().unwrap(),
    }
  }
}

pub fn select<'a>(
  instance: &'a Arc<Instance>,
  surface: Arc<Surface<Window>>,
  device_extensions: &DeviceExtensions,
) -> (PhysicalDevice<'a>, QueueFamilies<'a>) {
  let (physical_device, queue_family) = PhysicalDevice::enumerate(&instance)
    .filter(|&p| p.supported_extensions().is_superset_of(&device_extensions))
    .filter_map(|p| {
      let mut graphics = None;
      let mut compute = None;
      let mut transfers = None;
      for family in p.queue_families() {
        if family.supports_graphics() && family.supports_surface(&surface).unwrap_or(false) {
          graphics = Some(family);
        } else if family.supports_compute() {
          compute = Some(family)
        } else if family.explicitly_supports_transfers() {
          transfers = Some(family);
        }
      }

      if let Some(graphics) = graphics {
        Some((
          p,
          QueueFamilies {
            graphics,
            compute: if let Some(family) = compute {
              family
            } else {
              graphics
            },
            transfers: if let Some(family) = transfers {
              family
            } else {
              graphics
            },
          },
        ))
      } else {
        None
      }
    })
    .min_by_key(|(p, _)| match p.properties().device_type {
      PhysicalDeviceType::DiscreteGpu => 0,
      PhysicalDeviceType::IntegratedGpu => 1,
      PhysicalDeviceType::VirtualGpu => 2,
      PhysicalDeviceType::Cpu => 3,
      PhysicalDeviceType::Other => 4,
    })
    .expect("no device available");

  (physical_device, queue_family)
}
