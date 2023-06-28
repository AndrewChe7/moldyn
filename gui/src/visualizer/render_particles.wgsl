struct Particle {
    position: vec3<f64>,
    velocity: vec3<f64>,
    force: vec3<f64>,
    potential: f64,
    mass: f64,
    id: u32,
    _padding: u32,
};

@group(0)
@binding(0)
var<storage, read_write> state: array<Particle>;
@group(0)
@binding(1)
var screen_texture : texture_storage_2d<rgba8unorm, write>;

@compute
@workgroup_size(1, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var res = vec4<f32>(0.01, 0.01, 0.02, 1.0);
    let x = i32(global_id.x);
    let y = i32(global_id.y);
    let particle = global_id.z;

    if (abs(x - y) < 10) {
        res = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    }
    textureStore(screen_texture,
                    vec2<i32>(x, y),
                    res);

}