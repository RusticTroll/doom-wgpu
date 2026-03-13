struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) tex_coords: vec2f,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let vertices = array<vec3f, 6>(
        vec3f(-1.0, 1.0, 0.0),
        vec3f(1.0, 1.0, 0.0),
        vec3f(1.0, -1.0, 0.0),
        vec3f(-1.0, 1.0, 0.0),
        vec3f(-1.0, -1.0, 0.0),
        vec3f(1.0, -1.0, 0.0)
    );

    let tex_coords = array<vec2f, 6>(
        vec2(0.0, 0.0),
        vec2(1.0, 0.0),
        vec2(1.0, 1.0),
        vec2(0.0, 0.0),
        vec2(0.0, 1.0),
        vec2(1.0, 1.0)
    );

    var out: VertexOutput;
    out.clip_position = vec4(vertices[vertex_index], 1.0);
    out.tex_coords = tex_coords[vertex_index];

    return out;
}

@group(0) @binding(0)
var t_palette: texture_1d<f32>;

@group(1) @binding(0)
var t_palette_index: texture_2d<u32>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    let texture_dimensions = textureDimensions(t_palette_index);
    let coords = vec2i(in.tex_coords * vec2f(texture_dimensions));
    let palette_index = textureLoad(t_palette_index, coords, 0).r;

    return textureLoad(t_palette, palette_index, 0);
}