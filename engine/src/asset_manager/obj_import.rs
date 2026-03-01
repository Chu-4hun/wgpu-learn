use std::path::Path;

use anyhow::Context;
use futures_lite::io::{BufReader, Cursor};
use wgpu::util::DeviceExt as _;

use crate::{asset_manager::io::Io, gpu::context::GpuContext, model, texture};

pub struct ObjLoader;

impl ObjLoader {
    pub async fn load_texture(
        file_name: &str,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> anyhow::Result<texture::Texture> {
        let data = Io::load_binary(file_name).await?;
        texture::Texture::from_bytes(device, queue, &data, file_name)
    }

    pub async fn load_model(
        path: &Path,
        gpu_context: &GpuContext,
        layout: &wgpu::BindGroupLayout,
    ) -> anyhow::Result<model::Model> {
        let parent_path = path.parent().unwrap_or(Path::new(""));
        let file_name = path
            .file_name()
            .context("NO valid path found")?
            .to_string_lossy()
            .to_string();
        let obj_text =
            Io::load_string(parent_path.join(&file_name).to_string_lossy().as_ref()).await?;
        let obj_cursor = Cursor::new(obj_text);
        let mut obj_reader = BufReader::new(obj_cursor);

        let (models, obj_materials) = tobj::futures::load_obj_buf(
            &mut obj_reader,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
            |p| async move {
                let mat_text = Io::load_string(parent_path.join(p).to_string_lossy().as_ref())
                    .await
                    .unwrap();
                tobj::futures::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text))).await
            },
        )
        .await?;

        let mut materials = Vec::new();
        for m in obj_materials? {
            let diffuse_texture = Self::load_texture(
                parent_path
                    .join(
                        m.diffuse_texture
                            .unwrap_or("default_diffuse_texture".to_owned()),
                    )
                    .to_string_lossy()
                    .as_ref(),
                &gpu_context.device,
                &gpu_context.queue,
            )
            .await?;
            let bind_group = gpu_context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                        },
                    ],
                    label: None,
                });

            materials.push(model::Material {
                name: m.name,
                diffuse_texture,
                bind_group,
            })
        }

        let meshes = models
            .into_iter()
            .map(|m| {
                let vertices = (0..m.mesh.positions.len() / 3) // vertex is 3d so we divide be 3
                    .map(|i| model::ModelVertex {
                        position: [
                            m.mesh.positions[i * 3],
                            m.mesh.positions[i * 3 + 1],
                            m.mesh.positions[i * 3 + 2],
                        ],
                        tex_coords: [m.mesh.texcoords[i * 2], 1.0 - m.mesh.texcoords[i * 2 + 1]],
                        normal: if m.mesh.normals.is_empty() {
                            [0.0, 0.0, 0.0]
                        } else {
                            [
                                m.mesh.normals[i * 3],
                                m.mesh.normals[i * 3 + 1],
                                m.mesh.normals[i * 3 + 2],
                            ]
                        },
                    })
                    .collect::<Vec<_>>();

                let vertex_buffer =
                    gpu_context
                        .device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some(&format!("{:?} Vertex Buffer", file_name)),
                            contents: bytemuck::cast_slice(&vertices),
                            usage: wgpu::BufferUsages::VERTEX,
                        });
                let index_buffer =
                    gpu_context
                        .device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some(&format!("{:?} Index Buffer", file_name)),
                            contents: bytemuck::cast_slice(&m.mesh.indices),
                            usage: wgpu::BufferUsages::INDEX,
                        });

                model::Mesh {
                    name: file_name.to_string(),
                    vertex_buffer,
                    index_buffer,
                    num_elements: m.mesh.indices.len() as u32,
                    material: m.mesh.material_id.unwrap_or(0),
                }
            })
            .collect::<Vec<_>>();

        Ok(model::Model { meshes, materials })
    }
}
