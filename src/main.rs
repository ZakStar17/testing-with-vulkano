mod vulkano_graphics;

use std::time::Instant;
use vulkano_graphics::VulkanoProgram;

use vulkano::device::DeviceExtensions;
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
    let instance = vulkano_graphics::create_instance();

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

    let mut program = VulkanoProgram::start(
        device_extensions,
        physical_device,
        &queue_families,
        &surface,
    );

    let mut program_window_updated = false;

    let mut previous_frame_end = Some(sync::now(program.device.clone()).boxed());
    let start_time = Instant::now();

    let colors = [
        [0.0f32, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 1.0],
    ];

    let mut frames_drawn_since_last_second = 0;
    let mut last_second_tested = 0;

    println!("Starting main loop");

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
            println!("Window was resized");
            program_window_updated = true;
        }
        Event::RedrawEventsCleared => {
            previous_frame_end.as_mut().unwrap().cleanup_finished();

            if program_window_updated {
                println!("Updating with the window!");
                program.update_with_window(&surface);

                program_window_updated = false;
            }

            let (n_image, suboptimal, acquire_future) =
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

            let future = previous_frame_end
                .take()
                .unwrap()
                .join(acquire_future)
                .then_execute(
                    program.queues.graphical.clone(),
                    program.command_buffers.main[n_image].clone(),
                )
                .unwrap()
                .then_swapchain_present(
                    program.queues.graphical.clone(),
                    program.swapchain.clone(),
                    n_image,
                )
                .then_signal_fence_and_flush();

            // Update while the gpu executes the frame
            let elapsed = start_time.elapsed();
            let elapsed_seconds = elapsed.as_secs();

            frames_drawn_since_last_second += 1;
            if elapsed_seconds != last_second_tested {
                last_second_tested = elapsed_seconds;

                println!("fps: {}", frames_drawn_since_last_second);
                frames_drawn_since_last_second = 0;
            }

            // Update the buffers on the next frame
            let next_n_image = (n_image + 1) % program.image_count;
            program.update(&elapsed, next_n_image, &colors);

            match future {
                Ok(future) => {
                    // This is a bad approach, but I need a way to unblock the buffers (for example the uniform)
                    // so that the cpu can copy data to it. If the gpu is to fast, it will use all the command buffers (and block the buffers)

                    // todo: instead of waiting for the current frame, wait for the next that was submitted on the cycle, 
                    // so that one frame is getting writen and all the others are getting executen (minimal waiting time)

                    // todo: find out how to implement post processing with this
                    future.wait(None).unwrap();
                    previous_frame_end = Some(future.boxed());
                }
                Err(FlushError::OutOfDate) => {
                    println!("Out of date");
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
