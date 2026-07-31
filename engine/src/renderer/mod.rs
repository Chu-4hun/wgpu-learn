pub mod camera_bind;
pub mod frame;

use std::sync::Arc;

use egui::{Color32, Rgba};
use wgpu::{Buffer, Color, RenderPipeline, util::DeviceExt};
use winit::dpi::PhysicalSize;

use crate::{
    gpu::{context::GpuContext, pipeline::PipelineBuilder},
    instance::InstanceRaw,
    model::{DrawModel, INDICES, ModelVertex, Vertex},
    renderer::{camera_bind::CameraBinding, frame::Frame},
    scene::Scene,
    texture::Texture,
};

#[derive(Debug)]
pub enum RenderError {
    Lost,
    Outdated,
    Occluded,
    OutOfMemory,
    Timeout,
    Other,
}
pub struct Renderer {
    gpu_context: Arc<GpuContext>,
    depth_texture: Texture,

    render_pipeline: RenderPipeline,
    line_pipeline: RenderPipeline,
    index_buffer: Buffer,
    texture_layout: wgpu::BindGroupLayout,
    camera_binding: CameraBinding,

    pub clear_color: Color32,
}
pub struct DrawParams<'a> {
    pub scene: &'a Scene,
    pub instance_count: u32,
    pub clear_color: wgpu::Color,
    pub draw_lines: bool,
}

impl Renderer {
    pub fn new(gpu_context: Arc<GpuContext>) -> Self {
        let shader =
            gpu_context
                .as_ref()
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Shader"),
                    source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/shader.wgsl").into()),
                });

        let camera_binding = CameraBinding::new(&gpu_context.device);

        let depth_texture = Texture::create_depth_texture(&gpu_context, "depth_texture");

        let texture_layout: wgpu::BindGroupLayout =
            Texture::create_bind_group_layout(&gpu_context.device);

        let render_pipeline =
            PipelineBuilder::new(&gpu_context.device, gpu_context.config().format)
                .with_label("Render Pipeline")
                .with_shader(&shader)
                .with_entry_points("vs_main", "fs_main")
                .add_layout(&texture_layout)
                .add_layout(camera_binding.layout()) // <- same call as before, just via the binding
                .add_vertex_layout(Some(ModelVertex::desc()))
                .add_vertex_layout(Some(InstanceRaw::desc()))
                .with_depth(Texture::DEPTH_FORMAT)
                .build();

        let line_pipeline =
            PipelineBuilder::new(&gpu_context.as_ref().device, gpu_context.config().format)
                .with_label("Wireframe Render Pipeline")
                .with_shader(&shader)
                .with_entry_points("vs_main", "fs_main")
                .add_layout(&texture_layout)
                .add_layout(camera_binding.layout())
                .add_vertex_layout(Some(ModelVertex::desc()))
                .add_vertex_layout(Some(InstanceRaw::desc()))
                .with_polygon_mode(wgpu::PolygonMode::Line)
                .with_depth(Texture::DEPTH_FORMAT)
                .build();

        let index_buffer =
            gpu_context
                .as_ref()
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(INDICES),
                    usage: wgpu::BufferUsages::INDEX,
                });

        Self {
            gpu_context,
            depth_texture,
            render_pipeline,
            line_pipeline,
            index_buffer,
            camera_binding,
            texture_layout,
            clear_color: Color32::from_rgb(0, 50, 20),
        }
    }

    // fn begin_frame(&mut self) -> Result<Frame, RenderError> { todo!() }
    // fn draw(&mut self, frame: &mut Frame, scene: &Scene, camera_bind_group: &wgpu::BindGroup) { todo!() }
    // fn end_frame(&mut self, frame: Frame) { todo!() }
    // pub fn resize(&self, new_size: &PhysicalSize<u32>)

    pub fn begin_frame(&mut self) -> Result<Frame, RenderError> {
        let output = match self.gpu_context.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t)
            | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            wgpu::CurrentSurfaceTexture::Timeout => return Err(RenderError::Timeout),
            wgpu::CurrentSurfaceTexture::Occluded => return Err(RenderError::Occluded), // treat as "skip, not an error"
            wgpu::CurrentSurfaceTexture::Outdated => return Err(RenderError::Outdated),
            wgpu::CurrentSurfaceTexture::Lost => return Err(RenderError::Lost),
            wgpu::CurrentSurfaceTexture::Validation => return Err(RenderError::Other),
        };
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let encoder =
            self.gpu_context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Frame Encoder"),
                });
        Ok(Frame {
            surface_texture: output,
            view,
            encoder,
        })
    }

    pub fn draw(&mut self, frame: &mut Frame, params: DrawParams) {
        self.camera_binding
            .sync(&self.gpu_context.queue, &params.scene.camera);
        let clear_color = Rgba::from(self.clear_color).to_rgba_unmultiplied();
        let render_pass_desc = wgpu::RenderPassDescriptor {
            label: Some("Scene Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &frame.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(Color {
                        r: clear_color[0] as f64,
                        g: clear_color[1] as f64,
                        b: clear_color[2] as f64,
                        a: clear_color[3] as f64,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            multiview_mask: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        };

        let mut pass = frame.encoder.begin_render_pass(&render_pass_desc);
        pass.set_pipeline(if params.draw_lines {
            &self.line_pipeline
        } else {
            &self.render_pipeline
        });
        pass.set_vertex_buffer(1, params.scene.cubes.buffer().slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        pass.set_bind_group(1, self.camera_binding.bind_group(), &[]);
        pass.draw_model_instanced(
            &params.scene.obj_model,
            0..params.instance_count,
            self.camera_binding.bind_group(),
        );
    }

    pub fn end_frame(&mut self, frame: Frame) {
        self.gpu_context
            .queue
            .submit(std::iter::once(frame.encoder.finish()));
        self.gpu_context.queue.present(frame.surface_texture);
    }

    pub fn resize(&mut self, new_size: &PhysicalSize<u32>) {
        self.gpu_context.resize(new_size);
        self.depth_texture = Texture::create_depth_texture(&self.gpu_context, "depth_texture");
    }

    pub(crate) fn update_instances(&self, instance_buffer: &wgpu::Buffer, data: &[InstanceRaw]) {
        self.gpu_context
            .queue
            .write_buffer(instance_buffer, 0, bytemuck::cast_slice(data));
    }

    pub fn is_zero_sized(&self) -> bool {
        let cfg = self.gpu_context.config();
        cfg.width == 0 || cfg.height == 0
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.gpu_context.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.gpu_context.queue
    }
    pub fn gpu_context(&self) -> Arc<GpuContext> {
        self.gpu_context.clone()
    }

    pub fn width(&self) -> u32 {
        self.gpu_context.config().width
    }

    pub fn height(&self) -> u32 {
        self.gpu_context.config().height
    }
}
