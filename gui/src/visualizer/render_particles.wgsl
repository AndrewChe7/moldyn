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


@compute
@workgroup_size(1, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = i32(global_id.x);
    let y = i32(global_id.y);
    let x_relative = (f32(x) / f32(camera.width)) * 2.0 - 1.0;
    let y_relative = (f32(y) / f32(camera.height)) * 2.0 - 1.0;
    let particle = state[global_id.z];
    let particle_position = particle.position;
    let pos = vec3<f32>(f32(particle_position.x), f32(particle_position.y), f32(particle_position.z));
    let radius = f32(particle.potential_mass_id.y) * 0.1;
    let direction = camera.forward + camera.right * x_relative + camera.up * y_relative;
    let direction_normalized = normalize(direction.xyz) / 100.0;
    var collided = false;
    for (var i: i32 = 1; i < 100 && !collided; i += 1) {
        let check_point = camera.eye.xyz + direction_normalized * f32(i);
        let distance_to_particle = distance(check_point, pos);
        if (distance_to_particle < radius) {
            let c = f32(100 - i) / 100.0;
            textureStore(screen_texture,
                                vec2<i32>(x, y),
                                vec4<f32>(c, c, c, 1.0));
            break;
        }
    }
}