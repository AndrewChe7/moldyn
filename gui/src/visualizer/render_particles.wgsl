struct Particle {
    position: vec3<f64>,
    velocity: vec3<f64>,
    force: vec3<f64>,
    potential: f64,
    mass: f64,
    id: u32,
    _padding: u32,
};

struct CameraData {
    eye: vec3<f32>,
    fovx: f32,
    forward: vec3<f32>,
    right: vec3<f32>,
    up: vec3<f32>,
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


@compute
@workgroup_size(1, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var res = vec4<f32>(0.01, 0.01, 0.02, 1.0);
    let x = i32(global_id.x);
    let y = i32(global_id.y);
    let particle_id = global_id.z;
    let particle_position = state[particle_id].position;
    let pos = vec3<f32>(0.0, 0.0, 0.0);//vec3<f32>(f32(particle_position.x), f32(particle_position.y), f32(particle_position.z));
    let radius = 0.1;//f32(state[particle_id].mass) * 0.1;
    let x_relative = (f32(x) / f32(camera.width)) * 2.0 - 1.0;
    let y_relative = (f32(y) / f32(camera.height)) * 2.0 - 1.0;
    let direction = camera.forward + camera.right * x_relative + camera.up * y_relative;
    let direction_normalized = normalize(direction) / 100.0;
    for(var i: i32 = 1; i < 100; i += 1) {
        let check_point = camera.eye + direction_normalized * f32(i);
        let distance_to_particle = distance(check_point, pos);
        if (distance_to_particle < radius) {
            res = vec4<f32>(1.0, 1.0, 1.0, 1.0);
            break;
        }
    }
    textureStore(screen_texture,
                    vec2<i32>(x, y),
                    res);

}