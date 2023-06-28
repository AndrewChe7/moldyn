struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0)      uv: vec2<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    let i = in_vertex_index;
    let x = select(-1.0, 1.0, i == 2u || i == 3u || i == 5u);
    let y = select(-1.0, 1.0, i == 0u || i == 2u || i == 3u);
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = (vec2<f32>(x, -y) + 1.0) * 0.5;
    return out;
}

@group(0)
@binding(0)
var screen_texture : texture_2d<f32>;
@group(0)
@binding(1)
var screen_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(screen_texture, screen_sampler, in.uv);
}