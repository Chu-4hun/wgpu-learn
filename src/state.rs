use anyhow::{Context, Result};
use cgmath::prelude::*;
use egui_wgpu::ScreenDescriptor;
use std::{path::Path, sync::Arc};
use tracing::debug;
use wgpu::{util::DeviceExt, Buffer, RenderPipeline};

use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

use crate::{
    camera::{Camera, CameraUniform},
    camera_controller::CameraController,
    gui::EguiRenderer,
    instance::Instance,
    model::{DrawModel, Model, ModelVertex, Vertex, INDICES, VERTICES},
    resourses::load_model,
    texture::{self, Texture},
    NUM_INSTANCES_PER_ROW,
};

pub struct State {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: PhysicalSize<u32>,
    pub window: Arc<Window>,
    pub surface_configured: bool,

    pub render_pipeline: RenderPipeline,
    pub line_pipeline: RenderPipeline,

    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,

    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    camera_controller: CameraController,
    pub egui: EguiRenderer,
    pub delay: f32,

    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,
    pub draw_lines: bool,
    pub free_mouse: bool,

    depth_texture: Texture,
    obj_model: Model,
}

impl State {
    pub async fn new(window: Arc<Window>) -> State {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance_desc = wgpu::InstanceDescriptor {
            #[cfg(target_arch = "wasm32")]
            backends: if cfg!(not(target_arch = "wasm32")) {
                wgpu::Backends::PRIMARY
            } else {
                wgpu::Backends::GL
            },
            ..Default::default()
        };
        let instance = wgpu::Instance::new(instance_desc);

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let device_desc = wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::POLYGON_MODE_LINE,
            // WebGL doesn't support all of wgpu's features, so if
            // we're building for the web we'll have to disable some.
            required_limits: if cfg!(target_arch = "wasm32") {
                wgpu::Limits::downlevel_webgl2_defaults()
            } else {
                wgpu::Limits::default()
            },
            memory_hints: wgpu::MemoryHints::default(),
        };
        let (device, queue) = adapter.request_device(&device_desc, None).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an Srgb surface texture. Using a different
        // one will result all the colors coming out darker. If you want to support non
        // Srgb surfaces, you'll need to account for that when drawing to the frame.
        debug!(?surface_caps);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            desired_maximum_frame_latency: 2,
            view_formats: vec![],
        };

        let surface_configured;
        #[cfg(not(target_arch = "wasm32"))]
        {
            surface.configure(&device, &config);
            surface_configured = true;
        }
        #[cfg(target_arch = "wasm32")]
        {
            surface_configured = false;
        }

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shader.wgsl").into()),
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let camera = Camera {
            // position the camera 1 unit up and 2 units back
            // +z is out of the screen
            eye: (0.0, 1.0, 2.0).into(),
            // have it look at the origin
            target: (0.0, 0.0, 0.0).into(),
            // which way is "up"
            up: cgmath::Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };
        let mut camera_uniform = CameraUniform::new();

        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[ModelVertex::desc(), crate::instance::InstanceRaw::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                // 3.
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    // 4.
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });
        let line_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[ModelVertex::desc(), crate::instance::InstanceRaw::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                // 3.
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    // 4.
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: None,
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Line,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,                         // 2.
                mask: !0,                         // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
            cache: None,     // 6.
        });

        let egui: EguiRenderer = EguiRenderer::new(
            &device,       // wgpu Device
            config.format, // TextureFormat
            None,          // this can be None
            1,             // samples
            &window,       // winit Window
        );

        // Convert to a normalized OS-specific PathBuf

        let obj_model = load_model(
            Path::new("models/cube/cube.obj"),
            &device,
            &queue,
            &texture_bind_group_layout,
        )
        .await
        .context("unable to load model models/cube/cube.obj")
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
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
            surface_configured,
            render_pipeline,
            line_pipeline,
            vertex_buffer,
            index_buffer,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_controller,
            egui,
            delay: 0.0,
            instance_buffer,
            instances,
            draw_lines: false,
            free_mouse: true,
            depth_texture,
            obj_model,
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.depth_texture =
                texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");

            self.surface.configure(&self.device, &self.config);

            let center = (new_size.width / 2, new_size.height / 2);
            self.camera_controller.update_screen_center(center);

            self.camera.aspect = new_size.width as f32 / new_size.height as f32;
            self.camera_uniform.update_view_proj(&self.camera);
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        if !self.free_mouse {
            self.camera_controller.process_events(event)
        } else {
            false
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        self.camera_controller
            .update_camera(&mut self.camera, delta_time);
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
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

        self.queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&instance_data),
        );
    }

    pub fn render(&mut self, delta_time: f32) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let clear_color = wgpu::Color {
                r: 0.1,
                g: 0.3,
                b: 0.3,
                a: 1.0,
            };
            let color_attachment = wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear_color),
                    store: wgpu::StoreOp::Store,
                },
            };
            let render_pass_desc = wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(color_attachment)],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            };

            let mut render_pass = encoder.begin_render_pass(&render_pass_desc);
            if self.draw_lines {
                render_pass.set_pipeline(&self.line_pipeline);
            } else {
                render_pass.set_pipeline(&self.render_pipeline);
            }

            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);

            render_pass.draw_model_instanced(
                &self.obj_model,
                0..self.instances.len() as u32,
                &self.camera_bind_group,
            );
        }
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: self.window().scale_factor() as f32,
        };
        
        self.egui.draw(
            &self.device,
            &self.queue,
            &mut encoder,
            &self.window,
            &view,
            screen_descriptor,
            |ui| {
                egui::Window::new("Debug")
                    // .vscroll(true)
                    .title_bar(true)
                    .collapsible(true)
                    .default_open(false)
                    // .resizable(false)
                    .movable(true)
                    // .anchor(egui::Align2::LEFT_TOP, [10.0, 10.0])
                    .show(ui, |ui| {
                        ui.label(format!("FPS: {:.1}", 1.0 / delta_time));
                        ui.label(format!("Frame Time: {:.2}ms", delta_time * 1000.0));
                        if ui
                            .add(egui::Slider::new(&mut self.delay, 0.0..=240.0).text("Max fps"))
                            .changed()
                        {
                            println!("CHANGED {} {}", self.delay, 1.0 / self.delay);
                        }
                        ui.code(
                            egui::RichText::new(format!(
                                "{:#?}",
                                self.camera_controller.get_camera_state()
                            ))
                            .code(),
                        );
                        // if ui.add(egui::Button::new("Click me")).clicked() {
                        //     println!("PRESSED")
                        // }CHANGED

                        // ui.label(format!("Slider {}", egui::special_emojis::OS_LINUX));
                        // ui.add(egui::Slider::new(_, 0..=120).text("age"));
                        ui.end_row();

                        // proto_scene.egui(ui);
                    });
            },
        );
        if self.delay > 0.0 {
            // make frame cap from target fps
            let target_frame_time = 1.0 / self.delay; // Time per frame in secondst screen_descriptor = ScreenDescriptor {
            let delay = (target_frame_time - delta_time) * 1000.0; // Delay needed to achieve target FPSels: [self.config.width, self.config.height],

            std::thread::sleep(std::time::Duration::from_millis(delay as u64));
        };

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
    pub fn window(&self) -> &Window {
        &self.window
    }
}
