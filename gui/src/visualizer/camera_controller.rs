use cgmath::{Euler, Quaternion, Rad};
use winit::event::{DeviceEvent, ElementState, MouseScrollDelta};
use crate::visualizer::camera::Camera;

pub struct CameraController {
    speed: f32,
    scroll: f32,
    delta_mouse: (f32, f32),
    is_mouse_key_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            scroll: 0.0,
            delta_mouse: (0.0, 0.0),
            is_mouse_key_pressed: false,
        }
    }

    pub fn process_events(&mut self, event: &DeviceEvent) -> bool {
        match event {
            DeviceEvent::MouseMotion {
                delta: (x, y),
                ..
            } => {
                self.delta_mouse = (*x as f32, *y as f32);
                true
            },
            DeviceEvent::MouseWheel {
                delta
            } => {
                let dy =
                match delta {
                    MouseScrollDelta::LineDelta(_, y) => {
                        *y
                    },
                    MouseScrollDelta::PixelDelta(delta) => {
                        delta.y as f32
                    }
                };
                self.scroll = dy;
                true
            },
            DeviceEvent::Button {
                button,
                state
            } => {
                match button {
                    1 | 3 => {
                        match state {
                            ElementState::Pressed => {
                                self.is_mouse_key_pressed = true;
                            },
                            ElementState::Released => {
                                self.is_mouse_key_pressed = false;
                            }
                        }
                        true
                    },
                    _ => {
                        false
                    }
                }
            },
            _ => {
                false
            },
        }
    }

    pub fn update_camera(&mut self, camera: &mut Camera, width: u32, height: u32) {
        if self.is_mouse_key_pressed {
            let dx = if self.delta_mouse.0 > 0.0 {
                self.speed
            } else if self.delta_mouse.0 < 0.0 {
                -self.speed
            } else {
                0.0
            };
            let dy = if self.delta_mouse.1 > 0.0 {
                self.speed
            } else if self.delta_mouse.1 < 0.0 {
                -self.speed
            } else {
                0.0
            };
            let rotation = camera.get_model_matrix();
            let quaternion_rotation = Quaternion::from(Euler::new(Rad(dy*0.5), Rad(-dx*0.5), Rad(0.0)));
            let rotate = cgmath::Matrix4::from(quaternion_rotation);
            let matrix = rotate * rotation;
            camera.rotate(matrix, (0.0, 0.0, 0.0), width, height);
        }
        if self.scroll.abs() > 0.0 {
            camera.zoom(self.scroll * self.speed, width, height);
        }
        self.scroll = 0.0;
        self.delta_mouse = (0.0, 0.0);
    }
}
