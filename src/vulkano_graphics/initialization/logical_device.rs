use std::sync::Arc;
use vulkano::device::physical::{PhysicalDevice, QueueFamily};
use vulkano::device::{Device, DeviceExtensions, Features, Queue};

pub struct Devices {
  pub graphical: Arc<Device>,
  pub compute: Arc<Device>,
  pub transfers: Arc<Device>,
}

pub struct Queues {
  pub graphical: Arc<Queue>,
  pub compute: Arc<Queue>,
  pub transfers: Arc<Queue>,
}

impl Devices {
  pub fn new(
    physical_device: &PhysicalDevice,
    graphical_extensions: &DeviceExtensions,
  ) -> (Self, Queues) {
    let (g_queue_family, c_queue_family, t_queue_family) =
      Self::get_queue_families(physical_device);

    let (graphical, mut g_queues) = Device::new(
      *physical_device,
      &Features::none(),
      &physical_device
        .required_extensions()
        .union(&graphical_extensions),
      [(g_queue_family, 1.0)].iter().cloned(),
    )
    .unwrap();
    let (compute, mut c_queues) = Device::new(
      *physical_device,
      &Features::none(),
      &physical_device.required_extensions(),
      [(c_queue_family, 1.0)].iter().cloned(),
    )
    .unwrap();
    let (transfers, mut t_queues) = Device::new(
      *physical_device,
      &Features::none(),
      &physical_device.required_extensions(),
      [(t_queue_family, 1.0)].iter().cloned(),
    )
    .unwrap();

    (
      Self {
        graphical,
        compute,
        transfers,
      },
      Queues {
        graphical: g_queues.next().unwrap(),
        compute: c_queues.next().unwrap(),
        transfers: t_queues.next().unwrap(),
      },
    )
  }

  fn get_queue_families<'a>(
    physical_device: &'a PhysicalDevice,
  ) -> (QueueFamily<'a>, QueueFamily<'a>, QueueFamily<'a>) {
    let mut g_candidate: Option<QueueFamily> = None; // graphics
    let mut c_candidate: Option<QueueFamily> = None; // compute
    let mut t_candidate: Option<QueueFamily> = None; // transfers
    println!();
    for queue_family in physical_device.queue_families() {
      let queues_count = queue_family.queues_count();
      let supports_graphics = queue_family.supports_graphics();
      let supports_compute = queue_family.supports_compute();
      let supports_transfers = queue_family.explicitly_supports_transfers();
      let supports_binding = queue_family.supports_sparse_binding();

      // print additional debug information
      println!("Found a queue family with {} queues:", queues_count);
      println!("Supports graphics: {};", supports_graphics);
      println!("Supports compute operations: {};", supports_compute);
      println!(
        "Explicitely supports transfer opperations: {};",
        supports_transfers,
      );
      println!("Supports sparse bindings: {};", supports_binding);
      println!();

      if supports_graphics {
        if let Some(old) = g_candidate {
          if old.queues_count() < queues_count {
            // try to find the queue family with most queues
            g_candidate = Some(queue_family);
          }
        } else {
          g_candidate = Some(queue_family);
        };
      } else if supports_compute {
        if let Some(old) = c_candidate {
          if old.queues_count() < queues_count {
            c_candidate = Some(queue_family);
          }
        } else {
          c_candidate = Some(queue_family);
        };
      }
      if supports_transfers {
        if let Some(old) = t_candidate {
          let old_only_transfers = !old.supports_graphics() && !old.supports_compute();
          let cur_only_transfers = !supports_graphics && !supports_compute;
          let old_less_queues = old.queues_count() < queues_count;

          if old_only_transfers {
            if cur_only_transfers && old_less_queues {
              // there is a dedicated queue family with more queues
              t_candidate = Some(queue_family);
            }
          } else {
            if cur_only_transfers || old_less_queues {
              // there is a queue family dedicated for data transfers
              // or they are both not dedicated but the new has more queues that the old
              t_candidate = Some(queue_family);
            }
          }
        } else {
          // take anything that supports data transfers
          t_candidate = Some(queue_family);
        };
      }
    }
    let g_queue_family = if let Some(q) = g_candidate {
      q
    } else {
      // panic if there isn't any candidate
      // this shouldn't happen because physical devices are filtered with this in consideration
      panic!()
    };
    let c_queue_family = if let Some(q) = c_candidate {
      q
    } else {
      panic!()
    };
    let t_queue_family = if let Some(q) = t_candidate {
      q
    } else {
      panic!()
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
    (g_queue_family, c_queue_family, t_queue_family)
  }
}
