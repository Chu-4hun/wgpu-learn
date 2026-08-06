use std::sync::Arc;

use bevy_ecs::{component::Component, resource::Resource};

use crate::{instance::InstanceRaw, model::Model};

#[derive(Component)]
pub struct Transform {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
}
impl Transform {
    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (cgmath::Matrix4::from_translation(self.position)
                * cgmath::Matrix4::from(self.rotation))
            .into(),
        }
    }
}

#[derive(Component)]
pub struct MeshHandle(pub Arc<Model>);

#[derive(Component)]
pub struct Name(pub String);

#[derive(Component)]
pub struct Spin {
    pub axis: cgmath::Vector3<f32>,
    pub rate: f32,
}

#[derive(Resource)]
pub struct DeltaTime(pub f32);
