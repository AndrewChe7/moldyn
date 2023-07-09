use cgmath::{Euler, Quaternion, Rad};
use winit::event::{DeviceEvent, ElementState, KeyboardInput, MouseScrollDelta, VirtualKeyCode};
#[cfg(target_arch = "wasm32")]
use winit::event::WindowEvent;

use crate::visualizer::camera::Camera;

pub struct CameraController {
    speed: f32,
    scroll: f32,
    delta_mouse: (f32, f32),
    is_mouse_key_pressed: bool,
    wasd : (bool, bool, bool, bool),
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            scroll: 0.0,
            delta_mouse: (0.0, 0.0),
            is_mouse_key_pressed: false,
            wasd: (false, false, false, false),
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
            DeviceEvent::Key(key) => {
                self.process_keys(key)
            }
            _ => {
                false
            },
        }
    }

    fn process_keys (&mut self, key: &KeyboardInput) -> bool {
        let keycode = key.virtual_keycode;
        if keycode.is_none() {
            false
        } else {
            let keycode = keycode.unwrap();
            match keycode {
                VirtualKeyCode::W => {
                    self.wasd.0 = if key.state == ElementState::Pressed {
                        true
                    } else {
                        false
                    };
                    true
                }
                VirtualKeyCode::S => {
                    self.wasd.2  = if key.state == ElementState::Pressed {
                        true
                    } else {
                        false
                    };
                    true
                }
                VirtualKeyCode::A => {
                    self.wasd.1  = if key.state == ElementState::Pressed {
                        true
                    } else {
                        false
                    };
                    true
                }
                VirtualKeyCode::D => {
                    self.wasd.3  = if key.state == ElementState::Pressed {
                        true
                    } else {
                        false
                    };
                    true
                }
                _ => {
                    false
                }
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn process_window_events (&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input, ..
            } => {
                self.process_keys(input)
            }
            _ => {
                false
            }
        }

    }

    pub fn update_camera(&mut self, camera: &mut Camera, target: (f32, f32, f32), width: u32, height: u32) {
        let key_pressed = self.wasd.0 || self.wasd.1 || self.wasd.2 || self.wasd.3;
        if self.is_mouse_key_pressed || key_pressed {
            let dx = if self.delta_mouse.0 > 0.0 || (self.wasd.3 && !self.wasd.1) {
                self.speed
            } else if self.delta_mouse.0 < 0.0 || (!self.wasd.3 && self.wasd.1) {
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
            camera.rotate(matrix, target, width, height);
        }
        if self.scroll.abs() > 0.0 || self.wasd.0 || self.wasd.2 {
            let mut scroll = self.scroll;
            if self.wasd.0 && !self.wasd.2 && scroll < 0.05 {
                scroll = 1.0;
            } else if !self.wasd.0 && self.wasd.2 {
                scroll = -1.0;
            }
            camera.zoom(scroll * self.speed, width, height);
        }
        self.scroll = 0.0;
        self.delta_mouse = (0.0, 0.0);
    }
}
