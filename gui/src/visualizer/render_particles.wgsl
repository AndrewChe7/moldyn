struct Particle {
    position: vec4<f64>,
    velocity: vec4<f64>,
    force: vec4<f64>,
    potential_mass_radius_id: vec4<f64>,
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
var<storage, read_write> state: array<Particle, 512>;
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
@workgroup_size(8, 8, 4)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = i32(global_id.x);
    let y = i32(global_id.y);
    let x_relative = (f32(x) / f32(camera.width)) * 2.0 - 1.0;
    let y_relative = (f32(y) / f32(camera.height)) * 2.0 - 1.0;
    let light_dir = -normalize(vec3<f32>(1.0, -1.0, 1.0));
    let particle = state[global_id.z];
    let particle_position = particle.position;
    let pos = vec3<f32>(f32(particle_position.x), f32(particle_position.y), f32(particle_position.z));
    let radius = f32(particle.potential_mass_radius_id.z);
    let direction = camera.forward + camera.right * x_relative + camera.up * y_relative;
    let direction_normalized = normalize(direction.xyz);;
    var ray_point = camera.eye.xyz;
    for (var i: i32 = 0; i < 32; i += 1) {
        let distance_to_ball = distance(ray_point, pos) - radius;
        let check_point = ray_point + direction_normalized * distance_to_ball;
        let distance_to_particle = distance(check_point, pos);
        let distance_to_camera = distance(check_point, camera.eye.xyz);
        if (distance_to_particle > 2.0) {
            return;
        }
        if (distance_to_camera > 40.0) {
            return;
        }
        if (distance_to_particle - radius < 0.001) {
            let depth = (20.0 - distance_to_camera) / 20.0;
            if (depth < depth_buffer[y * i32(camera.width) + x]) {
                return;
            }
            var c = 0.01;
            let normal = normalize(check_point - pos);
            let view_vector = normalize(camera.eye.xyz - check_point);
            c += pow(max(dot(normal, light_dir), 0.0), 3.0);
            let half_dir = normalize(view_vector + light_dir);
            c += pow(max(dot(normal, half_dir), 0.0), 32.0);
            depth_buffer[y * i32(camera.width) + x] = depth;
            textureStore(screen_texture,
                                vec2<i32>(x, y),
                                vec4<f32>(c, c, c, 1.0));
            return;
        }
        ray_point = check_point;
    }
}