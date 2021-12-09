mod app;
mod camera;
mod vulkano_graphics;

use crate::app::App;
use crate::camera::Camera;
use std::time::Instant;
use vulkano_graphics::VulkanoProgram;

use vulkano::swapchain;
use vulkano::swapchain::AcquireError;
use vulkano::sync;
use vulkano::sync::{FlushError, GpuFuture};
use winit::event::{ElementState, Event, WindowEvent, MouseScrollDelta};
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    let event_loop = EventLoop::new();
    let mut app = App::start(&event_loop);

    let mut program_window_updated = false;

    let mut previous_frame_end = Some(sync::now(app.program.device.clone()).boxed());
    let start_time = Instant::now();

    let mut frames_drawn_since_last_second = 0;
    let mut last_second_tested = 0;

    println!("Starting main loop");

    {
        // window configuration
        let window = app.surface.window();
        window.set_cursor_grab(true).unwrap();
        window.set_cursor_visible(false);
    }

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
        Event::WindowEvent {
            event: WindowEvent::KeyboardInput { input, .. },
            ..
        } => {
            if let Some(key_code) = input.virtual_keycode {
                let is_pressed = match input.state {
                    ElementState::Pressed => true,
                    ElementState::Released => false,
                };

                let control_flow_should_exit = app.handle_keyboard_input(key_code, is_pressed);
                if control_flow_should_exit {
                    *control_flow = ControlFlow::Exit
                }
            }
        }
        Event::WindowEvent {
            event: WindowEvent::CursorMoved { position, .. },
            ..
        } => {
            app.handle_cursor_moved(position);
        }
        Event::WindowEvent {
            event: WindowEvent::MouseWheel {delta, ..},
            ..
        } => {
            if let MouseScrollDelta::LineDelta(_, y) = delta {
                app.handle_mouse_wheel(y);
            }
        }
        Event::WindowEvent {
            event: WindowEvent::CursorLeft { .. },
            ..
        } => app.handle_cursor_left_window(),
        Event::WindowEvent {
            event: WindowEvent::CursorEntered { .. },
            ..
        } => app.handle_cursor_entered_window(),
        Event::RedrawEventsCleared => {
            previous_frame_end.as_mut().unwrap().cleanup_finished();

            if program_window_updated {
                println!("Updating with the window!");
                app.handle_window_update();

                program_window_updated = false;
            }

            let (n_image, suboptimal, acquire_future) = match swapchain::acquire_next_image(
                app.program.swapchain_data.swapchain.clone(),
                None,
            ) {
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
                    app.program.queues.graphical.clone(),
                    app.program.command_buffers.main[n_image].clone(),
                )
                .unwrap()
                .then_swapchain_present(
                    app.program.queues.graphical.clone(),
                    app.program.swapchain_data.swapchain.clone(),
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
            let next_n_image = (n_image + 1) % app.program.image_count;
            app.update(&elapsed, next_n_image);

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
                    previous_frame_end = Some(sync::now(app.program.device.clone()).boxed());
                }
                Err(e) => {
                    println!("Failed to flush future: {:?}", e);
                    previous_frame_end = Some(sync::now(app.program.device.clone()).boxed());
                }
            }
        }
        _ => (),
    });
}
