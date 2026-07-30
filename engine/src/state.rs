use anyhow::Result;
use cgmath::prelude::*;
use egui_wgpu::ScreenDescriptor;
use std::{path::Path, sync::Arc};
use wgpu::{Color, util::DeviceExt};

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
    NUM_INSTANCES_PER_ROW,
    asset_manager::AssetManager,
    camera::{Camera, CameraUniform},
    camera_controller::CameraController,
    gpu::{context::GpuContext, resource::ShaderResource},
    gui::EguiRenderer,
    instance::Instance,
    model::Model ,
    renderer::{DrawParams, Renderer},
};

pub struct State {
    pub size: PhysicalSize<u32>,
    pub window: Arc<Window>,

    camera: Camera,
    camera_uniform: CameraUniform,
    camera_resource: ShaderResource<CameraUniform>,
    camera_controller: CameraController,

    pub egui: EguiRenderer,
    pub delay: f32,

    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,

    pub free_mouse: bool,

    obj_model: Arc<Model>,

    renderer: Renderer,
    pub draw_lines: bool,
}

impl State {
    pub async fn new(window: Arc<Window>) -> State {
        let size = window.inner_size();

        let gpu_context: Arc<GpuContext> = Arc::new(GpuContext::new(window.clone()).await);

        let mut asset_manager = AssetManager::new(gpu_context.clone());

        let renderer: Renderer = Renderer::new(gpu_context.clone()).await;

       
        let camera = Camera {
            //    x    y    z
            eye: (0.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: gpu_context.config().width as f32 / gpu_context.config().height as f32,
            fovy: 70.0,
            znear: 0.1,
            zfar: 100.0,
        };
        let mut camera_uniform = CameraUniform::new();

        camera_uniform.set_view_proj(&camera);

        let camera_res =
            ShaderResource::new_uniform(&gpu_context.as_ref().device, "camera", camera_uniform);

     
        let egui: EguiRenderer = EguiRenderer::new(
            &gpu_context.as_ref().device,
            gpu_context.config().format,
            1,
            &window,
        );

        let obj_model = asset_manager
            .load_obj(Path::new("models/cube/cube.obj"))
            .await
            .unwrap();

        let center = (
            window.inner_size().width / 2,
            window.inner_size().height / 2,
        );
        let camera_controller = CameraController::new(0.2, center);

        const SPACE_BETWEEN: f32 = 3.0;
        let instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                    let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);

                    let position = cgmath::Vector3 { x, y: 0.0, z };

                    let rotation = if position.is_zero() {
                        cgmath::Quaternion::from_axis_angle(
                            cgmath::Vector3::unit_z(),
                            cgmath::Deg(0.0),
                        )
                    } else {
                        cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                    };

                    Instance { position, rotation }
                })
            })
            .collect::<Vec<_>>();

        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer =
            gpu_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Instance Buffer"),
                    contents: bytemuck::cast_slice(&instance_data),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                });

        Self {
            size,
            window,
            camera,
            camera_uniform,
            camera_controller,
            egui,
            delay: 0.0,
            instance_buffer,
            instances,
            free_mouse: true,
            obj_model,
            camera_resource: camera_res,
            renderer,
            draw_lines: false
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.renderer.resize(&new_size);

            let center = (new_size.width / 2, new_size.height / 2);

            self.camera_controller.update_screen_center(center);
            self.camera.aspect = new_size.width as f32 / new_size.height as f32;
            self.camera_uniform.set_view_proj(&self.camera);
        }
    }

    pub fn device_input(&mut self, event: &DeviceEvent) -> bool {
        if !self.free_mouse {
            self.camera_controller.process_device_events(event)
        } else {
            false
        }
    }
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        if !self.free_mouse {
            self.camera_controller.process_events(event)
        } else {
            false
        }
    }

    #[profiling::function]
    pub fn update(&mut self, delta_time: f32) {
        self.camera_controller
            .update_camera(&mut self.camera, delta_time);

        let angle = cgmath::Rad(delta_time * 2.0);
        let rotation_delta =
            cgmath::Quaternion::from_axis_angle(cgmath::Vector3::new(0.0, 0.0, 1.0), angle);

        for inst in &mut self.instances {
            inst.rotation = inst.rotation * rotation_delta;
        }

        let instance_data = self
            .instances
            .iter()
            .map(Instance::to_raw)
            .collect::<Vec<_>>();
        self.renderer
            .update_instances(&self.instance_buffer, &instance_data);
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
                camera: &self.camera,
                model: &self.obj_model,
                instance_buffer: &self.instance_buffer,
                instance_count: self.instances.len() as u32,
                draw_lines: self.draw_lines,
                clear_color: Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                },
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
        let fovy = &mut self.camera.fovy;
        let color = &mut self.renderer.clear_color;
        let camera_state = self.camera_controller.get_camera_state(); // owned value, borrow ends here

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
