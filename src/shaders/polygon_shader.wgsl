struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};


@vertex
fn vs_main(
    model: VertexInput,
    @builtin(vertex_index) vIdx: u32
) -> VertexOutput {
    let size = 0.01;
    var points = array(
        vec2f(-size, -size), // bottom-left
        vec2f( size, -size), // bottom-right
        vec2f(-size,  size), // top-left

        vec2f( size,  size), // top-right
        vec2f(-size,  size), // top-left
        vec2f( size, -size), // bottom-right
    );
    let pos = points[vIdx];
    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = vec4<f32>(model.position + vec3<f32>(pos, 0.0), 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}
