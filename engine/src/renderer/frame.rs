pub struct Frame {
    pub surface_texture: wgpu::SurfaceTexture,
    pub view: wgpu::TextureView,
    pub encoder: wgpu::CommandEncoder,
}

impl Frame {
    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
    pub fn encoder_mut(&mut self) -> &mut wgpu::CommandEncoder {
        &mut self.encoder
    }
    pub fn encoder_and_view(&mut self) -> (&mut wgpu::CommandEncoder, &wgpu::TextureView) {
        (&mut self.encoder, &self.view)
    }
}
