// Renderer owns a small growable buffer pool, keyed by mesh identity
use std::collections::HashMap;

use crate::{instance::InstanceRaw, model::Model};

#[derive(Default)]
pub(crate) struct InstanceBufferPool {
    buffers: HashMap<*const Model, (wgpu::Buffer, usize)>, // buffer + capacity
}

impl InstanceBufferPool {
    pub(crate) fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        model: &Model,
        data: &[InstanceRaw],
    ) -> &wgpu::Buffer {
        let key = model as *const Model;
        let needed = data.len().max(1);
        let (buf, cap) = self
            .buffers
            .entry(key)
            .or_insert_with(|| (Self::create(device, needed), needed));
        if *cap < needed {
            *buf = Self::create(device, needed);
            *cap = needed;
        }
        queue.write_buffer(buf, 0, bytemuck::cast_slice(data));
        buf
    }

    fn create(device: &wgpu::Device, capacity: usize) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: (capacity * std::mem::size_of::<InstanceRaw>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }
}
