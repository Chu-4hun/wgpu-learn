use anyhow::Result;
use egui_wgpu::ScreenDescriptor;
use std::sync::Arc;
use wgpu::Color;

#[derive(Debug)]
pub enum RenderError {
    Lost,
    Outdated,
    Occluded,
    OutOfMemory,
    Timeout,
    Other,
}

use winit::{
    dpi::PhysicalSize,
    event::{DeviceEvent, WindowEvent},
    window::Window,
};

use crate::{
    asset_manager::AssetManager,
    gpu::context::GpuContext,
    gui::EguiRenderer,
    renderer::{DrawBatch, DrawParams, Renderer},
    scene::Scene,
};

pub struct State {
    pub size: PhysicalSize<u32>,
    pub window: Arc<Window>,

    pub egui: EguiRenderer,
    pub delay: f32,

    pub free_mouse: bool,

    renderer: Renderer,
    scene: Scene,

    pub draw_lines: bool,
}

impl State {
    pub async fn new(window: Arc<Window>) -> State {
        let size = window.inner_size();

        let gpu_context: Arc<GpuContext> = Arc::new(GpuContext::new(window.clone()).await);

        let mut asset_manager: AssetManager = AssetManager::new(gpu_context.clone());

        let renderer: Renderer = Renderer::new(gpu_context.clone());

        let scene = Scene::new(&renderer, &mut asset_manager).await;

        let egui: EguiRenderer = EguiRenderer::new(
            &gpu_context.as_ref().device,
            gpu_context.config().format,
            1,
            &window,
        );

        Self {
            size,
            window,
            egui,
            delay: 0.0,
            free_mouse: true,
            renderer,
            draw_lines: false,
            scene,
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.renderer.resize(&new_size);

            let center = (new_size.width / 2, new_size.height / 2);

            self.scene.camera_controller.update_screen_center(center);
            self.scene.camera.aspect = new_size.width as f32 / new_size.height as f32;
            // self.camera_uniform.set_view_proj(&self.camera);
        }
    }

    pub fn device_input(&mut self, event: &DeviceEvent) -> bool {
        if !self.free_mouse {
            self.scene.camera_controller.process_device_events(event)
        } else {
            false
        }
    }
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        if !self.free_mouse {
            self.scene.camera_controller.process_events(event)
        } else {
            false
        }
    }

    #[profiling::function]
    pub fn update(&mut self, delta_time: f32) {
        self.scene.update(delta_time, &self.renderer);
    }

    #[profiling::function]
    pub fn render(&mut self, delta_time: f32) -> Result<(), RenderError> {
        if self.renderer.is_zero_sized() {
            return Ok(());
        }

        let mut frame = self.renderer.begin_frame().unwrap();

        self.renderer.draw(
            &mut frame,
            DrawParams {
                draw_lines: self.draw_lines,
                clear_color: Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                },
                camera: &self.scene.camera,
                batches: &[DrawBatch {
                    model: self.scene.obj_model.as_ref(),
                    instance_buffer: self.scene.cubes.buffer(),
                    instance_count: self.scene.cubes.len(),
                }],
            },
        );

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [self.renderer.width(), self.renderer.height()],
            pixels_per_point: self.window.scale_factor() as f32,
        };
        let gpu_context = self.renderer.gpu_context();
        let device = &gpu_context.device;
        let queue = &gpu_context.queue;

        let (encoder, view) = frame.encoder_and_view();

        let delay = &mut self.delay;
        let fovy = &mut self.scene.camera.fovy;
        let color = &mut self.renderer.clear_color;
        let camera_state = self.scene.camera_controller.get_camera_state(); // owned value, borrow ends here

        self.egui.draw(
            device,
            queue,
            encoder,
            &self.window,
            view,
            screen_descriptor,
            |ctx| {
                egui::Window::new("Debug")
                    .collapsible(true)
                    .default_open(false)
                    .movable(true)
                    .show(ctx, |ui| {
                        ui.label(format!("FPS: {:.1}", 1.0 / delta_time));
                        ui.label(format!("Frame Time: {:.2}ms", delta_time * 1000.0));
                        ui.add(egui::Slider::new(delay, 0.0..=240.0).text("Max fps"));
                        ui.add(egui::Slider::new(fovy, 5.0..=100.0).text("Camera FOV"));
                        ui.color_edit_button_srgba(color);
                        ui.code(egui::RichText::new(format!("{:#?}", camera_state)).code());
                    });
            },
        );

        if self.delay > 0.0 {
            // make frame cap from target fps
            let target_frame_time = 1.0 / self.delay;
            let delay = (target_frame_time - delta_time) * 1000.0;

            std::thread::sleep(std::time::Duration::from_millis(delay as u64));
        };

        self.renderer.end_frame(frame);
        Ok(())
    }

    pub fn window(&self) -> &Window {
        &self.window
    }
}
