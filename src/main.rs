mod vulkano_graphics;

use std::time::Instant;
use vulkano_graphics::{shaders, VulkanProgram};

use shaders::triangle::vs;

use std::sync::Arc;
use vulkano::buffer::TypedBufferAccess;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, SubpassContents};
use vulkano::descriptor_set::PersistentDescriptorSet;
use vulkano::device::DeviceExtensions;
use vulkano::pipeline::PipelineBindPoint;
use vulkano::swapchain;
use vulkano::swapchain::AcquireError;
use vulkano::sync;
use vulkano::sync::{FlushError, GpuFuture};
use vulkano_win::VkSurfaceBuild;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

fn main() {
    let event_loop = EventLoop::new();
    let instance = vulkano_graphics::get_instance();

    let surface = WindowBuilder::new()
        .build_vk_surface(&event_loop, instance.clone())
        .unwrap();

    let device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::none()
    };

    let physical_device =
        vulkano_graphics::get_physical_device(&instance, &device_extensions, &surface);

    let queue_families = vulkano_graphics::get_queue_families(physical_device);

    let mut program = VulkanProgram::start(
        device_extensions,
        physical_device,
        &queue_families,
        &surface,
    );

    let mut program_window_updated = false;

    let mut previous_frame_end = Some(sync::now(program.device.clone()).boxed());
    let time_start = Instant::now();

    let colors = [
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 1.0],
    ];

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *control_flow = ControlFlow::Exit;
        }
        Event::WindowEvent {
            event: WindowEvent::Resized(_),
            ..
        } => {
            program_window_updated = true;
        }
        Event::RedrawEventsCleared => {
            instance.api_version();

            previous_frame_end.as_mut().unwrap().cleanup_finished();

            if program_window_updated {
                println!("Updating with the window!");
                program.update_with_window(&surface);

                program_window_updated = false;
            }

            // todo: more all of this out of here
            // todo: make so that the command buffer doesn't get recreated each frame
            let uniform_buffer_subbuffer = {
                let elapsed = time_start.elapsed();

                let uniform_data = vs::ty::Data {
                    color: colors[(elapsed * 4).as_secs() as usize % colors.len()],
                };

                Arc::new(program.buffers.uniform.next(uniform_data).unwrap())
            };

            let layout = program
                .pipeline
                .layout()
                .descriptor_set_layouts()
                .get(0)
                .unwrap();
            let mut set_builder = PersistentDescriptorSet::start(layout.clone());

            set_builder.add_buffer(uniform_buffer_subbuffer).unwrap();

            let set = Arc::new(set_builder.build().unwrap());

            let (image_num, suboptimal, acquire_future) =
                match swapchain::acquire_next_image(program.swapchain.clone(), None) {
                    Ok(r) => r,
                    Err(AcquireError::OutOfDate) => {
                        program_window_updated = true;
                        return;
                    }
                    Err(e) => panic!("Failed to acquire next image: {:?}", e),
                };

            if suboptimal {
                program_window_updated = true;
            }

            let clear_values = vec![[0.0, 0.0, 1.0, 1.0].into()];

            let mut builder = AutoCommandBufferBuilder::primary(
                program.device.clone(),
                program.queues.graphical.family(),
                CommandBufferUsage::OneTimeSubmit,
            )
            .unwrap();

            builder
                .begin_render_pass(
                    program.framebuffers[image_num].clone(),
                    SubpassContents::Inline,
                    clear_values,
                )
                .unwrap()
                .set_viewport(0, [program.viewport.clone()])
                .bind_pipeline_graphics(program.pipeline.clone())
                .bind_descriptor_sets(
                    PipelineBindPoint::Graphics,
                    program.pipeline.layout().clone(),
                    0,
                    set.clone(),
                )
                .bind_vertex_buffers(0, program.buffers.staging.clone())
                .draw(program.buffers.staging.len() as u32, 1, 0, 0)
                .unwrap()
                .end_render_pass()
                .unwrap();

            let command_buffer = builder.build().unwrap();

            let future = previous_frame_end
                .take()
                .unwrap()
                .join(acquire_future)
                .then_execute(program.queues.graphical.clone(), command_buffer)
                .unwrap()
                .then_swapchain_present(
                    program.queues.graphical.clone(),
                    program.swapchain.clone(),
                    image_num,
                )
                .then_signal_fence_and_flush();

            match future {
                Ok(future) => {
                    previous_frame_end = Some(future.boxed());
                }
                Err(FlushError::OutOfDate) => {
                    program_window_updated = true;
                    previous_frame_end = Some(sync::now(program.device.clone()).boxed());
                }
                Err(e) => {
                    println!("Failed to flush future: {:?}", e);
                    previous_frame_end = Some(sync::now(program.device.clone()).boxed());
                }
            }
        }
        _ => (),
    });
}
