struct Camera {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
}
@group(0) @binding(0)
var<uniform> camera: Camera;

struct VertexInput {
    @location(0) position: vec3<f32>,
}
struct InstanceInput {
    @location(1) position: vec4<f32>,
    @location(2) mass_id: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) sphere_center: vec4<f32>,
    @location(1) world_position: vec4<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let s = instance.mass_id.x;
    let model_matrix = mat4x4<f32>(
        s, 0.0, 0.0, instance.position.x,
        0.0, s, 0.0, instance.position.y,
        0.0, 0.0, s, instance.position.z,
        0.0, 0.0, 0.0, 1.0
    );
    var out: VertexOutput;
    var world_position: vec4<f32> = model_matrix * vec4<f32>(model.position * s, 1.0);
    out.clip_position = camera.view_proj * world_position;
    out.world_position = world_position;
    out.sphere_center = instance.position;
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var c = 0.01;
    let view_vector = normalize(camera.view_pos.xyz - in.world_position.xyz);
    let normal = normalize(in.world_position - in.sphere_center).xyz;
    let light_dir = -normalize(vec3<f32>(1.0, -1.0, 1.0));
    c += pow(max(dot(normal, light_dir), 0.0), 3.0);
    let half_dir = normalize(view_vector + light_dir);
    c += pow(max(dot(normal, half_dir), 0.0), 32.0);
    return vec4<f32>(c, c, c, 1.0);
}