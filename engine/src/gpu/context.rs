pub use std::sync::Arc;

use wgpu::ExperimentalFeatures;
use winit::{dpi::PhysicalSize, window::Window};

pub struct GpuContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
    pub adapter: wgpu::Adapter,
}

impl GpuContext {
    pub async fn new(window: Arc<Window>) -> Self {
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
        let instance = wgpu::Instance::new(&instance_desc);

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
            trace: wgpu::Trace::Off,
            experimental_features: ExperimentalFeatures::default(),
        };
        let (device, queue) = adapter.request_device(&device_desc).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

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

        #[cfg(not(target_arch = "wasm32"))]
        {
            surface.configure(&device, &config);
        }

        Self {
            device,
            queue,
            surface,
            config,
            adapter,
        }
    }
    pub fn resize(&mut self, new_size: &PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
}
