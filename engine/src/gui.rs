use egui::{Context, Visuals};
use egui_wgpu::ScreenDescriptor;
use egui_wgpu::{Renderer, RendererOptions};

use egui_winit::winit::event::WindowEvent;
use egui_winit::winit::window::Window;
use egui_winit::State;
use wgpu::{CommandEncoder, Device, Queue, TextureFormat, TextureView};

pub struct EguiRenderer {
    pub context: Context,
    state: State,
    renderer: Renderer,
}

impl EguiRenderer {
    pub fn new(
        device: &Device,
        output_color_format: TextureFormat,
        msaa_samples: u32,
        window: &Window,
    ) -> EguiRenderer {
        let egui_context = Context::default();

        const BORDER_RADIUS: u8 = 2;

        let visuals = Visuals {
            menu_corner_radius: egui::CornerRadius::same(BORDER_RADIUS),
            ..Default::default()
        };

        egui_context.set_visuals(visuals);

        let egui_context_clone = egui_context.clone();
        let viewport_id = egui_context_clone.viewport_id();
        let egui_state = State::new(
            egui_context_clone,
            viewport_id,
            &window,
            None,
            None,
            None,
        );

        // egui_state.set_pixels_per_point(window.scale_factor() as f32);
        let egui_renderer = Renderer::new(
            device,
            output_color_format,
            RendererOptions {
                msaa_samples,
                depth_stencil_format: None,
                dithering: true,
                predictable_texture_filtering: false,
            },
        );

        EguiRenderer {
            context: egui_context,
            state: egui_state,
            renderer: egui_renderer,
        }
    }

    pub fn handle_input(&mut self, window: &Window, event: &WindowEvent) {
        let _ = self.state.on_window_event(window, event);
    }
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        window: &Window,
        window_surface_view: &TextureView,
        screen_descriptor: ScreenDescriptor,
        mut run_ui: impl FnMut(&Context),
    ) {
        // self.state.set_pixels_per_point(window.scale_factor() as f32);
        let raw_input = self.state.take_egui_input(window);
        let full_output = self.context.run_ui(raw_input, |ctx| {
            run_ui(ctx);
        });

        self.state
            .handle_platform_output(window, full_output.platform_output);

        let tris = self
            .context
            .tessellate(full_output.shapes, full_output.pixels_per_point);
        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer
                .update_texture(device, queue, *id, image_delta);
        }
        {
            self.renderer
                .update_buffers(device, queue, encoder, &tris, &screen_descriptor);
            let pass = wgpu::RenderPassDescriptor {
                label: Some("egui main render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: window_surface_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                multiview_mask: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            };

            let mut render_pass = encoder.begin_render_pass(&pass).forget_lifetime();

            self.renderer
                .render(&mut render_pass, &tris, &screen_descriptor);
        }

        for x in &full_output.textures_delta.free {
            self.renderer.free_texture(x)
        }
    }
}