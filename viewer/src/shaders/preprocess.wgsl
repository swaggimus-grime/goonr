@group(0) @binding(0) var<storage, read> input_splats: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> output_splats: array<vec4<f32>>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    if (idx >= arrayLength(&input_splats)) {
        return;
    }

    let splat = input_splats[idx];

    // Example preprocessing: normalize color or do some transform
    let processed = vec4<f32>(splat.xyz * 0.5, splat.w);

    output_splats[idx] = processed;
}