use crate::vulkano_graphics::{QueueFamilies};

use std::sync::Arc;
use vulkano::device::physical::PhysicalDevice;
use vulkano::device::{Device, Features, DeviceExtensions, Queue};

pub struct Queues {
  pub graphical: Arc<Queue>,
  pub compute: Arc<Queue>,
  pub transfers: Arc<Queue>,
}

pub fn create_with_queues(
  physical_device: &PhysicalDevice,
  extensions: &DeviceExtensions,
  queue_families: &QueueFamilies,
) -> (Arc<Device>, Queues) {
  let (device, mut queues) = Device::new(
    *physical_device,
    &Features::none(),
    &physical_device.required_extensions().union(&extensions),
    // I don't understand what the priorities actually do, so this is kinda arbitrary
    [
      (queue_families.graphical, 1.0),
      (queue_families.compute, 0.4),
      (queue_families.graphical, 0.2),
    ]
    .iter()
    .cloned(),
  )
  .unwrap();

  let queues = Queues {
    graphical: queues.next().unwrap(),
    compute: queues.next().unwrap(),
    transfers: queues.next().unwrap(),
  };

  (device, queues)
}
