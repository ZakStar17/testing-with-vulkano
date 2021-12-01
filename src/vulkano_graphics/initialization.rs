use crate::vulkano_graphics::shaders;
use crate::vulkano_graphics::{Buffers, CommandBuffers, QueueFamilies, Queues, Shaders};

use shaders::triangle::{fs, vs};
use shaders::utils::Vertex2d;

use std::sync::Arc;
use vulkano::buffer::{
  BufferAccess, BufferUsage, CpuAccessibleBuffer, CpuBufferPool, DeviceLocalBuffer,
};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage};
use vulkano::device::physical::QueueFamily;
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::device::DeviceExtensions;
use vulkano::device::{Device, Features};
use vulkano::image::view::ImageView;
use vulkano::image::{ImageUsage, SwapchainImage};
use vulkano::instance::Instance;
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::render_pass::{Framebuffer, FramebufferAbstract, RenderPass, Subpass};
use vulkano::swapchain::Surface;
use vulkano::swapchain::Swapchain;
use vulkano::sync;
use vulkano::sync::GpuFuture;
use vulkano::Version;
use winit::window::Window;

pub fn get_instance() -> Arc<Instance> {
  let required_extensions = vulkano_win::required_extensions();

  let layers: Vec<_> = vulkano::instance::layers_list().unwrap().collect();
  let layer_names = layers
    .iter()
    .map(|l| l.name())
    .filter(|&n| n.contains("VK_LAYER_LUNARG"));
  println!(
    "Using layers {:?}",
    layer_names.clone().collect::<Vec<&str>>()
  );

  // substitute last None for layer_names to actually use them
  Instance::new(None, Version::V1_1, &required_extensions, None).unwrap()
}

pub fn get_physical_device<'a>(
  instance: &'a Arc<Instance>,
  device_extensions: &DeviceExtensions,
  surface: &Arc<Surface<Window>>,
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

pub fn get_queue_families<'a>(physical_device: PhysicalDevice<'a>) -> QueueFamilies<'a> {
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

  QueueFamilies {
    graphical: g_queue_family,
    compute: c_queue_family,
    transfers: t_queue_family,
  }
}

pub fn get_logical_device_and_queues(
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

pub fn get_swapchain(
  physical_device: &PhysicalDevice,
  logical_device: &Arc<Device>,
  surface: &Arc<Surface<Window>>,
  queues: &Queues,
) -> (Arc<Swapchain<Window>>, Vec<Arc<SwapchainImage<Window>>>) {
  let caps = surface.capabilities(*physical_device).unwrap();

  let composite_alpha = caps.supported_composite_alpha.iter().next().unwrap();

  let format = caps.supported_formats[0].0;

  let dimensions: [u32; 2] = surface.window().inner_size().into();

  Swapchain::start(logical_device.clone(), surface.clone())
    .num_images(caps.min_image_count)
    .format(format)
    .dimensions(dimensions)
    .usage(ImageUsage::color_attachment())
    .sharing_mode(&queues.graphical)
    .composite_alpha(composite_alpha)
    .build()
    .unwrap()
}

pub fn initialize_and_get_buffers(device: &Arc<Device>, queue_families: &QueueFamilies) -> Buffers {
  // todo: Make everything about buffers better
  let vertex_buffer = CpuAccessibleBuffer::from_iter(
    device.clone(),
    BufferUsage::transfer_source(),
    false,
    [
      Vertex2d {
        position: [-0.5, -0.25],
      },
      Vertex2d {
        position: [0.0, 0.5],
      },
      Vertex2d {
        position: [0.25, -0.1],
      },
    ]
    .iter()
    .cloned(),
  )
  .unwrap();

  let staging_buffer: Arc<DeviceLocalBuffer<[Vertex2d]>> = DeviceLocalBuffer::array(
    device.clone(),
    vertex_buffer.size(),
    BufferUsage::vertex_buffer_transfer_destination(),
    [queue_families.graphical, queue_families.transfers],
  )
  .unwrap();

  let uniform_buffer = CpuBufferPool::<vs::ty::Data>::new(device.clone(), BufferUsage::all());

  Buffers {
    vertex: vertex_buffer,
    staging: staging_buffer,
    uniform: uniform_buffer,
  }
}

pub fn load_shaders(device: &Arc<Device>) -> Shaders {
  Shaders {
    vertex: vs::Shader::load(device.clone()).unwrap(),
    fragment: fs::Shader::load(device.clone()).unwrap(),
  }
}

pub fn get_render_pass(
  device: &Arc<Device>,
  swapchain: &Arc<Swapchain<Window>>,
) -> Arc<RenderPass> {
  Arc::new(
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
    .unwrap(),
  )
}

pub fn load_command_buffers(
  device: &Arc<Device>,
  queue_families: &QueueFamilies,
  buffers: &Buffers,
) -> CommandBuffers {
  let stage_vertices_command_buffer = {
    let mut builder = AutoCommandBufferBuilder::primary(
      device.clone(),
      queue_families.transfers,
      CommandBufferUsage::MultipleSubmit,
    )
    .unwrap();

    builder
      .copy_buffer(buffers.vertex.clone(), buffers.staging.clone())
      .unwrap();
    builder.build().unwrap()
  };

  CommandBuffers {
    stage_vertices: Arc::new(stage_vertices_command_buffer),
  }
}

// Executes a command buffer copying the vertex buffer into the staging buffer,
// then waits for the gpu to finish
pub fn update_staging_buffer(
  device: &Arc<Device>,
  queues: &Queues,
  command_buffers: &CommandBuffers,
) {
  let future = sync::now(device.clone())
    .then_execute(
      queues.graphical.clone(),
      command_buffers.stage_vertices.clone(),
    )
    .unwrap()
    .then_signal_fence_and_flush()
    .unwrap();

  future.wait(None).unwrap();
}

pub fn get_pipeline(
  device: &Arc<Device>,
  shaders: &Shaders,
  render_pass: &Arc<RenderPass>,
  dimensions: &[u32; 2],
) -> Arc<GraphicsPipeline> {
  Arc::new(
    GraphicsPipeline::start()
      .vertex_input_single_buffer::<Vertex2d>()
      .vertex_shader(shaders.vertex.main_entry_point(), ())
      .triangle_list()
      .viewports_dynamic_scissors_irrelevant(1)
      .viewports([Viewport {
        origin: [0.0, 0.0],
        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
        depth_range: 0.0..1.0,
      }])
      .fragment_shader(shaders.fragment.main_entry_point(), ())
      .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
      .build(device.clone())
      .unwrap(),
  )
}

pub fn get_framebuffers(
  images: &[Arc<SwapchainImage<Window>>],
  render_pass: Arc<RenderPass>,
) -> Vec<Arc<dyn FramebufferAbstract>> {
  images
    .iter()
    .map(|image| {
      let view = ImageView::new(image.clone()).unwrap();
      Arc::new(
        Framebuffer::start(render_pass.clone())
          .add(view)
          .unwrap()
          .build()
          .unwrap(),
      ) as Arc<dyn FramebufferAbstract>
    })
    .collect::<Vec<_>>()
}
