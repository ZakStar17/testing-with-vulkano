use crate::vulkano_graphics::command_buffers;
use crate::vulkano_graphics::models;
use crate::vulkano_graphics::shaders;
use crate::vulkano_graphics::Queues;

use shaders::simple_cube::vs;
use shaders::vertex_data::Vertex3d;

use std::sync::Arc;
use vulkano::buffer::{BufferAccess, BufferUsage, CpuAccessibleBuffer, DeviceLocalBuffer};
use vulkano::descriptor_set::layout::DescriptorSetLayout;
use vulkano::descriptor_set::PersistentDescriptorSet;
use vulkano::device::Device;

type Uniforms = Vec<(
  Arc<CpuAccessibleBuffer<vs::ty::Data>>,
  Arc<PersistentDescriptorSet>,
)>;

pub struct Buffers {
  pub uniforms: Uniforms,
  pub models: ModelBuffers,
}

pub struct CubeModelBuffer {
  _vertex: Arc<CpuAccessibleBuffer<[Vertex3d]>>,
  pub vertex_staging: Arc<DeviceLocalBuffer<[Vertex3d]>>,
  _index: Arc<CpuAccessibleBuffer<[u16]>>,
  pub index_staging: Arc<DeviceLocalBuffer<[u16]>>,
}

pub struct ModelBuffers {
  pub cube: CubeModelBuffer,
}

impl Buffers {
  pub fn init(
    device: Arc<Device>,
    queues: &Queues,
    n_swapchain_images: usize,
    descriptor_set_layout: &Arc<DescriptorSetLayout>,
  ) -> Self {
    Self {
      uniforms: Self::create_uniforms(device.clone(), n_swapchain_images, descriptor_set_layout),
      models: ModelBuffers::init(device, queues),
    }
  }

  fn create_uniforms(
    device: Arc<Device>,
    buffer_count: usize,
    descriptor_set_layout: &Arc<DescriptorSetLayout>,
  ) -> Uniforms {
    let mut uniforms = Vec::with_capacity(buffer_count);
    for _ in 0..buffer_count {
      let buffer = CpuAccessibleBuffer::from_data(
        device.clone(),
        BufferUsage::uniform_buffer(),
        false,
        vs::ty::Data {
          matrix: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
          ],
        },
      )
      .unwrap();

      let descriptor_set = {
        let mut set_builder = PersistentDescriptorSet::start(descriptor_set_layout.clone());

        set_builder.add_buffer(buffer.clone()).unwrap();

        set_builder.build().unwrap()
      };

      uniforms.push((buffer, descriptor_set));
    }

    uniforms
  }
}

impl ModelBuffers {
  fn init(device: Arc<Device>, queues: &Queues) -> Self {
    Self {
      cube: CubeModelBuffer::init(device, queues),
    }
  }
}

impl CubeModelBuffer {
  fn init(device: Arc<Device>, queues: &Queues) -> Self {
    let vertices = models::cube::VERTICES;
    let vertex = CpuAccessibleBuffer::from_iter(
      device.clone(),
      BufferUsage::transfer_source(),
      false,
      vertices.iter().cloned(),
    )
    .unwrap();
    let vertex_staging: Arc<DeviceLocalBuffer<[Vertex3d]>> = DeviceLocalBuffer::array(
      device.clone(),
      vertex.size(),
      BufferUsage::vertex_buffer_transfer_destination(),
      [queues.graphical.family(), queues.transfers.family()],
    )
    .unwrap();
    let indices = models::cube::INDICES;
    let index = CpuAccessibleBuffer::from_iter(
      device.clone(),
      BufferUsage::transfer_source(),
      false,
      indices.iter().cloned(),
    )
    .unwrap();
    let index_staging: Arc<DeviceLocalBuffer<[u16]>> = DeviceLocalBuffer::array(
      device.clone(),
      index.size(),
      BufferUsage::index_buffer_transfer_destination(),
      [queues.graphical.family(), queues.transfers.family()],
    )
    .unwrap();
    command_buffers::transfer_buffer_contents_on_gpu(
      device.clone(),
      queues,
      &vertex,
      &vertex_staging,
    );
    command_buffers::transfer_buffer_contents_on_gpu(device, queues, &index, &index_staging);

    Self {
      _vertex: vertex,
      vertex_staging,
      _index: index,
      index_staging,
    }
  }
}
