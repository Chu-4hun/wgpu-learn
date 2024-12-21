pub mod camera;
pub mod camera_controller;
pub mod gui;
pub mod instance;
pub mod model;
pub mod resourses;
pub mod state;
pub mod texture;

use std::{sync::Arc, time::Instant};

use anyhow::Result;
use state::State;
use tracing::{info, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, Position},
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    keyboard::{KeyCode, PhysicalKey},
    window::{CursorGrabMode, WindowAttributes, WindowId},
};

const NUM_INSTANCES_PER_ROW: u32 = 10;

enum UserEvent {
    StateReady(State),
}

struct App {
    state: Option<State>,
    event_loop_proxy: EventLoopProxy<UserEvent>,

    frame_time: Instant,
    delta_time: f32,
    // rd: RenderDoc<V141>
}

impl App {
    fn new(event_loop: &EventLoop<UserEvent>) -> Self {
        // let rd: RenderDoc<V141> = RenderDoc::new().expect("Unable to connect");
        Self {
            state: None,
            event_loop_proxy: event_loop.create_proxy(),
            frame_time: Instant::now(),
            delta_time: 0.0,
            // rd
        }
    }
}

impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        tracing::info!("Resumed");

        let mut window_attrs = WindowAttributes::default();
        window_attrs.title = "Chu engine".to_string();

        let window = event_loop
            .create_window(window_attrs)
            .expect("Couldn't create window.");

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::Element;
            use winit::{dpi::PhysicalSize, platform::web::WindowExtWebSys};

            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dst = doc.get_element_by_id("wasm-example")?;
                    let canvas = Element::from(window.canvas()?);
                    dst.append_child(&canvas).ok()?;
                    Some(())
                })
                .expect("Couldn't append canvas to document body.");

            // Winit prevents sizing with CSS, so we have to set
            // the size manually when on web.
            let _ = window.request_inner_size(PhysicalSize::new(450, 400));

            let state_future = State::new(Arc::new(window));
            let event_loop_proxy = self.event_loop_proxy.clone();
            let future = async move {
                let state = state_future.await;
                assert!(event_loop_proxy
                    .send_event(UserEvent::StateReady(state))
                    .is_ok());
            };
            wasm_bindgen_futures::spawn_local(future)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let state = pollster::block_on(State::new(Arc::new(window)));

            assert!(self
                .event_loop_proxy
                .send_event(UserEvent::StateReady(state))
                .is_ok());
        }
    }

    fn user_event(&mut self, _: &ActiveEventLoop, event: UserEvent) {
        let UserEvent::StateReady(state) = event;
        self.state = Some(state);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(ref mut state) = self.state else {
            return;
        };

        if window_id != state.window.id() {
            return;
        }

        if state.input(&event) {
            return;
        }

        if !state.free_mouse {
            let size = state.window.inner_size();
            state
                .window
                .set_cursor_position(Position::Physical(PhysicalPosition::new(
                    (size.width / 2) as i32,
                    (size.height / 2) as i32,
                )))
                .unwrap();
        }
        match event {
            WindowEvent::CloseRequested => {
                tracing::info!("Exited!");
                event_loop.exit()
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        ..
                    },
                ..
            } => {
                state.window.set_cursor_visible(!state.free_mouse);
                state
                    .window
                    .set_cursor_grab(winit::window::CursorGrabMode::None)
                    .unwrap();
                state.free_mouse = !state.free_mouse;
            }

            WindowEvent::Focused(focus) => {
                info!("focus {focus}");
                if focus {
                    state
                        .window
                        .set_cursor_grab(CursorGrabMode::Confined)
                        .or_else(|_| state.window.set_cursor_grab(CursorGrabMode::Locked))
                        .unwrap();
                }
                state.window.set_cursor_visible(state.free_mouse);
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::F1),
                        ..
                    },
                ..
            } => {
                state.draw_lines = !state.draw_lines;
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::F2),
                        ..
                    },
                ..
            } => {
                // self.rd.trigger_capture();
            }
            WindowEvent::Resized(physical_size) => {
                state.surface_configured = true;
                state.resize(physical_size);
                // tracing::info!("physical_size: {physical_size:?}");
            }
            WindowEvent::RedrawRequested => {
                let start = Instant::now();
                let elapsed = (start - self.frame_time).as_secs_f32();

                if !state.surface_configured {
                    return;
                }
                state.update(elapsed);
                match state.render(elapsed) {
                    Ok(()) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        state.resize(state.size);
                    }
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        tracing::error!("OutOfMemory");
                        event_loop.exit();
                    }

                    // This happens when the frame takes too long to present
                    Err(wgpu::SurfaceError::Timeout) => {
                        tracing::warn!("Surface timeout");
                    }
                }

                self.frame_time = start;
                self.delta_time = elapsed;
            }
            _ => {}
        }
        state.egui.handle_input(&state.window, &event);
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        if let Some(ref state) = self.state {
            state.window.request_redraw();
        };
    }
}

pub fn run() -> Result<()> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(Level::INFO.into())
        .from_env_lossy()
        .add_directive("wgpu_core::device::resource=warn".parse()?);

    let subscriber = tracing_subscriber::registry().with(env_filter);
    #[cfg(target_arch = "wasm32")]
    {
        use tracing_wasm::{WASMLayer, WASMLayerConfig};

        console_error_panic_hook::set_once();
        let wasm_layer = WASMLayer::new(WASMLayerConfig::default());

        subscriber.with(wasm_layer).init();
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_line_number(true)
            .with_level(true);
        subscriber.with(fmt_layer).init();
    }
    
    
    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;
    let mut app = App::new(&event_loop);

    event_loop.run_app(&mut app)?;
    Ok(())
}
