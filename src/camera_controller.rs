use crate::camera::Camera;
use cgmath::{InnerSpace, Quaternion, Rad, Rotation, Rotation3, Vector3};
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
            // WindowEvent::Resized(_) => {
            //     // Сбрасываем положение мыши
            //     self.mouse_last_pos = (0.0, 0.0);
            //     self.mouse_delta = (0.0, 0.0);
            //     true
            // }
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
        // Основные векторы
        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let right = forward_norm.cross(camera.up).normalize();

        // Движение камеры
        if self.is_up_pressed {
            camera.eye += camera.up * self.speed * delta_time * 100.0;
            camera.target += camera.up * self.speed * delta_time * 100.0;
        }
        if self.is_down_pressed {
            camera.eye -= camera.up * self.speed * delta_time * 100.0;
            camera.target -= camera.up * self.speed * delta_time * 100.0;
        }
        if self.is_forward_pressed {
            camera.eye += forward_norm * self.speed * delta_time * 100.0;
            camera.target += forward_norm * self.speed * delta_time * 100.0;
        }
        if self.is_backward_pressed {
            camera.eye -= forward_norm * self.speed * delta_time * 100.0;
            camera.target -= forward_norm * self.speed * delta_time * 100.0;
        }
        if self.is_right_pressed {
            camera.eye += right * self.speed * delta_time * 100.0;
            camera.target += right * self.speed * delta_time * 100.0;
        }
        if self.is_left_pressed {
            camera.eye -= right * self.speed * delta_time * 100.0;
            camera.target -= right * self.speed * delta_time * 100.0;
        }

        // Повороты камеры
        let yaw = Rad(self.mouse_delta.0 * self.mouse_sensitivity);
        let pitch = Rad(self.mouse_delta.1 * self.mouse_sensitivity);

        // Ограничение угла наклона (pitch)
        let max_pitch = Rad(89.0f32.to_radians());
        let current_pitch = forward_norm.dot(camera.up).asin();
        let new_pitch = (current_pitch + pitch.0).clamp(-max_pitch.0, max_pitch.0);

        // Рассчитываем новые направления с использованием Quaternion
        let yaw_quaternion = Quaternion::from_angle_y(yaw);
        let pitch_quaternion = Quaternion::from_axis_angle(right, Rad(new_pitch - current_pitch));

        // Обновляем направление взгляда
        let mut forward_rotated = yaw_quaternion.rotate_vector(forward_norm);
        forward_rotated = pitch_quaternion.rotate_vector(forward_rotated).normalize();

        // Перемещение цели камеры
        camera.target = camera.eye + forward_rotated * forward.magnitude();

        // Сбрасываем дельту мыши
        self.mouse_delta = (0.0, 0.0);
    }
}
