use crate::{camera::{Camera, CameraUniform}, gpu::resource::ShaderResource};

pub struct CameraBinding {
    uniform: CameraUniform,
    resource: ShaderResource<CameraUniform>,
}

impl CameraBinding {
    pub fn new(device: &wgpu::Device) -> Self {
        let uniform = CameraUniform::new();
        let resource = ShaderResource::new_uniform(device, "camera", uniform);
        Self { uniform, resource }
    }

    pub fn layout(&self) -> &wgpu::BindGroupLayout { self.resource.layout() }
    pub fn bind_group(&self) -> &wgpu::BindGroup { self.resource.bind_group() }

    // called once per frame, fed the Scene's camera
    pub fn sync(&mut self, queue: &wgpu::Queue, camera: &Camera) {
        self.uniform.set_view_proj(camera);
        queue.write_buffer(self.resource.buffer(), 0, bytemuck::cast_slice(&[self.uniform]));
    }
}