use std::{path::Path, sync::Arc};

use cgmath::prelude::*;

use crate::{
    NUM_INSTANCES_PER_ROW,
    asset_manager::AssetManager,
    camera::Camera,
    camera_controller::CameraController,
    instance::{Instance, InstanceSet},
    model::Model,
    renderer::Renderer,
};
pub struct Scene {
    pub camera: Camera,
    pub camera_controller: CameraController,
    pub cubes: InstanceSet,
    pub obj_model: Arc<Model>,
}

impl Scene {
    pub async fn new(renderer: &Renderer, asset_manager: &mut AssetManager) -> Self {
        let camera = Camera {
            eye: (0.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: renderer.width() as f32 / renderer.height() as f32,
            fovy: 70.0,
            znear: 0.1,
            zfar: 100.0,
        };
        let center = (
            renderer.width() / 2,
            renderer.height() / 2,
        );
        let camera_controller = CameraController::new(0.2, center);


        const SPACE_BETWEEN: f32 = 3.0;
        let instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                    let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                    let position = cgmath::Vector3 { x, y: 0.0, z };
                    let rotation = if position.is_zero() {
                        cgmath::Quaternion::from_axis_angle(
                            cgmath::Vector3::unit_z(),
                            cgmath::Deg(0.0),
                        )
                    } else {
                        cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                    };
                    Instance { position, rotation }
                })
            })
            .collect::<Vec<_>>();

        let cubes = InstanceSet::new(renderer.device(), instances);

        let obj_model = asset_manager
            .load_obj(Path::new("models/cube/cube.obj"))
            .await
            .unwrap();

        Self {
            camera,
            camera_controller,
            cubes,
            obj_model,
        }
    }

    pub fn update(&mut self, dt: f32, renderer: &Renderer) {
        self.camera_controller.update_camera(&mut self.camera, dt);

        let angle = cgmath::Rad(dt * 2.0);
        let rotation_delta = cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), angle);
        for inst in self.cubes.iter_mut() {
            inst.rotation = inst.rotation * rotation_delta;
        }
        self.cubes.sync(renderer);
    }
}
