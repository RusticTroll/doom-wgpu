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
var t_palette: texture_2d<f32>;

@group(0) @binding(1)
var t_color_map: texture_2d<u32>;

@group(1) @binding(0)
var t_palette_index: texture_2d<u32>;

struct ColorMapping {
    palette_number: u32,
    color_map_index: u32,
}
var<immediate> color_mapping: ColorMapping;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    let texture_dimensions = textureDimensions(t_palette_index);
    let coords = vec2i(in.tex_coords * vec2f(texture_dimensions));
    let palette_index = textureLoad(t_palette_index, coords, 0).r;

    if palette_index > 255 {
        return vec4f(1.0, 1.0, 1.0, 0.0);
    }

    let mapped_index = textureLoad(t_color_map, vec2(palette_index, color_mapping.color_map_index), 0).r;

    return textureLoad(t_palette, vec2(mapped_index, color_mapping.palette_number), 0);
}