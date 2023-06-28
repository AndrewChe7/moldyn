// #[rustfmt::skip]
// pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
//     1.0, 0.0, 0.0, 0.0,
//     0.0, 1.0, 0.0, 0.0,
//     0.0, 0.0, 0.5, 0.0,
//     0.0, 0.0, 0.5, 1.0,
// );
//
// pub struct Camera {
//     eye: cgmath::Point3<f32>,
//     target: cgmath::Point3<f32>,
//     up: cgmath::Vector3<f32>,
//     aspect: f32,
//     fovy: f32,
//     znear: f32,
//     zfar: f32,
// }
//
// impl Camera {
//     pub fn new(eye: (f32, f32, f32), target: (f32, f32, f32), aspect: f32) -> Self {
//         Camera {
//             // position the camera one unit up and 2 units back
//             // +z is out of the screen
//             eye: eye.into(),
//             // have it look at the origin
//             target: target.into(),
//             // which way is "up"
//             up: cgmath::Vector3::unit_y(),
//             aspect,
//             fovy: 45.0,
//             znear: 0.1,
//             zfar: 100.0,
//         }
//     }
//
//     fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
//         let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
//         let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
//         return OPENGL_TO_WGPU_MATRIX * proj * view;
//     }
// }