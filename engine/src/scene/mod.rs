use std::{collections::HashMap, path::Path, sync::Arc};

use bevy_ecs::{schedule::Schedule, world::World};

use cgmath::Rotation3;

use crate::{
    asset_manager::AssetManager,
    camera::Camera,
    camera_controller::CameraController,
    components::{self, MeshHandle, Spin, Transform},
    instance::InstanceRaw,
    model::Model,
    renderer::{DrawBatch, Renderer},
};
pub struct Scene {
    pub camera: Camera,
    pub camera_controller: CameraController,
    world: World,
    _update_schedule: Schedule,
}

impl Scene {
    pub async fn new(renderer: &Renderer, asset_manager: &mut AssetManager) -> Self {
        let mut world = World::new();
        let camera = Camera {
            eye: (0.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: renderer.width() as f32 / renderer.height() as f32,
            fovy: 70.0,
            znear: 0.1,
            zfar: 100.0,
        };
        let center = (renderer.width() / 2, renderer.height() / 2);
        let camera_controller = CameraController::new(0.2, center);

        // const SPACE_BETWEEN: f32 = 3.0;
        // let instances = (0..NUM_INSTANCES_PER_ROW)
        //     .flat_map(|z| {
        //         (0..NUM_INSTANCES_PER_ROW).map(move |x| {
        //             let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
        //             let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
        //             let position = cgmath::Vector3 { x, y: 0.0, z };
        //             let rotation = if position.is_zero() {
        //                 cgmath::Quaternion::from_axis_angle(
        //                     cgmath::Vector3::unit_z(),
        //                     cgmath::Deg(0.0),
        //                 )
        // //             } else {
        //                 cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
        //             };
        //             Instance { position, rotation }
        //         })
        //     })
        //     .collect::<Vec<_>>();

        // let cubes = InstanceSet::new(renderer.device(), instances);

        let obj_model = asset_manager
            .load_obj(Path::new("models/cube/cube.obj"))
            .await
            .unwrap();

        for i in 0..10 {
            world.spawn((
                components::Transform {
                    position: cgmath::Vector3 {
                        x: 1.0 * i as f32,
                        y: 1.0 * i as f32,
                        z: 1.0,
                    },
                    rotation: cgmath::Quaternion::from_axis_angle(
                        cgmath::Vector3::unit_z(),
                        cgmath::Deg(0.0),
                    ),
                },
                MeshHandle(obj_model.clone()),
                components::Name("CUBE".into()),
                components::Spin {
                    axis: cgmath::Vector3::unit_z(),
                    rate: 3.0,
                },
            ));
        }

        Self {
            camera,
            camera_controller,

            world,
            _update_schedule: Schedule::default(),
        }
    }

    pub fn draw_batches(&mut self) -> Vec<DrawBatch> {
        let mut grouped: HashMap<*const Model, (Arc<Model>, Vec<InstanceRaw>)> = HashMap::new();

        let mut query = self.world.query::<(&Transform, &MeshHandle)>();
        for (transform, mesh) in query.iter(&self.world) {
            let key = Arc::as_ptr(&mesh.0);
            grouped
                .entry(key)
                .or_insert_with(|| (mesh.0.clone(), Vec::new()))
                .1
                .push(transform.to_raw());
        }

        grouped
            .into_values()
            .map(|(model, instances)| DrawBatch {
                model: model.clone(),
                instances,
            })
            .collect()
    }

    pub fn update(&mut self, dt: f32) {
        self.camera_controller.update_camera(&mut self.camera, dt);

        let mut query = self.world.query::<(&mut Transform, &Spin)>();
        for (mut transform, spin) in query.iter_mut(&mut self.world) {
            let delta = cgmath::Quaternion::from_axis_angle(spin.axis, cgmath::Rad(spin.rate * dt));
            transform.rotation = transform.rotation * delta;
        }
    }
}
