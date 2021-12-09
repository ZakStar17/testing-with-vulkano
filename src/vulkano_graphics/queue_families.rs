use vulkano::device::physical::PhysicalDevice;
use vulkano::device::physical::QueueFamily;

pub struct QueueFamilies<'a> {
  pub graphical: QueueFamily<'a>,
  pub compute: QueueFamily<'a>,
  pub transfers: QueueFamily<'a>,
}

impl<'a> QueueFamilies<'a> {
  pub fn init(physical_device: PhysicalDevice<'a>) -> QueueFamilies<'a> {
    let mut g_candidate: Option<QueueFamily> = None; // graphics
    let mut c_candidate: Option<QueueFamily> = None; // compute
    let mut t_candidate: Option<QueueFamily> = None; // transfers
    for queue_family in physical_device.queue_families() {
      if queue_family.supports_graphics() {
        g_candidate = Some(queue_family);
      } else if queue_family.supports_compute() {
        c_candidate = Some(queue_family);
      } else if queue_family.explicitly_supports_transfers() {
        t_candidate = Some(queue_family);
      }
    }

    let g_queue_family = g_candidate.take().unwrap();
    let c_queue_family = c_candidate.take().unwrap();
    let t_queue_family = if let Some(q) = t_candidate {
      q
    } else {
      println!("Warning: Assigned the graphics queue family to the transfers family because there isn't any family excusive for trasfer operations");
      g_queue_family
    };

    println!(
      "Assigned a queue family with {} queues for graphics operations",
      g_queue_family.queues_count()
    );
    println!(
      "Assigned a queue family with {} queues for compute operations",
      c_queue_family.queues_count()
    );
    println!(
      "Assigned a queue family with {} queues for data transfers operations",
      t_queue_family.queues_count()
    );

    QueueFamilies {
      graphical: g_queue_family,
      compute: c_queue_family,
      transfers: t_queue_family,
    }
  }
}
