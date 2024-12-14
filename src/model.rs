use std::ops::Range;

use crate::texture::Texture;

pub trait DrawModel<'a> {
    fn draw_mesh(&mut self, mesh: &'a Mesh);
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        instances: Range<u32>,
    );
}
impl<'a, 'b> DrawModel<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_mesh(&mut self, mesh: &'b Mesh) {
        self.draw_mesh_instanced(mesh, 0..1);
    }

    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        instances: Range<u32>,
    ){
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }
}

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}

 

 pub struct Material {
    pub name: String,
    pub diffuse_texture: Texture,
    pub bind_group: wgpu::BindGroup,
}

pub struct Mesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32, 
    pub material: usize,
}



pub trait Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
}

const ATTRIBUTES: [wgpu::VertexAttribute; 3] =
    wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x3];

impl Vertex for ModelVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}
#[rustfmt::skip]
pub const VERTICES: &[ModelVertex] = &[
    // Changed
   ModelVertex { position: [-0.0868241,   0.49240386, 0.0], normal: [0.0,    0.0,0.0],   tex_coords: [0.4131759,    0.00759614] }, // A
   ModelVertex { position: [-0.49513406,  0.06958647, 0.0], normal: [0.0,    0.0,0.0],   tex_coords: [0.0048659444, 0.43041354] }, // B
   ModelVertex { position: [-0.21918549, -0.44939706, 0.0], normal: [0.0,    0.0,0.0],   tex_coords: [0.28081453,   0.949397]   }, // C
   ModelVertex { position: [ 0.35966998, -0.3473291,  0.0], normal: [0.0,    0.0,0.0],   tex_coords: [0.85967,      0.84732914] },    // D
   ModelVertex { position: [ 0.44147372,  0.2347359,  0.0], normal: [0.0,    0.0,0.0],   tex_coords: [0.9414737,    0.2652641]   },    // E
];

pub const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];
