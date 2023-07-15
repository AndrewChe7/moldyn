struct Camera {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
}

struct VisualizationParameter {
    gradient_color_min: vec4<f32>,
    gradient_color_mid: vec4<f32>,
    gradient_color_max: vec4<f32>,
    gradient_min_max: vec4<f32>,
    visualization_parameter_type: u32,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(1) @binding(0)
var<uniform> vis_param: VisualizationParameter;

struct VertexInput {
    @location(0) position: vec3<f32>,
}
struct InstanceInput {
    @location(1) position: vec4<f32>,
    @location(2) velocity: vec4<f32>,
    @location(3) mass_radius_id: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) sphere_center: vec4<f32>,
    @location(1) world_position: vec4<f32>,
    @location(2) radius: f32,
    @location(3) color: vec4<f32>,
}

fn get_color (id: u32) -> vec4<f32> {
    var res = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    let x = id % 6u;
    switch (x) {
        case 0u: {
            res = vec4<f32>(1.0, 0.0, 0.0, 0.0);
        }
        case 1u: {
            res = vec4<f32>(0.0, 1.0, 0.0, 0.0);
        }
        case 2u: {
            res = vec4<f32>(0.0, 0.0, 1.0, 0.0);
        }
        case 3u: {
            res = vec4<f32>(1.0, 1.0, 0.0, 0.0);
        }
        case 4u: {
            res = vec4<f32>(1.0, 0.0, 1.0, 0.0);
        }
        case 5u: {
            res = vec4<f32>(0.0, 1.0, 1.0, 0.0);
        }
        default: {
            res = vec4<f32>(0.0, 0.0, 0.0, 0.0);
        }
    }
    return res;
}


@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let s = instance.mass_radius_id.y;
    let model_matrix = mat4x4<f32>(
        s, 0.0, 0.0, 0.0,
        0.0, s, 0.0, 0.0,
        0.0, 0.0, s, 0.0,
        instance.position.x, instance.position.y, instance.position.z, 1.0
    );
    var out: VertexOutput;
    var world_position: vec4<f32> = model_matrix * vec4<f32>(model.position, 1.0);
    out.clip_position = camera.view_proj * world_position;
    out.world_position = world_position;
    out.sphere_center = instance.position;
    out.radius = distance(world_position.xyz, instance.position.xyz);

    switch (vis_param.visualization_parameter_type) {
        case 0u: {
            let id = u32(instance.mass_radius_id[2]);
            out.color = get_color(id);
        }
        case 1u: {
            let gm = vis_param.gradient_min_max.x;
            let gM = vis_param.gradient_min_max.y;
            let v = (length(instance.velocity) - gm) / (gM - gm);
            let m = vis_param.gradient_color_min;
            let x = vis_param.gradient_color_mid;
            let M = vis_param.gradient_color_max;
            let c = v * M / 2.0 + (1.0 - v) * m / 2.0 + (1.0 - abs(0.5 - v)) * x / 2.0;
            out.color = c;
        }
        default: {
            out.color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
        }
    }

    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var c = 0.01;
    let normal = normalize(in.world_position - in.sphere_center).xyz;
    let pos = in.sphere_center.xyz + normal * in.radius;
    let view_vector = normalize(camera.view_pos.xyz - pos);
    let light_dir = -normalize(vec3<f32>(1.0, -1.0, 1.0));
    c += pow(max(dot(normal, light_dir), 0.0), 3.0);
    let half_dir = normalize(view_vector + light_dir);
    c += pow(max(dot(normal, half_dir), 0.0), 32.0);
    return vec4<f32>(in.color.xyz * c, 1.0);
}