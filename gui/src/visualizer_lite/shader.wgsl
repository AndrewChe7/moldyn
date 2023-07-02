struct Camera {
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
    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}