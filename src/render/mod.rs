pub mod executor;
pub mod state;
pub mod wgpu_init;

pub use executor::*;
pub use state::*;
pub use wgpu_init::*;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

pub async fn run(event_loop: EventLoop<()>, window: Window) {
    let mut state = State::new(&window).await;
    let mut last_render_time = instant::Instant::now();
    window.set_visible(true);

    let mut mouse_pressed: bool = false;
    let mut modifiers_pressed: ModifiersState = ModifiersState::default();

    let executor = Executor::new();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared => window.request_redraw(),
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion{ delta, },
                .. // We're not using device_id currently
            } => if mouse_pressed {
                state.camera_controller.process_mouse(delta.0, delta.1)
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                match event {
                    #[cfg(not(target_arch="wasm32"))]
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    WindowEvent::ModifiersChanged(m) => {
                        modifiers_pressed = *m;
                    }
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(key),
                                state: element_state,
                                ..
                            },
                        ..
                    } => match (modifiers_pressed, key) {
                        (ModifiersState::CTRL, VirtualKeyCode::O) => {
                            let dialog = rfd::AsyncFileDialog::new().set_parent(&window).pick_file();

                            executor.execute(async move {
                                let files = dialog.await;

                                let names: Vec<String> = files.into_iter().map(|f| f.file_name()).collect();
                                println!("{:?}", names);
                            });
                        }
                        _ => state
                            .camera_controller
                            .process_keyboard(*key, *element_state, modifiers_pressed),
                    },
                    WindowEvent::MouseWheel { delta, .. } => {
                        state.camera_controller.process_scroll(delta);
                    }
                    WindowEvent::MouseInput {
                        button: MouseButton::Left,
                        state: element_state,
                        ..
                    } => {
                        mouse_pressed = *element_state == ElementState::Pressed;
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                let now = instant::Instant::now();
                let dt = now - last_render_time;
                last_render_time = now;
                state.update(dt);
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => state.resize(state.init.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // We're ignoring timeouts
                    Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                }
            }
            _ => {}
        }
    });
}
