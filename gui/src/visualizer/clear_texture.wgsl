struct Particle {
    position: vec4<f64>,
    velocity: vec4<f64>,
    force: vec4<f64>,
    potential_mass_id: vec4<f64>,
};

struct CameraData {
    eye: vec4<f32>,
    forward: vec4<f32>,
    right: vec4<f32>,
    up: vec4<f32>,
    fovx: f32,
    width: u32,
    height: u32,
    _padding: u32,
};

@group(0)
@binding(0)
var<storage, read_write> state: array<Particle>;
@group(0)
@binding(1)
var screen_texture : texture_storage_2d<rgba8unorm, write>;
@group(0)
@binding(2)
var<uniform> camera: CameraData;
@group(0)
@binding(3)
var<storage, read_write> depth_buffer: array<f32>;

@compute
@workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = i32(global_id.x);
    let y = i32(global_id.y);
    textureStore(screen_texture,
                        vec2<i32>(x, y),
                        vec4<f32>(0.01, 0.01, 0.02, 1.0));
    depth_buffer[y * i32(camera.width) + x] = 0.0;
}