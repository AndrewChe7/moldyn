use std::f32::consts::PI;
use cgmath::{InnerSpace, MetricSpace, Point3, Vector3};

pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub fovx: f32,
    pub forward: cgmath::Vector3<f32>,
    pub right: cgmath::Vector3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub width: u32,
    pub height: u32,
}

fn get_right_up(forward: &cgmath::Vector3<f32>, fovx: f32, width: u32, height: u32)
    -> (cgmath::Vector3<f32>, cgmath::Vector3<f32>) {
    let world_up = cgmath::Vector3::new(0.0, 1.0, 0.0);
    let mut right = forward.cross(world_up).normalize();
    let mut up = forward.cross(right).normalize();
    let angle_right = fovx.to_radians() / 2.0;
    let up_to_right = height as f32 / width as f32;
    let right_length = angle_right.tan();
    let up_length = right_length * up_to_right;
    up *= up_length;
    right *= right_length;
    (right, up)
}

impl Camera {
    pub fn new(eye: (f32, f32, f32), fovx: f32, screen_size: (u32, u32)) -> Self {
        let forward = cgmath::Vector3::new(1.0, 0.0, 0.0);
        let (right, up) = get_right_up(&forward, fovx, screen_size.0, screen_size.1);
        Self {
            eye: cgmath::Point3::new(eye.0, eye.1, eye.2),
            fovx,
            forward,
            right,
            up,
            width: screen_size.0,
            height: screen_size.1,
        }
    }

    pub fn update (&mut self,
                   new_position: (f32, f32, f32),
                   new_direction: (f32, f32, f32),
                   width: u32,
                   height: u32) {
        self.eye.x = new_position.0;
        self.eye.y = new_position.1;
        self.eye.z = new_position.2;
        self.forward.x = new_direction.0;
        self.forward.y = new_direction.1;
        self.forward.z = new_direction.2;
        self.forward = self.forward.normalize();
        self.width = width;
        self.height = height;
        let (right, up) = get_right_up(&self.forward, self.fovx, width, height);
        self.right = right;
        self.up = up;
    }

    pub fn look_to(&mut self, target: (f32, f32, f32)) -> (f32, f32, f32) {
        let direction = (
                target.0 - self.eye.x,
                target.1 - self.eye.y,
                target.2 - self.eye.z,
            );
        direction
    }

    pub fn rotate(&mut self, dx: f32, dy: f32, target: (f32, f32, f32), width: u32, height: u32) {
        let target_point = Point3::new(target.0, target.1, target.2);
        let radius = target_point.distance(self.eye);
        let direction = (self.forward + dx * self.right + dy * self.up).normalize();
        let new_position = target_point - direction * radius;
        let direction_xz = Vector3::new(direction.x, 0.0, direction.z);
        let angle = direction.angle(direction_xz).0;
        if angle > PI / 3.0 || angle < -PI/3.0 {
            return;
        }
        let new_position = (new_position.x, new_position.y, new_position.z);
        let new_direction = self.look_to(target);

        self.update(new_position, new_direction, width, height);
    }

    pub fn zoom(&mut self, delta: f32, width: u32, height: u32) {
        let new_position = self.eye + delta * self.forward;
        let new_position = (new_position.x, new_position.y, new_position.z);
        let new_direction = (self.forward.x, self.forward.y, self.forward.z);
        self.update(new_position, new_direction, width, height);
    }
}
