struct Camera {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
}

struct BoundingBox {
    size: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(1) @binding(0)
var<uniform> bounding_box: BoundingBox;

struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local_position: vec4<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    let s = bounding_box.size;
    let model_matrix = mat4x4<f32>(
        s.x, 0.0, 0.0, 0.0,
        0.0, s.y, 0.0, 0.0,
        0.0, 0.0, s.z, 0.0,
        0.0, 0.0, 0.0, 1.0
    );
    var out: VertexOutput;
    var world_position: vec4<f32> = model_matrix * vec4<f32>(model.position, 1.0);
    out.clip_position = camera.view_proj * world_position;
    out.local_position = vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let x = abs(in.local_position.x - 0.5);
    let y = abs(in.local_position.y - 0.5);
    let z = abs(in.local_position.z - 0.5);
    let dx = x > 0.495;
    let dy = y > 0.495;
    let dz = z > 0.495;
    let edge = (dx && dy) || (dx && dz) || (dy && dz);
    var c = 0.0;
    if (edge) {
        c = 0.1;
    }
    return vec4<f32>(1.0, 1.0, 1.0, c);
}