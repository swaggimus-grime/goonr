// Vertex input struct — to be defined later via vertex buffer
struct VertexInput {
    @location(0) position: vec3<f32>, // mean
    @location(1) color: vec3<f32>,    // maybe sh_coeffs for now
};

// Output to fragment
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) frag_color: vec3<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    // We'll eventually apply model-view-projection transformation here
    output.clip_position = vec4<f32>(input.position, 1.0);
    output.frag_color = input.color;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(input.frag_color, 1.0); // full alpha
}
