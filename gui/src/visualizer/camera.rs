use cgmath::{InnerSpace, Matrix4, MetricSpace, Point3, Quaternion, Vector3, Vector4};

pub struct Camera {
    pub eye: Point3<f32>,
    pub fovx: f32,
    pub forward: Vector3<f32>,
    pub right: Vector3<f32>,
    pub up: Vector3<f32>,
    pub width: u32,
    pub height: u32,
}

fn get_right_up(forward: &Vector3<f32>, fovx: f32, width: u32, height: u32)
    -> (Vector3<f32>, Vector3<f32>) {
    let world_up = Vector3::new(0.0, 1.0, 0.0);
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
        let forward = Vector3::new(1.0, 0.0, 0.0);
        let (right, up) = get_right_up(&forward, fovx, screen_size.0, screen_size.1);
        Self {
            eye: Point3::new(eye.0, eye.1, eye.2),
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

    pub fn get_model_matrix(&self) -> Matrix4<f32> {
        let rotation = Quaternion::from_arc(Vector3::new(1.0, 0.0, 0.0), self.forward, None);
        let res = cgmath::Matrix4::from(rotation);
        res
    }
    
    

    pub fn rotate(&mut self, rotation: Matrix4<f32>, target: (f32, f32, f32), width: u32, height: u32) {
        let direction = (rotation * Vector4::new(1.0, 0.0, 0.0, 0.0))
            .normalize().truncate();
        let target_point = Point3::new(target.0, target.1, target.2);
        let radius = self.eye.distance(target_point);
        let position = target_point - direction * radius;
        let new_direction = (direction.x, direction.y, direction.z);
        let new_position = (position.x, position.y, position.z);
        self.update(new_position, new_direction, width, height);
    }

    pub fn zoom(&mut self, delta: f32, width: u32, height: u32) {
        let new_position = self.eye + delta * self.forward;
        let new_position = (new_position.x, new_position.y, new_position.z);
        let new_direction = (self.forward.x, self.forward.y, self.forward.z);
        self.update(new_position, new_direction, width, height);
    }
}
