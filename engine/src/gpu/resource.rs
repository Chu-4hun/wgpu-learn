use wgpu::util::DeviceExt as _;

pub struct ShaderResource<T> {
    buffer: wgpu::Buffer,
    layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    _data: std::marker::PhantomData<T>,
}

impl<T: bytemuck::Pod> ShaderResource<T> {
    pub fn new_uniform(device: &wgpu::Device, label: &str, data: T) -> Self {
        // Для Uniform ставим стандартные флаги сам
        let usage = wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST;
        let binding_type = wgpu::BufferBindingType::Uniform;

        Self::create(
            device,
            label,
            bytemuck::cast_slice(&[data]),
            usage,
            binding_type,
        )
    }

    pub fn new_storage(device: &wgpu::Device, label: &str, data: &[T], read_only: bool) -> Self {
        let usage = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;

        let binding_type = wgpu::BufferBindingType::Storage { read_only };

        Self::create(
            device,
            label,
            bytemuck::cast_slice(data),
            usage,
            binding_type,
        )
    }

    fn create(
        device: &wgpu::Device,
        label: &str,
        contents: &[u8],
        usage: wgpu::BufferUsages,
        buffer_binding_type: wgpu::BufferBindingType,
    ) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{label}_buffer")),
            contents,
            usage,
        });

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: buffer_binding_type,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some(&format!("{label}_layout")),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some(&format!("{label}_bind_group")),
        });

        Self {
            buffer,
            layout,
            bind_group,
            _data: std::marker::PhantomData,
        }
    }
    
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
    
    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }
    
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}
