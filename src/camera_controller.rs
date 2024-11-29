use crate::camera::Camera;
use cgmath::{InnerSpace, Vector3};
use winit::{
    event::{ElementState, KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

#[derive(Default)]
pub struct CameraController {
    speed: f32,
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

    mouse_last_pos: (f32, f32),
    mouse_delta: (f32, f32),
    mouse_sensitivity: f32,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            mouse_sensitivity: 0.002, // Скорость поворота в радианах
            ..Default::default()
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => {
                if self.mouse_last_pos.0 == 0.0 && self.mouse_last_pos.1 == 0.0 {
                    self.mouse_last_pos = (position.x as f32, position.y as f32)
                }
                self.mouse_delta = (
                    self.mouse_last_pos.0 - position.x as f32,
                    self.mouse_last_pos.1 - position.y as f32,
                );

                false
            }
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

    pub fn update_camera(&mut self, camera: &mut Camera, delta_time: f32) {
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
        let yaw = cgmath::Rad(-self.mouse_delta.0 * self.mouse_sensitivity);
        let pitch = cgmath::Rad(-self.mouse_delta.1 * self.mouse_sensitivity);

        // Yaw rotation (around the up vector)
        let yaw_rotation = cgmath::Matrix3::from_axis_angle(Vector3::unit_y(), yaw * -1.0);
        let forward_rotated = yaw_rotation * forward_norm;

        // Pitch rotation (around the right vector)
        let pitch_rotation = cgmath::Matrix3::from_axis_angle(right, pitch * -1.0);
        let final_rotated = pitch_rotation * forward_rotated;

        camera.target = camera.eye + final_rotated * forward.magnitude();
        self.mouse_delta = (0.0, 0.0);
    }
}
