pub struct PipelineBuilder<'a> {
    device: &'a wgpu::Device,
    label: String,
    shader: Option<&'a wgpu::ShaderModule>,
    vertex_entry: String,
    fragment_entry: String,
    vertex_layouts: Vec<wgpu::VertexBufferLayout<'static>>,
    bind_group_layouts: Vec<&'a wgpu::BindGroupLayout>,
    target_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    polygon_mode: wgpu::PolygonMode,
    cull_mode: Option<wgpu::Face>,
    topology: wgpu::PrimitiveTopology,
    topology_strip_index_format: Option<wgpu::IndexFormat>,
}

impl<'a> PipelineBuilder<'a> {
    pub fn new(device: &'a wgpu::Device, format: wgpu::TextureFormat) -> Self {
        Self {
            device,
            label: "Render Pipeline".into(),
            shader: None,
            vertex_entry: "vs_main".into(),
            fragment_entry: "fs_main".into(),
            vertex_layouts: Vec::new(),
            bind_group_layouts: Vec::new(),
            target_format: format,
            depth_format: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            cull_mode: Some(wgpu::Face::Back),
            topology: wgpu::PrimitiveTopology::TriangleList,
            topology_strip_index_format: None,
        }
    }

    pub fn with_label(mut self, label: &str) -> Self {
        self.label = label.into();
        self
    }

    pub fn with_shader(mut self, shader: &'a wgpu::ShaderModule) -> Self {
        self.shader = Some(shader);
        self
    }

    pub fn add_layout(mut self, layout: &'a wgpu::BindGroupLayout) -> Self {
        self.bind_group_layouts.push(layout);
        self
    }

    pub fn add_vertex_layout(mut self, layout: wgpu::VertexBufferLayout<'static>) -> Self {
        self.vertex_layouts.push(layout);
        self
    }

    pub fn with_depth(mut self, format: wgpu::TextureFormat) -> Self {
        self.depth_format = Some(format);
        self
    }
    /// Позволяет изменить названия входных точек шейдера
    pub fn with_entry_points(mut self, vertex: &str, fragment: &str) -> Self {
        self.vertex_entry = vertex.into();
        self.fragment_entry = fragment.into();
        self
    }

    /// Позволяет рисовать не только треугольники, но и линии или точки
    /// Полезно для реализации твоего флага draw_lines
    pub fn with_polygon_mode(mut self, plygon_mode: wgpu::PolygonMode) -> Self {
        self.polygon_mode = plygon_mode;
        self
    }

    /// Управление отсечением граней (Back, Front или None)
    pub fn with_culling(mut self, cull_mode: Option<wgpu::Face>) -> Self {
        self.cull_mode = cull_mode;
        self
    }

    /// In ANY strip format use with_topology_strip_index_format
    /// ```
    /// PointList,
    /// LineList,
    /// LineStrip, <- use with_topology_strip_index_format()
    /// TriangleList,
    /// TriangleStrip, <- use with_topology_strip_index_format()
    /// ```
    pub fn with_topology(mut self, topology: wgpu::PrimitiveTopology) -> Self {
        self.topology = topology;
        self
    }

    /// REQUIRED WITH with_topology wgpu::PrimitiveTopology any STRIP format
    pub fn with_topology_strip_index_format(mut self, index_format: wgpu::IndexFormat) -> Self {
        self.topology_strip_index_format = Some(index_format);
        self
    }

    pub fn build(self) -> wgpu::RenderPipeline {
        let shader = self.shader.expect("Shader module is required for pipeline");

        let layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(&format!("{} Layout", self.label)),
                bind_group_layouts: &self.bind_group_layouts,
                push_constant_ranges: &[],
            });

        self.device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(&self.label),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: shader,
                    entry_point: Some(&self.vertex_entry),
                    buffers: &self.vertex_layouts,
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: shader,
                    entry_point: Some(&self.fragment_entry),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: self.target_format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: self.topology,
                    strip_index_format: self.topology_strip_index_format,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: self.cull_mode,
                    polygon_mode: self.polygon_mode,
                    ..Default::default()
                },
                depth_stencil: self.depth_format.map(|format| wgpu::DepthStencilState {
                    format,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            })
    }
}
