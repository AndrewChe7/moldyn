use cgmath::InnerSpace;

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
    let angle_up = angle_right / width as f32 * height as f32;
    let right_length = angle_right.tan();
    let up_length = angle_up.tan();
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

    pub fn _update_position(&mut self, dx: f32, dy: f32, width: u32, height: u32) {
        let (right, up) = get_right_up(&self.forward, self.fovx, width, height);
        self.eye += self.forward * dy + self.right * dx;
        self.right = right;
        self.up = up;
        self.width = width;
        self.height = height;
    }

    pub fn _update_angle(&mut self, dx: f32, dy: f32, width: u32, height: u32) {
        let forward = self.forward + self.right * dx + self.up * dy;
        let forward = forward.normalize();
        let (right, up) = get_right_up(&forward, self.fovx, width, height);
        self.forward = forward;
        self.right = right;
        self.up = up;
        self.width = width;
        self.height = height;
    }

}
