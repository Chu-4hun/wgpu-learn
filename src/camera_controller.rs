use winit::{
    event::{ElementState, KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};
use cgmath::{InnerSpace, Vector3};
use crate::camera::Camera;

#[derive(Default)]
pub struct CameraController {
    speed: f32,
    rotation_speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_rotate_left_pressed: bool,
    is_rotate_right_pressed: bool,
    is_rotate_up_pressed: bool,
    is_rotate_down_pressed: bool,

    is_down_pressed: bool,
    is_up_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            rotation_speed: 0.05, // Скорость поворота в радианах
            ..Default::default()
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        physical_key: PhysicalKey::Code(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    // WASD для движения
                    KeyCode::KeyW => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyS => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyA => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyD => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    // Стрелочки для поворота
                    KeyCode::ArrowLeft => {
                        self.is_rotate_left_pressed = is_pressed;
                        true
                    }
                    KeyCode::ArrowRight => {
                        self.is_rotate_right_pressed = is_pressed;
                        true
                    }
                    KeyCode::ArrowDown => {
                        self.is_rotate_up_pressed = is_pressed;
                        true
                    }
                    KeyCode::ArrowUp => {
                        self.is_rotate_down_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyE => {
                        self.is_up_pressed = is_pressed;
                        true
                    }
                    KeyCode::KeyQ => {
                        self.is_down_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn update_camera(&self, camera: &mut Camera, delta_time: f32) {
        // Получаем основные векторы для навигации
        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let right = forward_norm.cross(camera.up);

        if self.is_up_pressed {
            camera.eye += camera.up * self.speed * delta_time * 100.0;
            camera.target += camera.up * self.speed * delta_time * 100.0;
        }
        if self.is_down_pressed {
            camera.eye -= camera.up * self.speed * delta_time * 100.0;
            camera.target -= camera.up * self.speed * delta_time * 100.0;
        }
        // Движение вперед/назад
        if self.is_forward_pressed {
            camera.eye += forward_norm * self.speed * delta_time * 100.0;
            camera.target += forward_norm * self.speed * delta_time * 100.0;
        }
        if self.is_backward_pressed {
            camera.eye -= forward_norm * self.speed * delta_time * 100.0;
            camera.target -= forward_norm * self.speed * delta_time * 100.0;
        }

        // Движение влево/вправо
        if self.is_right_pressed {
            camera.eye += right * self.speed * delta_time * 100.0;
            camera.target += right * self.speed * delta_time * 100.0;
        }
        if self.is_left_pressed {
            camera.eye -= right * self.speed * delta_time * 100.0;
            camera.target -= right * self.speed * delta_time * 100.0;
        }

        // Поворот камеры
        let target_dir = (camera.target - camera.eye).normalize();

        // Поворот влево/вправо вокруг оси Y
        if self.is_rotate_left_pressed {
            let rotation = cgmath::Matrix3::from_axis_angle(
                Vector3::unit_y(),
                cgmath::Rad(self.rotation_speed* delta_time * 100.0)
            );
            let rotated_dir = rotation * target_dir;
            camera.target = camera.eye + rotated_dir * forward.magnitude();
        }
        if self.is_rotate_right_pressed {
            let rotation = cgmath::Matrix3::from_axis_angle(
                Vector3::unit_y(),
                cgmath::Rad(-self.rotation_speed* delta_time * 100.0)
            );
            let rotated_dir = rotation * target_dir;
            camera.target = camera.eye + rotated_dir * forward.magnitude();
        }

        // Поворот вверх/вниз вокруг правого вектора
        if self.is_rotate_up_pressed {
            let rotation = cgmath::Matrix3::from_axis_angle(
                right,
                cgmath::Rad(-self.rotation_speed)
            );
            let rotated_dir = rotation * target_dir;
            camera.target = camera.eye + rotated_dir * forward.magnitude();
        }
        if self.is_rotate_down_pressed {
            let rotation = cgmath::Matrix3::from_axis_angle(
                right,
                cgmath::Rad(self.rotation_speed)
            );
            let rotated_dir = rotation * target_dir;
            camera.target = camera.eye + rotated_dir * forward.magnitude();
        }
    }
}