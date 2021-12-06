use crate::vulkano_graphics::shaders;
use crate::vulkano_graphics::{
  Buffers, CommandBuffers, DescriptorSets, QueueFamilies, Queues, Shaders,
};
use vulkano::descriptor_set::layout::DescriptorSetLayout;
use vulkano::descriptor_set::PersistentDescriptorSet;

use shaders::triangle::{fs, vs};
use shaders::utils::Vertex2d;

use std::sync::Arc;
use vulkano::buffer::TypedBufferAccess;
use vulkano::buffer::{BufferAccess, BufferUsage, CpuAccessibleBuffer, DeviceLocalBuffer};
use vulkano::command_buffer::PrimaryAutoCommandBuffer;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, SubpassContents};
use vulkano::device::physical::QueueFamily;
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::device::DeviceExtensions;
use vulkano::device::{Device, Features};
use vulkano::image::view::ImageView;
use vulkano::image::{ImageUsage, SwapchainImage};
use vulkano::instance::Instance;
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::PipelineBindPoint;
use vulkano::render_pass::{Framebuffer, FramebufferAbstract, RenderPass, Subpass};
use vulkano::swapchain::Surface;
use vulkano::swapchain::Swapchain;
use vulkano::sync;
use vulkano::sync::GpuFuture;
use vulkano::Version;
use winit::window::Window;

const LIST_AVAILABLE_LAYERS: bool = true;

const ENABLE_VALIDATION_LAYERS: bool = false;

// "VK_LAYER_LUNARG_screenshot", "VK_LAYER_LUNARG_monitor", "VK_LAYER_LUNARG_gfxreconstruct", "VK_LAYER_LUNARG_device_simulation", "VK_LAYER_LUNARG_api_dump"
const VALIDATION_LAYERS: &[&str] = &["VK_LAYER_LUNARG_api_dump"];

pub fn create_instance() -> Arc<Instance> {
  let required_extensions = vulkano_win::required_extensions();

  if LIST_AVAILABLE_LAYERS {
    let layers: Vec<_> = vulkano::instance::layers_list().unwrap().collect();
    let layer_names = layers.iter().map(|l| l.name());
    println!(
      "Using layers {:?}",
      layer_names.clone().collect::<Vec<&str>>()
    );
  }

  if ENABLE_VALIDATION_LAYERS {
    Instance::new(
      None,
      Version::V1_1,
      &required_extensions,
      VALIDATION_LAYERS.iter().cloned(),
    )
    .unwrap()
  } else {
    Instance::new(None, Version::V1_1, &required_extensions, None).unwrap()
  }
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

pub fn create_logical_device_and_queues(
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

pub fn create_swapchain(
  physical_device: &PhysicalDevice,
  logical_device: &Arc<Device>,
  surface: &Arc<Surface<Window>>,
  queues: &Queues,
  image_count: u32
) -> (Arc<Swapchain<Window>>, Vec<Arc<SwapchainImage<Window>>>) {
  let caps = surface.capabilities(*physical_device).unwrap();

  let composite_alpha = caps.supported_composite_alpha.iter().next().unwrap();

  let format = caps.supported_formats[0].0;

  let dimensions: [u32; 2] = surface.window().inner_size().into();

  Swapchain::start(logical_device.clone(), surface.clone())
    .num_images(image_count)
    .format(format)
    .dimensions(dimensions)
    .usage(ImageUsage::color_attachment())
    .sharing_mode(&queues.graphical)
    .composite_alpha(composite_alpha)
    .build()
    .unwrap()
}

pub fn create_and_initalize_buffers(
  device: &Arc<Device>,
  queues: &Queues,
  n_swapchain_images: usize,
) -> Buffers {
  let vertex = CpuAccessibleBuffer::from_iter(
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

  let staging: Arc<DeviceLocalBuffer<[Vertex2d]>> = DeviceLocalBuffer::array(
    device.clone(),
    vertex.size(),
    BufferUsage::vertex_buffer_transfer_destination(),
    [queues.graphical.family(), queues.transfers.family()],
  )
  .unwrap();

  let uniform = create_uniform_buffers(device, n_swapchain_images);

  Buffers {
    vertex,
    staging,
    uniform,
  }
}

fn create_uniform_buffers(
  device: &Arc<Device>,
  buffer_count: usize,
) -> Vec<Arc<CpuAccessibleBuffer<vs::ty::Data>>> {
  let mut buffers = Vec::with_capacity(buffer_count);
  for _ in 0..buffer_count {
    buffers.push(
      CpuAccessibleBuffer::from_data(
        device.clone(),
        BufferUsage::uniform_buffer(),
        false,
        vs::ty::Data {
          color: [0.0, 0.0, 0.0],
        },
      )
      .unwrap(),
    );
  }

  buffers
}

pub fn create_descriptor_sets(
  buffers: &Buffers,
  descriptor_set_layout: &Arc<DescriptorSetLayout>,
) -> DescriptorSets {
  let uniform = buffers
    .uniform
    .iter()
    .map(|buffer| {
      let mut set_builder = PersistentDescriptorSet::start(descriptor_set_layout.clone());

      set_builder.add_buffer(buffer.clone()).unwrap();

      Arc::new(set_builder.build().unwrap())
    })
    .collect();

  DescriptorSets { uniform }
}

pub fn load_shaders(device: &Arc<Device>) -> Shaders {
  Shaders {
    vertex: vs::Shader::load(device.clone()).unwrap(),
    fragment: fs::Shader::load(device.clone()).unwrap(),
  }
}

pub fn create_render_pass(
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

pub fn create_command_buffers(
  device: &Arc<Device>,
  queues: &Queues,
  buffers: &Buffers,
  framebuffers: &Vec<Arc<dyn FramebufferAbstract>>,
  viewport: &Viewport,
  pipeline: &Arc<GraphicsPipeline>,
  descriptor_sets: &DescriptorSets,
) -> CommandBuffers {
  let stage_vertices = {
    let mut builder = AutoCommandBufferBuilder::primary(
      device.clone(),
      queues.transfers.family(),
      CommandBufferUsage::MultipleSubmit,
    )
    .unwrap();

    builder
      .copy_buffer(buffers.vertex.clone(), buffers.staging.clone())
      .unwrap();

    Arc::new(builder.build().unwrap())
  };

  let main = create_main_command_buffers(
    device,
    queues,
    buffers,
    framebuffers,
    viewport,
    pipeline,
    descriptor_sets,
  );

  CommandBuffers {
    stage_vertices,
    main,
  }
}

pub fn create_main_command_buffers(
  device: &Arc<Device>,
  queues: &Queues,
  buffers: &Buffers,
  framebuffers: &Vec<Arc<dyn FramebufferAbstract>>,
  viewport: &Viewport,
  pipeline: &Arc<GraphicsPipeline>,
  descriptor_sets: &DescriptorSets,
) -> Vec<Arc<PrimaryAutoCommandBuffer>> {
  let clear_values = vec![[0.0, 0.0, 1.0, 1.0].into()];

  framebuffers
    .iter()
    .enumerate()
    .map(|(i, framebuffer)| {
      let mut builder = AutoCommandBufferBuilder::primary(
        device.clone(),
        queues.graphical.family(),
        CommandBufferUsage::SimultaneousUse,
      )
      .unwrap();

      builder
        .begin_render_pass(
          framebuffer.clone(),
          SubpassContents::Inline,
          clear_values.clone(),
        )
        .unwrap()
        .set_viewport(0, [viewport.clone()])
        .bind_pipeline_graphics(pipeline.clone())
        .bind_descriptor_sets(
          PipelineBindPoint::Graphics,
          pipeline.layout().clone(),
          0,
          descriptor_sets.uniform[i].clone(),
        )
        .bind_vertex_buffers(0, buffers.staging.clone())
        .draw(buffers.staging.len() as u32, 1, 0, 0)
        .unwrap()
        .end_render_pass()
        .unwrap();

      builder.build().unwrap()
    })
    .map(|c_b| Arc::new(c_b))
    .collect()
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

pub fn create_pipeline(
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

pub fn create_framebuffers(
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
